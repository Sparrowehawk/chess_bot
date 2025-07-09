use crate::{bitboard::{Bitboard, Piece}, Game};
use std::collections::HashMap; // Ensure HashMap is in scope if not already.

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

        // 1. Piece Placement
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
                if file > 7 { return Err("Invalid FEN: file out of bounds"); }
                let square_index = (rank * 8 + file) as usize;
                let piece = match ch {
                    'P' => { board.white_pawns |= 1 << square_index; }
                    'N' => { board.white_knight |= 1 << square_index; }
                    'B' => { board.white_bishop |= 1 << square_index; }
                    'R' => { board.white_rook |= 1 << square_index; }
                    'Q' => { board.white_queen |= 1 << square_index; }
                    'K' => { board.white_king |= 1 << square_index; }
                    'p' => { board.black_pawns |= 1 << square_index; }
                    'n' => { board.black_knight |= 1 << square_index; }
                    'b' => { board.black_bishop |= 1 << square_index; }
                    'r' => { board.black_rook |= 1 << square_index; }
                    'q' => { board.black_queen |= 1 << square_index; }
                    'k' => { board.black_king |= 1 << square_index; }
                    _ => return Err("Invalid character in FEN piece placement."),
                };
                file += 1;
            }
        }

        // 2. Active Color
        let is_white_turn = match parts[1] {
            "w" => true,
            "b" => false,
            _ => return Err("Invalid active color in FEN."),
        };

        // 3. Castling Availability
        let mut castling = 0u8;
        let castling_rights = parts[2];
        if castling_rights.contains('K') { castling |= 1 << 3; }
        if castling_rights.contains('Q') { castling |= 1 << 2; }
        if castling_rights.contains('k') { castling |= 1 << 1; }
        if castling_rights.contains('q') { castling |= 1 << 0; }

        // 4. En Passant Target Square
        let en_passant_str = parts[3];
        let en_passent = if en_passant_str == "-" {
            None
        } else {
            let file = en_passant_str.chars().nth(0).ok_or("Invalid en passant square")? as u8 - b'a';
            let rank = en_passant_str.chars().nth(1).ok_or("Invalid en passant square")? as u8 - b'1';
            if file > 7 || rank > 7 { return Err("Invalid en passant square"); }
            Some((rank * 8 + file) as usize)
        };
        
        // 5. & 6. Halfmove and Fullmove clocks (ignored for perft)

        Ok(Game {
            board,
            is_white_turn,
            castling,
            en_passent,
            position_history: HashMap::new(),
        })
    }
}