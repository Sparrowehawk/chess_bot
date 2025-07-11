use crate::{Game, bitboard::Bitboard, transposition_table::TranspositionTable};
use std::cell::RefCell;
use std::collections::HashMap; // Ensure HashMap is in scope if not already.
use std::rc::Rc;
use std::sync::{Arc, Mutex};

impl Game {
    /// Creates a new Game instance from a FEN string.
    /// This is essential for setting up specific test positions.
    ///
    /// # Arguments
    ///
    /// * `fen` - A string slice that holds the FEN notation.
    ///
    /// # Returns
    ///
    /// * `Result<Self, &'static str>` - A new Game instance or an error message if the FEN is invalid.
    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let mut board = Bitboard::empty(); // Start with an empty board
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Invalid FEN string: not enough parts.");
        }

        // Pieces on le board
        let piece_placement = parts[0];
        let mut rank = 7;
        let mut file = 0;
        for ch in piece_placement.chars() {
            if ch == '/' {
                rank -= 1;
                file = 0;
            } else if let Some(digit) = ch.to_digit(10) {
                file += digit as i32;
            } else {
                if file > 7 {
                    return Err("Invalid FEN: file out of bounds");
                }
                let square_index = (rank * 8 + file) as usize;
                match ch {
                    'P' => {
                        board.white_pawns |= 1 << square_index;
                    }
                    'N' => {
                        board.white_knight |= 1 << square_index;
                    }
                    'B' => {
                        board.white_bishop |= 1 << square_index;
                    }
                    'R' => {
                        board.white_rook |= 1 << square_index;
                    }
                    'Q' => {
                        board.white_queen |= 1 << square_index;
                    }
                    'K' => {
                        board.white_king |= 1 << square_index;
                    }
                    'p' => {
                        board.black_pawns |= 1 << square_index;
                    }
                    'n' => {
                        board.black_knight |= 1 << square_index;
                    }
                    'b' => {
                        board.black_bishop |= 1 << square_index;
                    }
                    'r' => {
                        board.black_rook |= 1 << square_index;
                    }
                    'q' => {
                        board.black_queen |= 1 << square_index;
                    }
                    'k' => {
                        board.black_king |= 1 << square_index;
                    }
                    _ => return Err("Invalid character in FEN piece placement."),
                };
                file += 1;
            }
        }

        // Colour time
        let is_white_turn = match parts[1] {
            "w" => true,
            "b" => false,
            _ => return Err("Invalid active colour in FEN."),
        };

        // Parse castling rights
        let mut castling = 0u8;
        let castling_rights = parts[2];
        if castling_rights.contains('K') {
            castling |= 1 << 3;
        }
        if castling_rights.contains('Q') {
            castling |= 1 << 2;
        }
        if castling_rights.contains('k') {
            castling |= 1 << 1;
        }
        if castling_rights.contains('q') {
            castling |= 1 << 0;
        }

        // if en passent is available
        let en_passent_str = parts[3];
        let en_passent = if en_passent_str == "-" {
            None
        } else {
            let file = en_passent_str
                .chars()
                .nth(0)
                .ok_or("Invalid en passant square")? as u8
                - b'a';
            let rank = en_passent_str
                .chars()
                .nth(1)
                .ok_or("Invalid en passant square")? as u8
                - b'1';
            if file > 7 || rank > 7 {
                return Err("Invalid en passant square");
            }
            Some((rank * 8 + file) as usize)
        };

        // Ignore halfmove and full_move clos

        Ok(Game {
            board,
            is_white_turn,
            castling,
            en_passent,
            position_history: HashMap::new(),
            tt: Arc::new(Mutex::new(TranspositionTable::new())),
        })
    }
}
