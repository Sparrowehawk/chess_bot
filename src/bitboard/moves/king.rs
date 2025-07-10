use crate::bitboard::{Bitboard, Piece};

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
        // castling check if distance is 2
        let is_castle_move = ((from as i8) - (to as i8)).abs() == 2;
        if is_castle_move {
            // White kingside
            if is_white && to == 6 && (*castling & (1 << 3)) != 0 {
                // Rook must be on h1, and squares f1, g1 must be empty.
                if (self.white_rook & (1 << 7)) == 0 || (self.all_pieces() & 0x60) != 0 {
                    return false;
                }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 7, 1u64 << 5, Piece::Rook, is_white);
            }
            // White queenside 
            else if is_white && to == 2 && (*castling & (1 << 2)) != 0 {
                // Rook must be on a1, and squares b1, c1, d1 must be empty.
                if (self.white_rook & (1 << 0)) == 0 || (self.all_pieces() & 0xE) != 0 {
                    return false;
                }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 0, 1u64 << 3, Piece::Rook, is_white);
            }
            // Black kingside 
            else if !is_white && to == 62 && (*castling & (1 << 1)) != 0 {
                // Rook must be on h8, and squares f8, g8 must be empty.
                if (self.black_rook & (1 << 63)) == 0
                    || (self.all_pieces() & 0x6000000000000000) != 0
                {
                    return false;
                }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 63, 1u64 << 61, Piece::Rook, is_white);
            }
            // Black queenside 
            else if !is_white && to == 58 && (*castling & (1 << 0)) != 0 {
                // Rook must be on a8, and squares b8, c8, d8 must be empty.
                if (self.black_rook & (1 << 56)) == 0
                    || (self.all_pieces() & 0xE00000000000000) != 0
                {
                    return false;
                }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 56, 1u64 << 59, Piece::Rook, is_white);
            } else {
                // Fallback
                return false;
            }
        }
        // Normal moves
        else {
            let from_file = from % 8;
            let to_file = to % 8;
            let file_distance = (from_file as i8 - to_file as i8).abs();
            let rank_distance = ((from / 8) as i8 - (to / 8) as i8).abs();

            // 1 square distance
            if file_distance > 1 || rank_distance > 1 {
                return false;
            }
            // Remove the opps
            if (opponent_pieces & to_mask) != 0 {
                self.clear_piece(to_mask, !is_white);
            }
            self.apply_move(from_mask, to_mask, Piece::King, is_white);
        }

        // After any move, strip castling rights
        if is_white {
            *castling &= !(1 << 3 | 1 << 2);
        } else {
            *castling &= !(1 << 1 | 1 << 0);
        }

        true
    }
}
