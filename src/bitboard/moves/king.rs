use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_king(&mut self, from: usize, to: usize, is_white: bool) -> bool {
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

        // Essentially, the king can move +/- 1, 7, 8, 9
        let from_file = from % 8;
        let to_file = to % 8;

        let file_distance = (from_file as i8 - to_file as i8).abs();
        let rank_distance = ( (from / 8) as i8 - (to / 8) as i8 ).abs();

        if file_distance > 1 || rank_distance > 1 {
            return false; 
        }
        if (opponent_pieces & to_mask) == 0 {
            self.king_move(from_mask, to_mask, is_white)
        } else {
            self.king_capture(from_mask, to_mask, opponent_pieces, is_white)
        }
    }

    fn king_move(
        &mut self,
        from_mask: u64,
        to_mask: u64,
        is_white: bool,
    ) -> bool {
        if (self.all_pieces() & to_mask) != 0 {
            return false;
        }

        self.apply_move(from_mask, to_mask, Piece::King, is_white);
        true
    }

    fn king_capture(
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
        self.apply_move(from_mask, to_mask, Piece::King, is_white);
        true
    }
}
