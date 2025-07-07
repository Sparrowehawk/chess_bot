use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_pawn(&mut self, from: usize, to: usize, is_white: bool) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let (pawn_board, opponent_pieces) = if is_white {
            (self.white_pawns, self.black_pieces())
        } else {
            (self.black_pawns, self.white_pieces())
        };

        if (pawn_board & from_mask) == 0 {
            return false;
        }

        println!("{from}, {to}");

        if self.is_pawn_capture(from, to, is_white) {
            self.pawn_capture(from_mask, to_mask, opponent_pieces, is_white)
        } else {
            self.pawn_push(from, to, from_mask, to_mask, is_white)
        }
    }

    fn pawn_push(&mut self, from: usize, to: usize, from_mask: u64, to_mask: u64, is_white: bool) -> bool {
        if (self.all_pieces() & to_mask) != 0 { return false; }

        let single_valid_push = if is_white { from + 8 == to } else { to + 8 == from };
        if single_valid_push {
            self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
            return true;
        }

        let is_on_starting_rank = if is_white { from / 8 == 1 } else { from / 8 == 6 };
        let double_push_valid = if is_white { from + 16 == to } else { to + 16 == from };

        if is_on_starting_rank && double_push_valid {
            let intermediate_square = if is_white { from + 8 } else { from - 8 };
            if (self.all_pieces() & (1u64 << intermediate_square)) == 0 {
                self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
                return true;
            }
        }
        false
    }

    fn pawn_capture(&mut self, from_mask: u64, to_mask: u64, opponent_pieces: u64, is_white: bool) -> bool {
        if (opponent_pieces & to_mask) == 0 {
            return false;
        } else {
            self.clear_piece(to_mask, !is_white);
        }
        self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
        true
    }

    fn is_pawn_capture(&self, from: usize, to: usize, is_white: bool) -> bool {
        if is_white { from + 7 == to || from + 9 == to } else { to + 7 == from || to + 9 == from }
    }
}
