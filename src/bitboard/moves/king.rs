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

        // Check for castling here
        if is_white {
            if from == 4 && to == 6 && ((*castling & (1 << 3)) != 0) {
                // Castle king side
                if (self.white_rook & (1 << 7) == 0)
                    || (self.all_pieces() & (1 << 5) != 0)
                    || (self.all_pieces() & (1 << 6) != 0)
                {
                    return false;
                }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 7, 1u64 << 5, Piece::Rook, is_white);
                *castling &= !(1 << 5);
                return true;
            } else if from == 4 && to == 2 && ((*castling & (1 << 2)) != 0) {
                // Castle queen side
                if (self.white_rook & (1 << 0) == 0)
                    || (self.all_pieces() & (1 << 1) != 0)
                    || (self.all_pieces() & (1 << 2) != 0)
                    || (self.all_pieces() & (1 << 3) != 0)
                {
                    return false;
                }
                self.apply_move(from_mask, to_mask, Piece::King, is_white);
                self.apply_move(1u64 << 0, 1u64 << 3, Piece::Rook, is_white);
                *castling &= !(1 << 4);
                return true;
            }
        } else if from == 60 && to == 62 && ((*castling & (1 << 1)) != 0) {
            // Black castle king side
            if (self.black_rook & (1 << 63) == 0)
                || (self.all_pieces() & (1 << 61) != 0)
                || (self.all_pieces() & (1 << 62) != 0)
            {
                return false;
            }
            self.apply_move(from_mask, to_mask, Piece::King, is_white);
            self.apply_move(1u64 << 63, 1u64 << 61, Piece::Rook, is_white);
            *castling &= !(1 << 4);
            return true;
        } else if from == 60 && to == 58 && ((*castling & (1 << 2)) != 0) {
            // Black castle queen side
            if (self.black_rook & (1 << 56) == 0)
                || (self.all_pieces() & (1 << 57) != 0)
                || (self.all_pieces() & (1 << 58) != 0)
                || (self.all_pieces() & (1 << 59) != 0)
            {
                return false;
            }
            self.apply_move(from_mask, to_mask, Piece::King, is_white);
            self.apply_move(1u64 << 56, 1u64 << 59, Piece::Rook, is_white);
            *castling &= !(1 << 4);
            return true;
        }

        // Essentially, the king can move +/- 1, 7, 8, 9
        let from_file = from % 8;
        let to_file = to % 8;

        let file_distance = (from_file as i8 - to_file as i8).abs();
        let rank_distance = ((from / 8) as i8 - (to / 8) as i8).abs();

        if file_distance > 1 || rank_distance > 1 {
            return false;
        }
        if (opponent_pieces & to_mask) != 0 {
            self.clear_piece(to_mask, !is_white);
        }
        self.apply_move(from_mask, to_mask, Piece::King, is_white);

        // Clear both king-side and queen-side castling rights for the moving side
        *castling &= if is_white { !(0b1100) } else { !(0b0011) };
        true
    }
}
