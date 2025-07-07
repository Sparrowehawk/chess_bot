use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_bishop(&mut self, from: usize, to: usize, is_white: bool) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let (bishop_board, opponent_pieces, friendly_pieces) = if is_white {
            (self.white_bishop, self.black_pieces(), self.white_pieces())
        } else {
            (self.black_bishop, self.white_pieces(), self.black_pieces())
        };

        if (bishop_board & from_mask) == 0 || (friendly_pieces & to_mask) != 0 {
            return false;
        }

        let from_file = from % 8;
        let to_file = to % 8;

        let file_distance = (from_file as i8 - to_file as i8).abs();
        let rank_distance = ( (from / 8) as i8 - (to / 8) as i8 ).abs();

        if file_distance != rank_distance {
            return false; 
        }

        let step = if (to / 8) > (from / 8) { // Moving up the board
            if to_file > from_file { 9 } else { 7 } // Up-right or Up-left
        } else { // Moving down the board
            if to_file > from_file { -7 } else { -9 } // Down-right or Down-left
        };

        // Check for obstructions along the path
        let mut current_pos = (from as isize) + step;
        while current_pos != to as isize {
            if (self.all_pieces() & (1u64 << current_pos)) != 0 {
                return false; // Path is blocked
            }
            current_pos += step;
        }


        if (opponent_pieces & to_mask) == 0 {
            self.bishop_move(from_mask, to_mask, is_white)
        } else {
            self.bishop_capture(from_mask, to_mask, opponent_pieces, is_white)
        }
    }

    fn bishop_move(
        &mut self,
        from_mask: u64,
        to_mask: u64,
        is_white: bool,
    ) -> bool {
        if (self.all_pieces() & to_mask) != 0 {
            return false;
        }
        self.apply_move(from_mask, to_mask, Piece::Bishop, is_white);
        true
    }

    fn bishop_capture(
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
        self.apply_move(from_mask, to_mask, Piece::Bishop, is_white);
        true
    }
}