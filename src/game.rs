use crate::bitboard::Bitboard;
use std::io::{self, Write};

use crate::parser::parse_move;
#[derive(Clone)]
pub struct Game {
    board: Bitboard,
    pub is_white_turn: bool,
    pub castling: u8,              // This will be represented with a 4 digit binary
    pub en_passent: Option<usize>, // Essentially, en_passent moves are pushed onto the vec and popped off after 1 turn
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    Normal,
    Check,
    Checkmate,
    Stalemate,
}

impl Default for Game {
    fn default() -> Self {
        // for castling :
        // 1U << 0 : black castling on queen side
        // 1U << 1 : black castling on king side
        // 1U << 2 : white castling on queen side
        // 1U << 3 : white castling on king side
        Self {
            board: Bitboard::new(),
            is_white_turn: true,
            castling: 0b1111,
            en_passent: None,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn make_move(&mut self, from: usize, to: usize) -> bool {
        let original_board = self.board.clone();
        let original_castling = self.castling;
        let original_en_passent = self.en_passent;

        let from_mask = 1u64 << from;
        let en_passent_available = self.en_passent;
        self.en_passent = None;

        let move_success = if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            self.board.move_pawn(
                from,
                to,
                self.is_white_turn,
                en_passent_available,
                &mut self.en_passent,
            )
        } else if (self.board.white_knight | self.board.black_knight) & from_mask != 0 {
            self.board.move_knight(from, to, self.is_white_turn)
        } else if (self.board.white_bishop | self.board.black_bishop) & from_mask != 0 {
            self.board.move_bishop(from, to, self.is_white_turn)
        } else if (self.board.white_rook | self.board.black_rook) & from_mask != 0 {
            self.board.move_rook(from, to, self.is_white_turn)
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

        if self.board.possible_check(king_pos, !self.is_white_turn) {
            self.board = original_board;
            self.castling = original_castling;
            self.en_passent = original_en_passent;
            return false;
        }


        true
    }

    pub fn is_in_check(&self) -> bool {
        let king_board = if self.is_white_turn {
            self.board.white_king
        } else {
            self.board.black_king
        };

        let king_pos = king_board.trailing_zeros() as usize;
        self.board.possible_check(king_pos, !self.is_white_turn)
    }

    fn generate_legal_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();

        let all_pieces = if self.is_white_turn {
            self.board.white_pieces()
        } else {
            self.board.black_pieces()
        };

        for from in 0..64 {
            if (all_pieces & (1u64 << from)) != 0 {
                for to in 0..64 {
                    if from == to {
                        continue;
                    }

                    let mut temp_game = self.clone();
                    if temp_game.make_move(from, to) {
                        moves.push((from, to));
                    }
                }
            }
        }

        moves
    }

    pub fn game_state(&self) -> GameState {
        let is_in_check = self.is_in_check();
        let has_legal_moves = !self.generate_legal_moves().is_empty();

        if is_in_check {
            if has_legal_moves {
                GameState::Check
            } else {
                GameState::Checkmate
            }
        } else if has_legal_moves {
            GameState::Normal
        } else {
            GameState::Stalemate
        }
    }

    pub fn run(&mut self) {
        loop {
            self.board.print_board();

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
                GameState::Check => {
                    println!("\n*** You are in CHECK! ***");
                }
                GameState::Normal => {}
            }

            let player = if self.is_white_turn { "White" } else { "Black" };
            print!("\n{player}> ");
            io::stdout().flush().unwrap(); // Ensure the prompt appears before input

            // 3. Read user input
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            // 4. Parse the input
            let trimmed_input = input.trim();
            if trimmed_input == "exit" {
                break;
            }

            match parse_move(trimmed_input) {
                Some((from, to, _promo)) => {
                    if self.make_move(from, to) {
                        self.is_white_turn = !self.is_white_turn;
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
