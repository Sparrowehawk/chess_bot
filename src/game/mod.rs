pub mod fen;
pub mod perft;

use crate::board::Bitboard;
use crate::board::display::print_board;
use crate::board::movegen::generate_pseudo_legal_moves;
use crate::search::pst::{get_piece_at, get_piece_colour_at};
use crate::search::tt::TranspositionTable;
use crate::search::zobrist::{ZobristKeys, compute_zobrist_hash};
use crate::{MoveList, Piece}; // Import Bitboard from the appropriate module
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    io::{self, Write},
};

pub static ZOBRIST_KEYS: Lazy<ZobristKeys> = Lazy::new(ZobristKeys::new);

#[derive(Clone)]
pub struct Game {
    pub board: Bitboard,
    pub is_white_turn: bool,
    pub castling: u8, // This will be represented with a 8 digit binary
    pub en_passent: Option<usize>,
    pub position_history: HashMap<u64, u32>, // Essentially, en_passent moves are pushed onto the vec and popped off after 1 turn
    pub tt: Arc<Mutex<TranspositionTable>>,
    pub zobrist_hash: u64,
}
#[derive(Clone)]
pub struct Undo {
    pub from: usize,
    pub to: usize,
    pub captured_piece: Option<Piece>,
    pub promotion: Option<Piece>,
    pub previous_castling_rights: u8,
    pub previous_en_passant_square: Option<usize>,
    pub previous_zobrist_hash: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    Normal,
    Check,
    Checkmate,
    Stalemate,
    RepMoves,
}

impl Default for Game {
    fn default() -> Self {
        let game = Self {
            board: Bitboard::new(),
            is_white_turn: true,
            castling: 0b11111111,
            en_passent: None,
            position_history: HashMap::new(),
            tt: Arc::new(Mutex::new(TranspositionTable::new())),
            zobrist_hash: 0,
        };
        let mut game = game;
        game.zobrist_hash = compute_zobrist_hash(game.clone());
        game
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn make_move(&mut self, from: usize, to: usize, promo: Option<Piece>) -> bool {
        let original_board = self.board.clone();
        let original_castling = self.castling;
        let original_en_passent = self.en_passent;
        let moving_side = self.is_white_turn;

        let from_mask = 1u64 << from;
        let en_passent_available = self.en_passent;
        self.en_passent = None;

        if promo.is_some() && (self.board.white_pawns | self.board.black_pawns) & from_mask == 0 {
            return false;
        }

        let move_success = if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            self.board.move_pawn(
                from,
                to,
                moving_side,
                promo,
                en_passent_available,
                &mut self.en_passent,
            )
        } else if (self.board.white_knight | self.board.black_knight) & from_mask != 0 {
            self.board.move_knight(from, to, moving_side)
        } else if (self.board.white_bishop | self.board.black_bishop) & from_mask != 0 {
            self.board.move_bishop(from, to, moving_side)
        } else if (self.board.white_rook | self.board.black_rook) & from_mask != 0 {
            self.board
                .move_rook(from, to, moving_side, &mut self.castling)
        } else if (self.board.white_queen | self.board.black_queen) & from_mask != 0 {
            self.board.move_queen(from, to, moving_side)
        } else if (self.board.white_king | self.board.black_king) & from_mask != 0 {
            self.board
                .move_king(from, to, moving_side, &mut self.castling)
        } else {
            false
        };

        if !move_success {
            return false;
        }

        let king_board = if moving_side {
            self.board.white_king
        } else {
            self.board.black_king
        };

        if king_board == 0 {
            self.board = original_board;
            self.castling = original_castling;
            self.en_passent = original_en_passent;
            return false;
        }

        let king_pos = king_board.trailing_zeros() as usize;

        if self.board.possible_check(king_pos, !moving_side) {
            self.board = original_board;
            self.castling = original_castling;
            self.en_passent = original_en_passent;
            return false;
        }

        self.is_white_turn = !self.is_white_turn;
        true
    }

    // If the king IS in check (bozo)
    pub fn is_in_check(&self) -> bool {
        let king_board = if self.is_white_turn {
            self.board.white_king
        } else {
            self.board.black_king
        };

        if king_board == 0 {
            return false;
        }

        let king_pos = king_board.trailing_zeros() as usize;
        self.board.possible_check(king_pos, !self.is_white_turn)
    }

    // Checks for castling rights if it still can
    fn castling_check(&mut self) {
        // White king-side and queen-side
        if self.castling & (1 << 4) != 0 {
            // White queen-side
            self.is_white_turn = true;
            let undo1 = self.make_move_unchecked(60, 59, None);
            let safe1 = !self.is_in_check();
            self.unmake_move(undo1);

            let undo2 = self.make_move_unchecked(60, 58, None);
            let safe2 = !self.is_in_check();
            self.unmake_move(undo2);

            if safe1 && safe2 {
                self.castling |= 1 << 0; // Allow white queen-side
            } else {
                self.castling &= !(1 << 0);
            }

            // White king-side
            let undo3 = self.make_move_unchecked(60, 61, None);
            let safe3 = !self.is_in_check();
            self.unmake_move(undo3);

            let undo4 = self.make_move_unchecked(60, 62, None);
            let safe4 = !self.is_in_check();
            self.unmake_move(undo4);

            if safe3 && safe4 {
                self.castling |= 1 << 1; // Allow white king-side
            } else {
                self.castling &= !(1 << 1);
            }
        } else {
            self.castling &= !(1 << 0);
            self.castling &= !(1 << 1);
        }

        // Black king-side and queen-side
        if self.castling & (1 << 5) != 0 {
            self.is_white_turn = false;

            // Black queen-side
            let undo5 = self.make_move_unchecked(4, 3, None);
            let safe5 = !self.is_in_check();
            self.unmake_move(undo5);

            let undo6 = self.make_move_unchecked(4, 2, None);
            let safe6 = !self.is_in_check();
            self.unmake_move(undo6);

            if safe5 && safe6 {
                self.castling |= 1 << 2; // Allow black queen-side
            } else {
                self.castling &= !(1 << 2);
            }

            // Black king-side
            let undo7 = self.make_move_unchecked(4, 5, None);
            let safe7 = !self.is_in_check();
            self.unmake_move(undo7);

            let undo8 = self.make_move_unchecked(4, 6, None);
            let safe8 = !self.is_in_check();
            self.unmake_move(undo8);

            if safe7 && safe8 {
                self.castling |= 1 << 3; // Allow black king-side
            } else {
                self.castling &= !(1 << 3);
            }
        } else {
            self.castling &= !(1 << 2);
            self.castling &= !(1 << 3);
        }
    }

    pub fn generate_legal_moves(&self) -> MoveList {
        let mut legal_moves = MoveList::new();
        let pseudo_legal_moves = generate_pseudo_legal_moves(self);

        for &(from, to, promo) in pseudo_legal_moves.iter() {
            let mut temp_game = self.clone();
            let ok = temp_game.make_move(from, to, promo);
            if ok {
                legal_moves.add(from, to, promo);
            }
        }

        legal_moves
    }

    pub fn game_state(&mut self) -> GameState {
        let is_in_check = self.is_in_check();
        let has_legal_moves = !self.generate_legal_moves().is_empty();

        if is_in_check {
            if has_legal_moves {
                if self.is_white_turn {
                    self.castling &= !(1 << 6)
                } else {
                    self.castling &= !(1 << 7)
                };
                GameState::Check
            } else {
                GameState::Checkmate
            }
        } else if has_legal_moves {
            if self.is_white_turn {
                self.castling |= 1 << 6
            } else {
                self.castling |= 1 << 7
            }
            GameState::Normal
        } else {
            GameState::Stalemate
        }
    }

    pub fn make_move_unchecked(&mut self, from: usize, to: usize, promo: Option<Piece>) -> Undo {
        let piece_moving = get_piece_at(self, from).expect("No piece on 'from' square");
        let side = if self.is_white_turn { 0 } else { 1 }; // 0 = white, 1 = black
        let opponent_side = 1 - side;

        let previous_en_passant = self.en_passent;
        let previous_castling = self.castling;
        let previous_hash = self.zobrist_hash;

        let mut captured_piece = get_piece_at(self, to);

        let is_en_passant = piece_moving == Piece::Pawn && Some(to) == previous_en_passant;
        if is_en_passant {
            captured_piece = Some(Piece::Pawn);
        }

        let undo = Undo {
            from,
            to,
            captured_piece,
            promotion: promo,
            previous_castling_rights: previous_castling,
            previous_en_passant_square: previous_en_passant,
            previous_zobrist_hash: previous_hash,
        };

        // === Zobrist Unhash Old State ===

        self.zobrist_hash ^= ZOBRIST_KEYS.side_to_move_key;

        if let Some(ep_sq) = previous_en_passant {
            let file = ep_sq % 8;
            self.zobrist_hash ^= ZOBRIST_KEYS.en_passent_keys[file];
        }

        self.zobrist_hash ^= ZOBRIST_KEYS.castling_keys[(previous_castling & 0x0F) as usize];

        // XOR out moving piece at 'from'
        self.zobrist_hash ^= ZOBRIST_KEYS.piece_keys[side][piece_moving as usize][from];

        // XOR out captured piece
        if let Some(piece) = captured_piece {
            let capture_square = if is_en_passant {
                if self.is_white_turn { to - 8 } else { to + 8 }
            } else {
                to
            };

            self.zobrist_hash ^=
                ZOBRIST_KEYS.piece_keys[opponent_side][piece as usize][capture_square];
        }
        self.en_passent = None;

        let from_mask = 1u64 << from;
        if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            self.board.move_pawn(
                from,
                to,
                self.is_white_turn,
                promo,
                previous_en_passant,
                &mut self.en_passent,
            );
        } else if (self.board.white_knight | self.board.black_knight) & from_mask != 0 {
            self.board.move_knight(from, to, self.is_white_turn);
        } else if (self.board.white_bishop | self.board.black_bishop) & from_mask != 0 {
            self.board.move_bishop(from, to, self.is_white_turn);
        } else if (self.board.white_rook | self.board.black_rook) & from_mask != 0 {
            self.board
                .move_rook(from, to, self.is_white_turn, &mut self.castling);
        } else if (self.board.white_queen | self.board.black_queen) & from_mask != 0 {
            self.board.move_queen(from, to, self.is_white_turn);
        } else if (self.board.white_king | self.board.black_king) & from_mask != 0 {
            self.board
                .move_king(from, to, self.is_white_turn, &mut self.castling);
        }

        // === Update castling rights on rook capture ===
        if let Some(Piece::Rook) = captured_piece {
            match to {
                0 => self.castling &= !0b0001,  // a8
                7 => self.castling &= !0b0010,  // h8
                56 => self.castling &= !0b0100, // a1
                63 => self.castling &= !0b1000, // h1
                _ => {}
            }
        }

        // === Zobrist Re-hash New State ===

        if let Some(ep_sq) = self.en_passent {
            let file = ep_sq % 8;
            self.zobrist_hash ^= ZOBRIST_KEYS.en_passent_keys[file];
        }

        self.zobrist_hash ^= ZOBRIST_KEYS.castling_keys[(self.castling & 0x0F) as usize];

        // XOR in piece at 'to'
        let piece_on_to = promo.unwrap_or(piece_moving);
        self.zobrist_hash ^= ZOBRIST_KEYS.piece_keys[side][piece_on_to as usize][to];

        // Flip side to move
        self.is_white_turn = !self.is_white_turn;
        self.zobrist_hash ^= ZOBRIST_KEYS.side_to_move_key;

        undo
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        self.is_white_turn = !self.is_white_turn;
        self.castling = undo.previous_castling_rights;
        self.en_passent = undo.previous_en_passant_square;
        let color_that_moved = self.is_white_turn;
        let from = undo.from;
        let to = undo.to;

        let moved_piece = undo.promotion.unwrap_or_else(|| {
            get_piece_at(self, to).expect("unmake_move: No piece on the 'to' square to unmake.")
        });

        self.remove_piece(to, moved_piece, color_that_moved);

        // Undo promo
        if undo.promotion.is_some() {
            self.add_piece(from, Piece::Pawn, color_that_moved);
        } else {
            self.add_piece(from, moved_piece, color_that_moved);
        }

        // Handle special moves
        if moved_piece == Piece::King && (from as i8 - to as i8).abs() == 2 {
            let (rook_from, rook_to) = match to {
                58 => (59, 56),
                62 => (61, 63),
                6 => (5, 7),
                2 => (3, 0),
                _ => unreachable!("A king move of 2 squares must be a castle."),
            };
            // Move the rook back from its post-castle square to its original corner.
            self.remove_piece(rook_from, Piece::Rook, color_that_moved);
            self.add_piece(rook_to, Piece::Rook, color_that_moved);
        }

        if let Some(captured_piece) = undo.captured_piece {
            let captured_color = !color_that_moved;

            // The captured piece is on the 'to' square, except if en_passent
            let moved_piece_was_pawn = get_piece_at(self, from) == Some(Piece::Pawn);
            let is_en_passant_capture =
                moved_piece_was_pawn && Some(to) == undo.previous_en_passant_square;

            let capture_square = if is_en_passant_capture {
                // The captured pawn in en passant is on a different rank
                if color_that_moved { to - 8 } else { to + 8 }
            } else {
                to
            };
            // Add the captured piece back to its square
            self.add_piece(capture_square, captured_piece, captured_color);
        }

        // Restore prev zobrist hash
        self.zobrist_hash = undo.previous_zobrist_hash;
    }
    fn get_piece_bb_mut(&mut self, piece: Piece, is_white: bool) -> &mut u64 {
        match (is_white, piece) {
            (true, Piece::Pawn) => &mut self.board.white_pawns,
            (true, Piece::Knight) => &mut self.board.white_knight,
            (true, Piece::Bishop) => &mut self.board.white_bishop,
            (true, Piece::Rook) => &mut self.board.white_rook,
            (true, Piece::Queen) => &mut self.board.white_queen,
            (true, Piece::King) => &mut self.board.white_king,
            (false, Piece::Pawn) => &mut self.board.black_pawns,
            (false, Piece::Knight) => &mut self.board.black_knight,
            (false, Piece::Bishop) => &mut self.board.black_bishop,
            (false, Piece::Rook) => &mut self.board.black_rook,
            (false, Piece::Queen) => &mut self.board.black_queen,
            (false, Piece::King) => &mut self.board.black_king,
        }
    }

    fn add_piece(&mut self, square: usize, piece: Piece, is_white: bool) {
        let mask = 1u64 << square;

        if let Some((existing_piece, existing_color)) = get_piece_colour_at(self, square) {
            let bb = self.board.get_mut_board(existing_piece, existing_color);
            *bb &= !mask;
        }

        let bb = self.board.get_mut_board(piece, is_white);
        *bb |= mask;
    }

    fn remove_piece(&mut self, square: usize, piece: Piece, is_white: bool) {
        let bitboard = self.get_piece_bb_mut(piece, is_white);
        *bitboard &= !(1u64 << square);
    }

    pub fn hash_position(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.board.all_pieces().hash(&mut hasher);
        self.is_white_turn.hash(&mut hasher);
        self.castling.hash(&mut hasher);
        self.en_passent.hash(&mut hasher);

        hasher.finish()
    }

    pub fn parse_move(input: &str) -> Option<(usize, usize, Option<Piece>)> {
        if input.len() != 4 && input.len() != 5 {
            return None;
        }

        let from_file = input.chars().next()?;
        let from_rank = input.chars().nth(1)?;
        let to_file = input.chars().nth(2)?;
        let to_rank = input.chars().nth(3)?;

        let from_file_idx = (from_file as u8).checked_sub(b'a')? as usize;
        let from_rank_idx = (from_rank as u8).checked_sub(b'1')? as usize;
        let to_file_idx = (to_file as u8).checked_sub(b'a')? as usize;
        let to_rank_idx = (to_rank as u8).checked_sub(b'1')? as usize;

        if from_file_idx > 7 || from_rank_idx > 7 || to_file_idx > 7 || to_rank_idx > 7 {
            return None;
        }

        let from_square = from_rank_idx * 8 + from_file_idx;
        let to_square = to_rank_idx * 8 + to_file_idx;

        let promotion = if input.len() == 5 {
            match input.chars().nth(4)? {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => None,
            }
        } else {
            None
        };

        Some((from_square, to_square, promotion))
    }

    // GAME TIME
    pub fn run(&mut self) {
        loop {
            print_board(&self.board);
            self.castling_check();
            match self.game_state() {
                GameState::Checkmate => {
                    println!(
                        "\n*** CHECKMATE! {} wins! ***",
                        if self.is_white_turn { "Black" } else { "White" }
                    );
                    break;
                }
                GameState::Stalemate => {
                    println!("\n*** STALEMATE! The game is a draw. ***");
                    break;
                }
                GameState::RepMoves => {
                    println!("\n*** 3 moves in a row! The game is a draw. ***");
                    break;
                }
                GameState::Check => {
                    println!("\n*** You are in CHECK! ***");
                }
                GameState::Normal => {}
            }

            let player = if self.is_white_turn { "White" } else { "Black" };
            print!("\n{player}> ");
            io::stdout().flush().unwrap(); // Ensure the prompt appears before input

            // read user input
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let trimmed_input = input.trim();
            if trimmed_input == "exit" {
                break;
            }

            match Self::parse_move(trimmed_input) {
                Some((from, to, promo)) => {
                    if self.make_move(from, to, promo) {
                        // If a move is possible, add it to the move history list
                        let position_hash = self.hash_position();

                        let count = self.position_history.entry(position_hash).or_insert(0);
                        *count += 1;

                        if *count == 3 {
                            println!("\n*** 3 moves in a row! The game is a draw. ***");
                            break;
                        }
                    } else {
                        println!("\n*** Illegal move! Try again. ***");
                    }
                }
                None => {
                    println!("\n*** Invalid format. Use algebraic notation (e.g., 'e2e4'). ***");
                }
            }
        }
    }
}
