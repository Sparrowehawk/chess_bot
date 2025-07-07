use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_knight(&mut self, from: usize, to: usize, is_white: bool) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let (knight_board, opponent_pieces, friendly_pieces) = if is_white {
            (self.white_knight, self.black_pieces(), self.white_pieces())
        } else {
            (self.black_knight, self.white_pieces(), self.black_pieces())
        };

        if (knight_board & from_mask) == 0 || (friendly_pieces & to_mask) != 0 {
            return false;
        }

        // Essentially, the knight can move +/- 2 up or across with +/- 1 up or across 
        let from_file = from % 8;
        let to_file = to % 8;

        let file_distance = (from_file as i8 - to_file as i8).abs();
        let rank_distance = ( (from / 8) as i8 - (to / 8) as i8 ).abs();

        if !((file_distance == 1 && rank_distance == 2) || (file_distance == 2 && rank_distance == 1)) {
            return false; 
        }

        if (opponent_pieces & to_mask) != 0 {
            self.clear_piece(to_mask, !is_white);
        } 
        self.apply_move(from_mask, to_mask, Piece::Knight, is_white);
        true
    }
}
