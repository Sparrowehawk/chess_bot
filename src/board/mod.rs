use crate::{board::pre_calculated::{bishop_magics, rook_magics}, Piece};

pub mod display;
pub mod movegen;
pub mod pre_calculated;

#[derive(Clone)]
pub struct Bitboard {
    pub white_king: u64,
    pub white_queen: u64,
    pub white_rook: u64,
    pub white_bishop: u64,
    pub white_knight: u64,
    pub white_pawns: u64,
    pub black_king: u64,
    pub black_queen: u64,
    pub black_rook: u64,
    pub black_bishop: u64,
    pub black_knight: u64,
    pub black_pawns: u64,
}
impl Bitboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty() -> Self {
        Bitboard {
            white_king: 0,
            white_queen: 0,
            white_rook: 0,
            white_bishop: 0,
            white_knight: 0,
            white_pawns: 0,
            black_king: 0,
            black_queen: 0,
            black_rook: 0,
            black_bishop: 0,
            black_knight: 0,
            black_pawns: 0,
        }
    }

    pub fn white_pieces(&self) -> u64 {
        self.white_king
            | self.white_queen
            | self.white_rook
            | self.white_bishop
            | self.white_knight
            | self.white_pawns
    }

    pub fn black_pieces(&self) -> u64 {
        self.black_king
            | self.black_queen
            | self.black_rook
            | self.black_bishop
            | self.black_knight
            | self.black_pawns
    }

    pub fn all_pieces(&self) -> u64 {
        self.white_pieces() | self.black_pieces()
    }

    pub fn possible_check(&self, position: usize, attacker_is_white: bool) -> bool {
        let (
            opponent_pawn,
            opponent_knight,
            opponent_bishop,
            opponent_rook,
            opponent_queen,
            opponent_king,
        ) = if attacker_is_white {
            (
                self.white_pawns,
                self.white_knight,
                self.white_bishop,
                self.white_rook,
                self.white_queen,
                self.white_king,
            )
        } else {
            (
                self.black_pawns,
                self.black_knight,
                self.black_bishop,
                self.black_rook,
                self.black_queen,
                self.black_king,
            )
        };


        if (Self::get_pawn_attacks(if attacker_is_white { 1 } else { 0 }, position) & opponent_pawn)
            != 0
        {
            return true;
        }
        if (self.get_knight_attacks(position) & opponent_knight) != 0 {
            return true;
        }
        if (self.get_king_attacks(position) & opponent_king) != 0 {
            return true;
        }

        // Sliding attacks
        let all_pieces = self.all_pieces();
        if (Self::get_bishop_attacks(position, all_pieces) & (opponent_bishop | opponent_queen))
            != 0
        {
            return true;
        }
        if (Self::get_rook_attacks(position, all_pieces) & (opponent_rook | opponent_queen)) != 0 {
            return true;
        }

        false
    }

    pub fn get_knight_attacks(&self, from: usize) -> u64 {
        pre_calculated::KNIGHT_ATTACKS[from]
    }

    pub fn get_king_attacks(&self, from: usize) -> u64 {
        pre_calculated::KING_ATTACKS[from]
    }

    pub fn get_bishop_attacks(square: usize, all_pieces: u64) -> u64 {
        let blockers = all_pieces & bishop_magics::BISHOP_MASKS[square];
        let magic_index = (blockers.wrapping_mul(bishop_magics::BISHOP_MAGICS[square])
            >> bishop_magics::BISHOP_SHIFTS[square]) as usize;
        let offset = bishop_magics::BISHOP_OFFSETS[square];
        bishop_magics::BISHOP_ATTACKS[offset + magic_index]
    }

    pub fn get_rook_attacks(square: usize, all_pieces: u64) -> u64 {
        let blockers = all_pieces & rook_magics::ROOK_MASKS[square];
        let magic_index = (blockers.wrapping_mul(rook_magics::ROOK_MAGIC[square])
            >> rook_magics::ROOK_SHIFTS[square]) as usize;
        let offset = rook_magics::ROOK_OFFSETS[square];
        rook_magics::ROOK_ATTACKS[offset + magic_index]
    }

    pub fn get_pawn_attacks(colour: usize, from: usize) -> u64 {
        pre_calculated::PAWN_ATTACKS[colour][from]
    }

    pub fn get_pawn_pushes(colour: usize, from: usize) -> u64 {
        pre_calculated::PAWN_PUSHES[colour][from]
    }

    fn apply_move(&mut self, from_mask: u64, to_mask: u64, piece: Piece, is_white: bool) {
        // Everything else required is done via individual piece
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
        let rank_distance = ((from / 8) as i8 - (to / 8) as i8).abs();

        if file_distance != rank_distance {
            return false;
        }

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

        if (opponent_pieces & to_mask) != 0 {
            self.clear_piece(to_mask, !is_white);
        }
        self.apply_move(from_mask, to_mask, Piece::Bishop, is_white);
        true
    }

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
        let rank_distance = ((from / 8) as i8 - (to / 8) as i8).abs();

        if !((file_distance == 1 && rank_distance == 2)
            || (file_distance == 2 && rank_distance == 1))
        {
            return false;
        }

        if (opponent_pieces & to_mask) != 0 {
            self.clear_piece(to_mask, !is_white);
        }
        self.apply_move(from_mask, to_mask, Piece::Knight, is_white);
        true
    }

    pub fn move_queen(&mut self, from: usize, to: usize, is_white: bool) -> bool {
        // Queen is bassically a rook and a bishop combined

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

        // Remove the rights
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

        // Checks if the pawn is to capture or en passant

        if self.is_pawn_capture(from, to, is_white) {
            // En passent check
            if Some(to) == en_passent_target && self.is_pawn_capture(from, to, is_white) {
                self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
                let caputed_pawn_pos = if is_white { to - 8 } else { to + 8 };
                self.clear_piece(1u64 << caputed_pawn_pos, !is_white);
                true
            } else {
                // Regualr capture check
                self.pawn_capture(from, to, opponent_pieces, is_white, promo)
            }
        } else {
            // SHINZO SASAGEYOOOOOOOOOOOOOOOOO
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
            // Promotion check
            if is_white {
                if to / 8 == 7 {
                    return self.promo_move(from_mask, to_mask, is_white, promo);
                }
            } else if to / 8 == 0 {
                return self.promo_move(from_mask, to_mask, is_white, promo);
            }

            if promo.is_some() {
                // Cannot promo on double move
                return false;
            }

            self.apply_move(from_mask, to_mask, Piece::Pawn, is_white);
            return true;
        }

        // Double push check
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
                self.clear_piece(to_mask, !is_white);
                self.apply_move(from_mask, to_mask, Piece::Queen, is_white);
            }
            Some(Piece::Rook) => {
                self.clear_piece(to_mask, !is_white);
                self.apply_move(from_mask, to_mask, Piece::Rook, is_white);
            }
            Some(Piece::Bishop) => {
                self.clear_piece(to_mask, !is_white);
                self.apply_move(from_mask, to_mask, Piece::Bishop, is_white);
            }
            Some(Piece::Knight) => {
                self.clear_piece(to_mask, !is_white);
                self.apply_move(from_mask, to_mask, Piece::Knight, is_white);
            }
            None => return false,
            _ => return false,
        }

        self.clear_piece(from_mask, is_white);
        true
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Self {
            white_king: 0x0000000000000010,   // 0x00000010
            white_queen: 0x0000000000000008,  // 0x00000008
            white_rook: 0x0000000000000081,   // 0x00000081
            white_bishop: 0x0000000000000024, // 0x00000024
            white_knight: 0x0000000000000042, // 0x00000042
            white_pawns: 0x000000000000FF00,  // 0x0000FF00
            black_king: 0x1000000000000000,   // 0x08000000
            black_queen: 0x0800000000000000,  // 0x10000000
            black_rook: 0x8100000000000000,   // 0x81000000
            black_bishop: 0x2400000000000000, // 0x24000000
            black_knight: 0x4200000000000000, // 0x42000000
            black_pawns: 0x00FF000000000000,  // 0x00FF0000
        }
    }
}
