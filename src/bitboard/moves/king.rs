use crate::bitboard::{Bitboard, Piece};

const WHITE_KINGSIDE: u8 = 1 << 3;
const WHITE_QUEENSIDE: u8 = 1 << 2;
const BLACK_KINGSIDE: u8 = 1 << 1;
const BLACK_QUEENSIDE: u8 = 1 << 0;

impl Bitboard {
    pub fn move_king(&mut self, from: usize, to: usize, is_white: bool, castling: &mut u8) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let (king_board, opponent_pieces, friendly_pieces) = if is_white {
            (self.white_king, self.black_pieces(), self.white_pieces())
        } else {
            (self.black_king, self.white_pieces(), self.black_pieces())
        };

        if (king_board & from_mask) == 0 || (friendly_pieces & to_mask) != 0 {
            return false;
        }
        // --- Handle Castling ---
        let is_castle_move = ((from as i8) - (to as i8)).abs() == 2;
        if is_castle_move {
            // White Kingside Castle (O-O)
            if is_white && to == 6 && (*castling & WHITE_KINGSIDE) != 0 {
                // Rook must be on h1, and squares f1, g1 must be empty.
                if (self.white_rook & (1 << 7)) == 0 || (self.all_pieces() & 0x60) != 0 { return false; }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 7, 1u64 << 5, Piece::Rook, is_white);
            }
            // White Queenside Castle (O-O-O)
            else if is_white && to == 2 && (*castling & WHITE_QUEENSIDE) != 0 {
                // Rook must be on a1, and squares b1, c1, d1 must be empty.
                if (self.white_rook & (1 << 0)) == 0 || (self.all_pieces() & 0xE) != 0 { return false; }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 0, 1u64 << 3, Piece::Rook, is_white);
            }
            // Black Kingside Castle (o-o)
            else if !is_white && to == 62 && (*castling & BLACK_KINGSIDE) != 0 {
                // Rook must be on h8, and squares f8, g8 must be empty.
                if (self.black_rook & (1 << 63)) == 0 || (self.all_pieces() & 0x6000000000000000) != 0 { return false; }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 63, 1u64 << 61, Piece::Rook, is_white);
            }
            // Black Queenside Castle (o-o-o)
            else if !is_white && to == 58 && (*castling & BLACK_QUEENSIDE) != 0 {
                 // Rook must be on a8, and squares b8, c8, d8 must be empty.
                if (self.black_rook & (1 << 56)) == 0 || (self.all_pieces() & 0xE00000000000000) != 0 { return false; }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 56, 1u64 << 59, Piece::Rook, is_white);
            }
            else {
                return false; // It looked like a castle, but was not legal or possible.
            }
        }
        // --- Handle Standard Moves ---
        else {
            let from_file = from % 8;
            let to_file = to % 8;
            let file_distance = (from_file as i8 - to_file as i8).abs();
            let rank_distance = ((from / 8) as i8 - (to / 8) as i8).abs();

            // A standard king move must be within a 1-square box.
            if file_distance > 1 || rank_distance > 1 {
                return false;
            }
            // Handle captures
            if (opponent_pieces & to_mask) != 0 {
                self.clear_piece(to_mask, !is_white);
            }
            self.apply_move(from_mask, to_mask, Piece::King, is_white);
        }

        // After ANY king move (castling or standard), remove all castling rights for that color.
        if is_white {
            *castling &= !(WHITE_KINGSIDE | WHITE_QUEENSIDE);
        } else {
            *castling &= !(BLACK_KINGSIDE | BLACK_QUEENSIDE);
        }

        true
    }
}
