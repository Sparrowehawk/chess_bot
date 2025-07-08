use crate::bitboard::{Bitboard, Piece};

impl Bitboard {
    pub fn move_pawn(
        &mut self,
        from: usize,
        to: usize,
        is_white: bool,
        promo: Option<Piece>,
        en_passent_target: Option<usize>,
        en_passent_next: &mut Option<usize>,
    ) -> bool {
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

        // println!("{en_passent_target:?}");
        if Some(to) == en_passent_target && self.is_pawn_capture(from, to, is_white) {
            self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
            let caputed_pawn_pos = if is_white { to - 8 } else { to + 8 };
            self.clear_piece(1u64 << caputed_pawn_pos, !is_white);
            return true;
        }

        if self.is_pawn_capture(from, to, is_white) {
            self.pawn_capture(from, to, opponent_pieces, is_white, promo)
        } else {
            self.pawn_push(from, to, is_white, promo, en_passent_next)
        }
    }

    fn pawn_push(
        &mut self,
        from: usize,
        to: usize,
        is_white: bool,
        promo: Option<Piece>,
        en_passent_next: &mut Option<usize>,
    ) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;
        if (self.all_pieces() & to_mask) != 0 {
            return false;
        }

        let single_valid_push = if is_white {
            from + 8 == to
        } else {
            to + 8 == from
        };
        if single_valid_push {
            if is_white {
                if to / 8 == 7 {
                    return self.promo_move(from_mask, to_mask, is_white, promo);
                }
            } else if to / 8 == 0 {
                return self.promo_move(from_mask, to_mask, is_white, promo);
            }

            if promo.is_some() {
                return false;
            }

            self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
            return true;
        }

        let is_on_starting_rank = if is_white {
            from / 8 == 1
        } else {
            from / 8 == 6
        };

        let double_push_valid = if is_white {
            from + 16 == to
        } else {
            to + 16 == from
        };

        if is_on_starting_rank && double_push_valid {
            let intermediate_square = if is_white { from + 8 } else { from - 8 };
            if (self.all_pieces() & (1u64 << intermediate_square)) == 0 {
                self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
                *en_passent_next = Some(intermediate_square);
                return true;
            }
        }
        false
    }

    fn pawn_capture(
        &mut self,
        from: usize,
        to: usize,
        opponent_pieces: u64,
        is_white: bool,
        promo: Option<Piece>,
    ) -> bool {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;
        if (opponent_pieces & to_mask) == 0 {
            return false;
        }

        if is_white {
            if to / 8 == 7 {
                return self.promo_move(from_mask, to_mask, is_white, promo);
            }
        } else if to / 8 == 0 {
            return self.promo_move(from_mask, to_mask, is_white, promo);
        }

        if promo.is_some() {
            return false;
        }
        self.clear_piece(to_mask, !is_white);
        self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
        true
    }

    fn is_pawn_capture(&self, from: usize, to: usize, is_white: bool) -> bool {
        if is_white {
            from + 7 == to || from + 9 == to
        } else {
            to + 7 == from || to + 9 == from
        }
    }

    fn promo_move(
        &mut self,
        from_mask: u64,
        to_mask: u64,
        is_white: bool,
        promo: Option<Piece>,
    ) -> bool {
        match promo {
            Some(Piece::Queen) => {
                self.apply_move(from_mask, to_mask, Piece::Queen, is_white);
            }
            Some(Piece::Rook) => {
                self.apply_move(from_mask, to_mask, Piece::Rook, is_white);
            }
            Some(Piece::Bishop) => {
                self.apply_move(from_mask, to_mask, Piece::Bishop, is_white);
            }
            Some(Piece::Knight) => {
                self.apply_move(from_mask, to_mask, Piece::Knight, is_white);
            }
            None => return false,
            _ => return false,
        }

        self.clear_piece(from_mask, is_white);
        true
    }
}
