use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_rook(&mut self, from: usize, to: usize, is_white: bool, castling: &mut u8) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let (rook_board, opponent_pieces, friendly_pieces) = if is_white {
            (self.white_rook, self.black_pieces(), self.white_pieces())
        } else {
            (self.black_rook, self.white_pieces(), self.black_pieces())
        };

        if (rook_board & from_mask) == 0 {
            return false;
        }

        if (friendly_pieces & to_mask) != 0 {
            return false;
        }

        if (from / 8) == (to / 8) {
            let start = from % 8;
            let end = to % 8;

            let range = if start < end {
                (start + 1) .. end
            } else {
                (end + 1) .. start
            };

            for intermediate in range {
                let intermediate_square = (from/8) * 8 + intermediate;
                if self.all_pieces() & (1u64 << intermediate_square) != 0 {
                    return false;
                }
            }
        } else if (from % 8) == (to % 8) {

            let start = from / 8;
            let end = to / 8;

            let range = if start < end {
                (start + 1) .. end
            } else {
                (end + 1) .. start
            };

            for intermediate in range {
                let intermediate_square = (from%8) + 8 * intermediate;
                if self.all_pieces() & (1u64 << intermediate_square) != 0 {
                    return false;
                }
            }

        } else {
            return false;
        }

        match from {
            0 => *castling &= !(1 << 2),
            7 => *castling &= !(1 << 3),
            56 => *castling &= !(1 << 0),
            63 => *castling &= !(1 << 1),
            _ => (),
        }

        if (opponent_pieces & to_mask) == 0 {
            self.rook_push(from_mask, to_mask, is_white)
        } else {
            self.rook_capture(from_mask, to_mask, opponent_pieces, is_white)
        }
    }

    fn rook_push(&mut self, from_mask: u64, to_mask: u64, is_white: bool) -> bool {
        if (self.all_pieces() & to_mask) != 0 {
            return false;
        }
        self.apply_move(from_mask, to_mask, Piece::Rook, is_white);
        true
    }

    fn rook_capture(
        &mut self,
        from_mask: u64,
        to_mask: u64,
        opponent_pieces: u64,
        is_white: bool,
    ) -> bool {
        if (opponent_pieces & to_mask) == 0 {
            return false;
        } else {
            self.clear_piece(to_mask, !is_white);
        }
        self.apply_move(from_mask, to_mask, Piece::Rook, is_white);
        true
    }
}
