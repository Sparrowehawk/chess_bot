use crate::movelist::MoveList;
use crate::Bitboard;
use crate::bitboard::Piece;
use crate::game::Game;

impl Game {
    // Returns all possible moves for each piece
    pub fn generate_pseudo_legal_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        self.generate_pawn_moves(&mut moves);
        self.generate_knight_moves(&mut moves);
        self.generate_bishop_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_queen_moves(&mut moves);
        self.generate_king_moves(&mut moves);
        moves
    }

    // Biggest crutch since atm it isn't pre computed
    fn generate_pawn_moves(&self,  moves: &mut MoveList) {
        let (my_pawns, enemy_pieces, rank_7, _rank_2, _push_dir, colour) = if self.is_white_turn {
            (
                self.board.white_pawns,
                self.board.black_pieces(),
                6,
                1,
                8,
                0,
            )
        } else {
            (
                self.board.black_pawns,
                self.board.white_pieces(),
                1,
                6,
                -8,
                1,
            )
        };

        let all_pieces = self.board.all_pieces();
        let mut pawns = my_pawns;

        while pawns != 0 {
            let from = pawns.trailing_zeros() as usize;

            let mut push_targets = Bitboard::get_pawn_pushes(colour, from) & !all_pieces;
            while push_targets != 0 {
                let to = push_targets.trailing_zeros() as usize;

                if from / 8 == rank_7 {
                    moves.add(from, to, Some(Piece::Queen));
                    moves.add(from, to, Some(Piece::Rook));
                    moves.add(from, to, Some(Piece::Bishop));
                    moves.add(from, to, Some(Piece::Knight));
                } else {
                    moves.add(from, to, None);
                }

                push_targets &= push_targets - 1; // Clear the lowest set bit
            }

            let mut capture_targets = Bitboard::get_pawn_attacks(colour, from) & enemy_pieces;
            
            while capture_targets != 0 {
                let to = capture_targets.trailing_zeros() as usize;

                if from / 8 == rank_7 {
                    moves.add(from, to, Some(Piece::Queen));
                    moves.add(from, to, Some(Piece::Rook));
                    moves.add(from, to, Some(Piece::Bishop));
                    moves.add(from, to, Some(Piece::Knight));
                } else {
                    moves.add(from, to, None);
                }

                capture_targets &= capture_targets - 1;
            }

            pawns &= pawns - 1;
        }

        if let Some(ep_square) = self.en_passent {
            let required_rank = if self.is_white_turn { 4 } else { 3 };
            let ep_file = ep_square % 8;
            if ep_file > 0 {
                let attacker_pos = if self.is_white_turn {
                    ep_square - 9
                } else {
                    ep_square + 7
                };
                if (attacker_pos / 8 == required_rank) && ((my_pawns & (1u64 << attacker_pos)) != 0)
                {
                    moves.add(attacker_pos, ep_square, None);
                }
            }
            if ep_file < 7 {
                let attacker_pos = if self.is_white_turn {
                    ep_square - 7
                } else {
                    ep_square + 9
                };
                if (attacker_pos / 8 == required_rank) && ((my_pawns & (1u64 << attacker_pos)) != 0)
                {
                    moves.add(attacker_pos, ep_square, None);
                }
            }
        }
    }

    // Rest of these work the same
    // calls the pre computed bitboard
    fn generate_knight_moves(&self, moves: &mut MoveList) {
        let (my_knights, my_pieces) = if self.is_white_turn {
            (self.board.white_knight, self.board.white_pieces())
        } else {
            (self.board.black_knight, self.board.black_pieces())
        };
        let mut knights = my_knights;
        while knights != 0 {
            let from = knights.trailing_zeros() as usize;
            let mut attacks = self.board.get_knight_attacks(from);
            attacks &= !my_pieces;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                moves.add(from, to, None);
                attacks &= attacks - 1;
            }
            knights &= knights - 1;
        }
    }

    fn generate_bishop_moves(&self, moves: &mut MoveList) {
        let (my_bishops, my_pieces) = if self.is_white_turn {
            (self.board.white_bishop, self.board.white_pieces())
        } else {
            (self.board.black_bishop, self.board.black_pieces())
        };

        let mut bishops = my_bishops;
        while bishops != 0 {
            let from = bishops.trailing_zeros() as usize;
            let mut attacks = Bitboard::get_bishop_attacks(from, self.board.all_pieces());
            attacks &= !my_pieces;

            let mut targets = attacks;
            while targets != 0 {
                let to = targets.trailing_zeros() as usize;
                moves.add(from, to, None);
                targets &= targets - 1;
            }
            bishops &= bishops - 1;
        }
    }

    fn generate_rook_moves(&self, moves: &mut MoveList) {
        let (my_rooks, my_pieces) = if self.is_white_turn {
            (self.board.white_rook, self.board.white_pieces())
        } else {
            (self.board.black_rook, self.board.black_pieces())
        };

        let mut rooks = my_rooks;
        while rooks != 0 {
            let from = rooks.trailing_zeros() as usize;
            let mut attacks = Bitboard::get_rook_attacks(from, self.board.all_pieces());
            attacks &= !my_pieces;

            let mut targets = attacks;
            while targets != 0 {
                let to = targets.trailing_zeros() as usize;
                moves.add(from, to, None);
                targets &= targets - 1;
            }
            rooks &= rooks - 1;
        }
    }

    fn generate_queen_moves(&self, moves: &mut MoveList) {
        let (my_queens, my_pieces) = if self.is_white_turn {
            (self.board.white_queen, self.board.white_pieces())
        } else {
            (self.board.black_queen, self.board.black_pieces())
        };

        let mut queens = my_queens;
        while queens != 0 {
            let from = queens.trailing_zeros() as usize;
            let blockers = self.board.all_pieces();

            // A queen's move is the union of a rook's and bishop's moves from the same square.
            let mut attacks = Bitboard::get_rook_attacks(from, blockers)
                | Bitboard::get_bishop_attacks(from, blockers);

            attacks &= !my_pieces; // Can't capture your own pieces

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                moves.add(from, to, None);
                attacks &= attacks - 1;
            }
            queens &= queens - 1;
        }
    }

    fn generate_king_moves(&self, moves: &mut MoveList) {
        let (my_king, my_pieces) = if self.is_white_turn {
            (self.board.white_king, self.board.white_pieces())
        } else {
            (self.board.black_king, self.board.black_pieces())
        };
        if my_king == 0 {
            return;
        }
        let from = my_king.trailing_zeros() as usize;
        let mut attacks = self.board.get_king_attacks(from);
        attacks &= !my_pieces;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            moves.add(from, to, None);
            attacks &= attacks - 1;
        }
        let all = self.board.all_pieces();
        if self.is_white_turn {
            if (self.castling & 0b1000) != 0
                && (all & 0x60) == 0
                && !self.board.possible_check(4, false)
                && !self.board.possible_check(5, false)
            {
                moves.add(4, 6, None);
            }
            if (self.castling & 0b0100) != 0
                && (all & 0xE) == 0
                && !self.board.possible_check(4, false)
                && !self.board.possible_check(3, false)
            {
                moves.add(4, 2, None);
            }
        } else {
            if (self.castling & 0b0010) != 0
                && (all & 0x6000000000000000) == 0
                && !self.board.possible_check(60, true)
                && !self.board.possible_check(61, true)
            {
                moves.add(60, 62, None);
            }
            if (self.castling & 0b0001) != 0
                && (all & 0xE00000000000000) == 0
                && !self.board.possible_check(60, true)
                && !self.board.possible_check(59, true)
            {
                moves.add(60, 58, None);
            }
        }
    }
}
