pub mod king;
pub mod queen;
pub mod rook;
pub mod bishop;
pub mod knight;
pub mod pawn;

use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    fn apply_move(&mut self, from_mask: u64, to_mask: u64, piece: Piece, is_white: bool) {
        let tmp_board = self.get_mut_board(piece, is_white);
        *tmp_board &= !from_mask;
        *tmp_board |= to_mask;

    }

    fn clear_piece(&mut self, mask: u64, is_white: bool) {
        let targets = if is_white {
            &mut [
                &mut self.white_pawns,
                &mut self.white_rook,
                &mut self.white_knight,
                &mut self.white_bishop,
                &mut self.white_queen,
                &mut self.white_king,
            ]
        } else {
            &mut [
                &mut self.black_pawns,
                &mut self.black_rook,
                &mut self.black_knight,
                &mut self.black_bishop,
                &mut self.black_queen,
                &mut self.black_king,
            ]
        };

        for piece in targets.iter_mut() {
            **piece &= !mask;
        }
    }

    pub fn get_mut_board(&mut self, piece: Piece, is_white: bool) -> &mut u64 {
        if is_white {
            match piece {
                Piece::King => &mut self.white_king,
                Piece::Queen => &mut self.white_queen,
                Piece::Rook => &mut self.white_rook,
                Piece::Bishop => &mut self.white_bishop,
                Piece::Knight => &mut self.white_knight,
                Piece::Pawn => &mut self.white_pawns,
            }
        } else {
            match piece {
                Piece::King => &mut self.black_king,
                Piece::Queen => &mut self.black_queen,
                Piece::Rook => &mut self.black_rook,
                Piece::Bishop => &mut self.black_bishop,
                Piece::Knight => &mut self.black_knight,
                Piece::Pawn => &mut self.black_pawns,
            }
        }
    }
}
