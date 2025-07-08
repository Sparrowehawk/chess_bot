use crate::bitboard::{Bitboard, Piece};
use std::io::{self, Write};

use crate::parser::parse_move;
pub struct Game {
    board: Bitboard,
    pub is_white_turn: bool,
    pub castling_option: u8, // This will be represented with a 4 digit binary
    pub en_passent: Option<usize>, // Essentially, en_passent moves are pushed onto the vec and popped off after 1 turn
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Bitboard::new(),
            is_white_turn: true,
            castling_option: 0b1111,
            en_passent: None,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    fn make_move(&mut self, from: usize, to: usize) -> bool {
        let from_mask = 1u64 << from;
        let en_passent_move_avalible = self.en_passent;
        self.en_passent = None;

        if (self.board.white_pawns | self.board.black_pawns) & from_mask != 0 {
            self.board.move_pawn(
                from,
                to,
                self.is_white_turn,
                en_passent_move_avalible,
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
            self.board.move_king(from, to, self.is_white_turn)
        } else {
            // No piece was on the 'from' square
            false
        }
    }

    pub fn run(&mut self) {
        loop {
            self.board.print_board();

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
