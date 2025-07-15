use crate::bitboard::{Bitboard, Piece};
use crate::movelist::MoveList;
use crate::transposition_table::TranspositionTable;
use crate::zobrist::ZobristKeys;
use array_init;
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    io::{self, Write},
};

pub static ZOBRIST_KEYS: Lazy<ZobristKeys> = Lazy::new(ZobristKeys::new);

use crate::parser::parse_move;
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
        game.zobrist_hash = game.compute_zobrist_hash();
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

        
        if from == 6 && to == 21{
        if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            println!("1");
        } else if (self.board.white_knight | self.board.black_knight) & from_mask != 0 {
            println!("2");
        } else if (self.board.white_bishop | self.board.black_bishop) & from_mask != 0 {
            println!("3");
        } else if (self.board.white_rook | self.board.black_rook) & from_mask != 0 {
            println!("4");
        } else if (self.board.white_queen | self.board.black_queen) & from_mask != 0 {
            println!("5");
        } else if (self.board.white_king | self.board.black_king) & from_mask != 0 {
            println!("6");
        };}

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
        if self.castling & (1 << 4) != 0 {
            let mut temp_game = self.clone();
            temp_game.is_white_turn = false;
            if temp_game.make_move(60, 59, None) && temp_game.make_move(60, 58, None) {
                self.castling |= 1 << 0;
            }

            let mut temp_game = self.clone();
            temp_game.is_white_turn = false;
            if temp_game.make_move(60, 61, None) && temp_game.make_move(60, 62, None) {
                self.castling |= 1 << 1;
            }
        } else {
            self.castling &= !(1 << 0);
            self.castling &= !(1 << 1);
        }
        if self.castling & (1 << 5) != 0 {
            let mut temp_game = self.clone();
            temp_game.is_white_turn = true;
            if temp_game.make_move(4, 2, None) && temp_game.make_move(4, 3, None) {
                self.castling |= 1 << 2;
            }
            let mut temp_game = self.clone();
            temp_game.is_white_turn = true;
            if temp_game.make_move(4, 5, None) && temp_game.make_move(4, 6, None) {
                self.castling |= 1 << 3;
            }
        } else {
            self.castling &= !(1 << 2);
            self.castling &= !(1 << 3);
        }
    }

    pub fn generate_legal_moves(&self) -> MoveList {
        let mut legal_moves = MoveList::new();
        let pseudo_legal_moves = self.generate_pseudo_legal_moves();

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

    fn get_piece_and_color(&self, square: usize) -> (Option<Piece>, Option<bool>) {
        let mask = 1u64 << square;
        if (self.board.white_pawns & mask) != 0 {
            return (Some(Piece::Pawn), Some(true));
        }
        if (self.board.black_pawns & mask) != 0 {
            return (Some(Piece::Pawn), Some(false));
        }
        if (self.board.white_knight & mask) != 0 {
            return (Some(Piece::Knight), Some(true));
        }
        if (self.board.black_knight & mask) != 0 {
            return (Some(Piece::Knight), Some(false));
        }
        if (self.board.white_bishop & mask) != 0 {
            return (Some(Piece::Bishop), Some(true));
        }
        if (self.board.black_bishop & mask) != 0 {
            return (Some(Piece::Bishop), Some(false));
        }
        if (self.board.white_rook & mask) != 0 {
            return (Some(Piece::Rook), Some(true));
        }
        if (self.board.black_rook & mask) != 0 {
            return (Some(Piece::Rook), Some(false));
        }
        if (self.board.white_queen & mask) != 0 {
            return (Some(Piece::Queen), Some(true));
        }
        if (self.board.black_queen & mask) != 0 {
            return (Some(Piece::Queen), Some(false));
        }
        if (self.board.white_king & mask) != 0 {
            return (Some(Piece::King), Some(true));
        }
        if (self.board.black_king & mask) != 0 {
            return (Some(Piece::King), Some(false));
        }
        (None, None)
    }

    pub fn make_move_unchecked(&mut self, from: usize, to: usize, promo: Option<Piece>) -> Undo {
        let piece_moving = self.get_piece_at(from).expect("No piece on 'from' square");
        let previous_en_passant = self.en_passent;

        let mut captured_piece = self.get_piece_at(to);
        let is_en_passant_capture = piece_moving == Piece::Pawn && Some(to) == previous_en_passant;
        if is_en_passant_capture {
            captured_piece = Some(Piece::Pawn);
        }

        let undo = Undo {
            from,
            to,
            captured_piece,
            promotion: promo,
            previous_castling_rights: self.castling,
            previous_en_passant_square: self.en_passent,
            previous_zobrist_hash: self.zobrist_hash,
        };

        let mut new_hash = self.zobrist_hash;
        let color_idx = if self.is_white_turn { 0 } else { 1 };

        new_hash ^= ZOBRIST_KEYS.piece_keys[color_idx][piece_moving as usize][from];

        if let Some(captured) = captured_piece {
            let captured_color_idx = 1 - color_idx;
            let capture_square = if is_en_passant_capture {
                if self.is_white_turn { to - 8 } else { to + 8 }
            } else {
                to
            };
            new_hash ^=
                ZOBRIST_KEYS.piece_keys[captured_color_idx][captured as usize][capture_square];
        }

        let final_piece = promo.unwrap_or(piece_moving);
        new_hash ^= ZOBRIST_KEYS.piece_keys[color_idx][final_piece as usize][to];

        new_hash ^= ZOBRIST_KEYS.castling_keys[(self.castling & 0xF) as usize]; // XOR out old rights


        if let Some(ep_sq) = self.en_passent {
            new_hash ^= ZOBRIST_KEYS.en_passent_keys[ep_sq % 8];
        }

        new_hash ^= ZOBRIST_KEYS.side_to_move_key;

        self.en_passent = None; 
        if piece_moving == Piece::Pawn {
            self.board.move_pawn(
                from,
                to,
                self.is_white_turn,
                promo,
                previous_en_passant,
                &mut self.en_passent,
            );
        } else if piece_moving == Piece::Knight {
            self.board.move_knight(from, to, self.is_white_turn);
        } else if piece_moving == Piece::Bishop {
            self.board.move_bishop(from, to, self.is_white_turn);
        } else if piece_moving == Piece::Rook {
            self.board
                .move_rook(from, to, self.is_white_turn, &mut self.castling);
        } else if piece_moving == Piece::Queen {
            self.board.move_queen(from, to, self.is_white_turn);
        } else if piece_moving == Piece::King {
            self.board
                .move_king(from, to, self.is_white_turn, &mut self.castling);
        }

        new_hash ^= ZOBRIST_KEYS.castling_keys[(self.castling & 0xF) as usize]; 
        if let Some(ep_sq) = self.en_passent {
            new_hash ^= ZOBRIST_KEYS.en_passent_keys[ep_sq % 8]; 
        }

        self.is_white_turn = !self.is_white_turn;
        self.zobrist_hash = new_hash;

        undo
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        self.is_white_turn = !self.is_white_turn;
        let color_that_moved = self.is_white_turn;
        self.castling = undo.previous_castling_rights;
        self.en_passent = undo.previous_en_passant_square;

        let from = undo.from;
        let to = undo.to;

        if let Some(promoted_piece) = undo.promotion {
            self.remove_piece(to, promoted_piece, color_that_moved);
            self.add_piece(from, Piece::Pawn, color_that_moved);
        } else {
            let (moved_piece_opt, moved_color_opt) = self.get_piece_and_color(to);
            if let (Some(moved_piece), Some(moved_color)) = (moved_piece_opt, moved_color_opt) {
                self.move_piece(from, to, moved_piece, moved_color);
            } else {
                // self.board.print_board();
                println!("{:?}, {:?}, {from}, {to}", moved_piece_opt, moved_color_opt);
                return;
                // panic!("unmake_move: piece to move back not found at 'to' square {to}");
            }
        }

        if let Some(captured_piece) = undo.captured_piece {
            let captured_color = !color_that_moved;

            let moved_piece_was_pawn = self.get_piece_at(from) == Some(Piece::Pawn);
            let is_en_passant_capture =
                moved_piece_was_pawn && Some(to) == undo.previous_en_passant_square;

            let capture_square = if is_en_passant_capture {
                if color_that_moved { to - 8 } else { to + 8 }
            } else {
                to
            };
            self.add_piece(capture_square, captured_piece, captured_color);
        }
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

        let mut all_bitboards = [
            &mut self.board.white_pawns,
            &mut self.board.white_knight,
            &mut self.board.white_bishop,
            &mut self.board.white_rook,
            &mut self.board.white_queen,
            &mut self.board.white_king,
            &mut self.board.black_pawns,
            &mut self.board.black_knight,
            &mut self.board.black_bishop,
            &mut self.board.black_rook,
            &mut self.board.black_queen,
            &mut self.board.black_king,
        ];

        for bb in all_bitboards.iter_mut() {
            **bb &= !mask;
        }

        let bitboard = self.board.get_mut_board(piece, is_white);
        *bitboard |= mask;
    }

    fn remove_piece(&mut self, square: usize, piece: Piece, is_white: bool) {
        let bitboard = self.get_piece_bb_mut(piece, is_white);
        *bitboard &= !(1u64 << square);
    }

    fn move_piece(&mut self, from: usize, to: usize, piece: Piece, is_white: bool) {
        let bitboard = self.get_piece_bb_mut(piece, is_white);
        let move_mask = (1u64 << from) | (1u64 << to);
        *bitboard ^= move_mask;
    }

    pub fn hash_position(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.board.all_pieces().hash(&mut hasher);
        self.is_white_turn.hash(&mut hasher);
        self.castling.hash(&mut hasher);
        self.en_passent.hash(&mut hasher);

        hasher.finish()
    }

    // GAME TIME
    pub fn run(&mut self) {
        loop {
            self.board.print_board();
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

            match parse_move(trimmed_input) {
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
