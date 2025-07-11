use crate::bitboard::{Bitboard, Piece};
use crate::movelist::MoveList;
use crate::transposition_table::TranspositionTable;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{
    collections::HashMap,
    io::{self, Write},
};
use std::sync::{Arc, Mutex};

use crate::parser::parse_move;
#[derive(Clone)]
pub struct Game {
    pub board: Bitboard,
    pub is_white_turn: bool,
    pub castling: u8, // This will be represented with a 8 digit binary
    pub en_passent: Option<usize>,
    pub position_history: HashMap<u64, u32>, // Essentially, en_passent moves are pushed onto the vec and popped off after 1 turn
    pub tt: Arc<Mutex<TranspositionTable>>,
}
#[derive(Clone)]
pub struct Undo {
    board: Bitboard,
    castling: u8,
    en_passent: Option<usize>,
    is_white_turn: bool,
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
        // for castling :
        // 1U << 0 : black castling on queen side available
        // 1U << 1 : black castling on king side available
        // 1U << 2 : white castling on queen side available
        // 1U << 3 : white castling on king side available
        // 1U << 4 : black has not castled
        // 1U << 5 : white has not castled
        // 1U << 6 : black not in check
        // 1U << 7 : white not in check

        Self {
            board: Bitboard::new(),
            is_white_turn: true,
            castling: 0b11111111,
            en_passent: None,
            position_history: HashMap::new(),
            tt: Arc::new(Mutex::new(TranspositionTable::new())),
        }
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

        let from_mask = 1u64 << from;
        let en_passent_available = self.en_passent;
        self.en_passent = None;

        // If they put a user input and it's not for promo
        if promo.is_some() && (self.board.white_pawns | self.board.black_pawns) & from_mask == 0 {
            return false;
        }

        // Test is a move is sucessful or not
        let move_success = if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            self.board.move_pawn(
                from,
                to,
                self.is_white_turn,
                promo,
                en_passent_available,
                &mut self.en_passent,
            )
        } else if (self.board.white_knight | self.board.black_knight) & from_mask != 0 {
            self.board.move_knight(from, to, self.is_white_turn)
        } else if (self.board.white_bishop | self.board.black_bishop) & from_mask != 0 {
            self.board.move_bishop(from, to, self.is_white_turn)
        } else if (self.board.white_rook | self.board.black_rook) & from_mask != 0 {
            self.board
                .move_rook(from, to, self.is_white_turn, &mut self.castling)
        } else if (self.board.white_queen | self.board.black_queen) & from_mask != 0 {
            self.board.move_queen(from, to, self.is_white_turn)
        } else if (self.board.white_king | self.board.black_king) & from_mask != 0 {
            self.board
                .move_king(from, to, self.is_white_turn, &mut self.castling)
        } else {
            // No piece was on the 'from' square
            false
        };

        if !move_success {
            return false;
        }

        let king_board = if self.is_white_turn {
            self.board.white_king
        } else {
            self.board.black_king
        };

        let king_pos = king_board.trailing_zeros() as usize;

        // Check if the new move will put the king as check
        if self.board.possible_check(king_pos, !self.is_white_turn) {
            // Revert if so
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
        // Takes all moves, checks if the game can process them

        let mut legal_moves = MoveList::new();

        let pseudo_legal_moves = self.generate_pseudo_legal_moves();

        for &(from, to, promo) in pseudo_legal_moves.iter() {
            let mut temp_game = self.clone();
            if temp_game.make_move(from, to, promo) {
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

    /// Apply a move without legality check, returning the Undo data.
    pub fn make_move_unchecked(&mut self, from: usize, to: usize, promo: Option<Piece>) -> Undo {
        let undo = Undo {
            board: self.board.clone(),
            castling: self.castling,
            en_passent: self.en_passent,
            is_white_turn: self.is_white_turn,
        };

        let from_mask = 1u64 << from;
        self.en_passent = None;

        if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            self.board.move_pawn(
                from,
                to,
                self.is_white_turn,
                promo,
                undo.en_passent,
                &mut self.en_passent,
            );
        } else if (self.board.white_knight | self.board.black_knight) & from_mask != 0 {
            self.board.move_knight(from, to, self.is_white_turn);
        } else if (self.board.white_bishop | self.board.black_bishop) & from_mask != 0 {
            self.board.move_bishop(from, to, self.is_white_turn);
        } else if (self.board.white_rook | self.board.black_rook) & from_mask != 0 {
            self.board.move_rook(from, to, self.is_white_turn, &mut self.castling);
        } else if (self.board.white_queen | self.board.black_queen) & from_mask != 0 {
            self.board.move_queen(from, to, self.is_white_turn);
        } else if (self.board.white_king | self.board.black_king) & from_mask != 0 {
            self.board.move_king(from, to, self.is_white_turn, &mut self.castling);
        }

        self.is_white_turn = !self.is_white_turn;
        undo
    }

    /// Revert a move using undo info.
    pub fn unmake_move(&mut self, undo: Undo) {
        self.board = undo.board;
        self.castling = undo.castling;
        self.en_passent = undo.en_passent;
        self.is_white_turn = undo.is_white_turn;
    }

    // Used for 3 fold repitition
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
