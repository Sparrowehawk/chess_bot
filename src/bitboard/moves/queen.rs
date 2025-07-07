use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_queen(&mut self, from: usize, to: usize, is_white: bool) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let (queen_board, opponent_pieces, friendly_pieces) = if is_white {
            (self.white_queen, self.black_pieces(), self.white_pieces())
        } else {
            (self.black_queen, self.white_pieces(), self.black_pieces())
        };

        if (queen_board & from_mask) == 0 || (friendly_pieces & to_mask) != 0 {
            return false;
        }

        let from_file = from % 8;
        let to_file = to % 8;

        let file_distance = (from_file as i8 - to_file as i8).abs();
        let rank_distance = ((from / 8) as i8 - (to / 8) as i8).abs();

        // To calculate if there's anything in between
        if file_distance == rank_distance {
            let step = if (to / 8) > (from / 8) {
                // Moving up the board
                if to_file > from_file { 9 } else { 7 } // Up-right or Up-left
            } else {
                // Moving down the board
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
        } else if (from / 8) == (to / 8) {
            // move like a rook
            let start = from % 8;
            let end = to % 8;

            let range = if start < end {
                (start + 1)..end
            } else {
                (end + 1)..start
            };

            for intermediate in range {
                let intermediate_square = (from / 8) * 8 + intermediate;
                if self.all_pieces() & (1u64 << intermediate_square) != 0 {
                    return false;
                }
            }
        } else if (from % 8) == (to % 8) {
            // move like a rook
            let start = from / 8;
            let end = to / 8;

            let range = if start < end {
                (start + 1)..end
            } else {
                (end + 1)..start
            };

            for intermediate in range {
                let intermediate_square = (from % 8) + 8 * intermediate;
                if self.all_pieces() & (1u64 << intermediate_square) != 0 {
                    return false;
                }
            }
        } else {
            return false;
        }

        if (opponent_pieces & to_mask) != 0 {
            self.clear_piece(to_mask, !is_white);
        }
        self.apply_move(from_mask, to_mask, Piece::Queen, is_white);
        true
    }
}
