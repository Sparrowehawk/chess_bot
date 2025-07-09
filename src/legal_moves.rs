use crate::bitboard::Piece;
use crate::game::Game;

impl Game {
    pub fn generate_pseudo_legal_moves(&self) -> Vec<(usize, usize, Option<Piece>)> {
        let mut moves = Vec::new();
        self.generate_pawn_moves(&mut moves);
        self.generate_knight_moves(&mut moves);
        self.generate_bishop_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_queen_moves(&mut moves);
        self.generate_king_moves(&mut moves);
        moves
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<(usize, usize, Option<Piece>)>) {
        let (my_pawns, enemy_pieces, rank_7, rank_2, push_dir) = if self.is_white_turn {
            (self.board.white_pawns, self.board.black_pieces(), 6, 1, 8)
        } else {
            (self.board.black_pawns, self.board.white_pieces(), 1, 6, -8)
        };

        let all_pieces = self.board.all_pieces();
        let mut pawns = my_pawns;

        while pawns != 0 {
            let from = pawns.trailing_zeros() as usize;

            // Single and double push
            let push = (from as i8 + push_dir) as usize;
            if (all_pieces & (1u64 << push)) == 0 {
                // promotion
                if from / 8 == rank_7 {
                    moves.push((from, push, Some(Piece::Queen)));
                    moves.push((from, push, Some(Piece::Rook)));
                    moves.push((from, push, Some(Piece::Bishop)));
                    moves.push((from, push, Some(Piece::Knight)));
                } else {
                    moves.push((from, push, None));
                }

                if from / 8 == rank_2 {
                    let double_push = (from as i8 + 2 * push_dir) as usize;
                    if (all_pieces & (1u64 << double_push)) == 0 {
                        moves.push((from, double_push, None));
                    }
                }
            }

            // Captures
            for &capture_dir in &[-1, 1] {
                let to = (from as i8 + push_dir + capture_dir) as usize;
                if (from % 8 == 0 && capture_dir == -1) || (from % 8 == 7 && capture_dir == 1) {
                    continue;
                }

                if (enemy_pieces & (1u64 << to)) != 0 {
                    if from / 8 == rank_7 {
                        moves.push((from, to, Some(Piece::Queen)));
                        moves.push((from, to, Some(Piece::Rook)));
                        moves.push((from, to, Some(Piece::Bishop)));
                        moves.push((from, to, Some(Piece::Knight)));
                    } else {
                        moves.push((from, to, None));
                    }
                }
            }

            // En passent

            pawns &= pawns - 1;
        }

        if let Some(en_passent_target) = self.en_passent {
            let attacking_squares = if self.is_white_turn {
                ((1u64 << (en_passent_target - 7)) * ((en_passent_target % 8 != 0) as u64))
                    | ((1u64 << (en_passent_target - 9)) * ((en_passent_target % 8 != 7) as u64))
            } else {
                ((1u64 << (en_passent_target + 7)) * ((en_passent_target % 8 != 0) as u64))
                    | ((1u64 << (en_passent_target + 9)) * ((en_passent_target % 8 != 7) as u64))
            };

            let attackers = my_pawns & attacking_squares;
            if attackers != 0 {
                let from = attackers.trailing_zeros() as usize;
                moves.push((from, en_passent_target, None));
            }
        }
    }

    fn generate_knight_moves(&self, moves: &mut Vec<(usize, usize, Option<Piece>)>) {
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
                moves.push((from, to, None));
                attacks &= attacks - 1;
            }
            knights &= knights - 1;
        }
    }
fn generate_bishop_moves(&self, moves: &mut Vec<(usize, usize, Option<Piece>)>) {
    let (my_bishops, my_pieces, enemy_pieces) = if self.is_white_turn {
        (self.board.white_bishop, self.board.white_pieces(), self.board.black_pieces())
    } else {
        (self.board.black_bishop, self.board.black_pieces(), self.board.white_pieces())
    };

    let mut bishops = my_bishops;

    while bishops != 0 {
        let from = bishops.trailing_zeros() as usize;
        let directions = [-9, -7, 7, 9]; // Diagonal directions

        for &dir in &directions {
            let mut current_pos = from;
            loop {
                // Check for board edges before making the next step
                let at_h_file = current_pos % 8 == 7;
                let at_a_file = current_pos % 8 == 0;

                if (at_h_file && (dir == -7 || dir == 9)) || (at_a_file && (dir == -9 || dir == 7)) {
                    break;
                }
                
                let next_pos_i8 = current_pos as i8 + dir;
                if !(0..=63).contains(&next_pos_i8) { break; }
                current_pos = next_pos_i8 as usize;

                let to_mask = 1u64 << current_pos;

                if (my_pieces & to_mask) != 0 { break; } // Blocked by our own piece

                moves.push((from, current_pos, None));

                if (enemy_pieces & to_mask) != 0 { break; } // Capture, so stop here
            }
        }
        bishops &= bishops - 1;
    }
}

fn generate_rook_moves(&self, moves: &mut Vec<(usize, usize, Option<Piece>)>) {
    let (my_rooks, my_pieces, enemy_pieces) = if self.is_white_turn {
        (self.board.white_rook, self.board.white_pieces(), self.board.black_pieces())
    } else {
        (self.board.black_rook, self.board.black_pieces(), self.board.white_pieces())
    };

    let mut rooks = my_rooks;
    while rooks != 0 {
        let from = rooks.trailing_zeros() as usize;
        let directions = [-8, -1, 1, 8]; // Cardinal directions

        for &dir in &directions {
            let mut current_pos = from;
            loop {
                // Check for board edges before making the next step
                let at_h_file = current_pos % 8 == 7;
                let at_a_file = current_pos % 8 == 0;

                if (at_h_file && dir == 1) || (at_a_file && dir == -1) { break; }

                let next_pos_i8 = current_pos as i8 + dir;
                if !(0..=63).contains(&next_pos_i8) { break; }
                current_pos = next_pos_i8 as usize;
                
                let to_mask = 1u64 << current_pos;
                
                if (my_pieces & to_mask) != 0 { break; }

                moves.push((from, current_pos, None));

                if (enemy_pieces & to_mask) != 0 { break; }
            }
        }
        rooks &= rooks - 1;
    }
}

// A simpler and more direct implementation for the queen
fn generate_queen_moves(&self, moves: &mut Vec<(usize, usize, Option<Piece>)>) {
    let (my_queens, my_pieces, enemy_pieces) = if self.is_white_turn {
        (self.board.white_queen, self.board.white_pieces(), self.board.black_pieces())
    } else {
        (self.board.black_queen, self.board.black_pieces(), self.board.white_pieces())
    };
    
    let mut queens = my_queens;
    while queens != 0 {
        let from = queens.trailing_zeros() as usize;
        // All 8 directions combined
        let directions = [-9, -8, -7, -1, 1, 7, 8, 9]; 

        for &dir in &directions {
            let mut current_pos = from;
            loop {
                let at_h_file = current_pos % 8 == 7;
                let at_a_file = current_pos % 8 == 0;
                
                // Combined checks for all directions
                if (at_h_file && (dir == -7 || dir == 1 || dir == 9)) || (at_a_file && (dir == -9 || dir == -1 || dir == 7)) {
                    break;
                }

                let next_pos_i8 = current_pos as i8 + dir;
                if !(0..=63).contains(&next_pos_i8) { break; }
                current_pos = next_pos_i8 as usize;

                let to_mask = 1u64 << current_pos;

                if (my_pieces & to_mask) != 0 { break; }

                moves.push((from, current_pos, None));

                if (enemy_pieces & to_mask) != 0 { break; }
            }
        }
        queens &= queens - 1;
    }
}

    fn generate_king_moves(&self, moves: &mut Vec<(usize, usize, Option<Piece>)>) {
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
            moves.push((from, to, None));
            attacks &= attacks - 1;
        }

        // Castling
        let all = self.board.all_pieces();

        if self.is_white_turn {
            if (self.castling & 0b1000) != 0 && (all & 0x60) == 0
                && !self.board.possible_check(4, false) && !self.board.possible_check(5, false) {
                    moves.push((4, 6, None));
                }
            if (self.castling & 0b0100) != 0 && (all & 0xE) == 0
                && !self.board.possible_check(4, false) && !self.board.possible_check(3, false) {
                    moves.push((4, 2, None));
                }
        } else {
            if (self.castling & 0b0010) != 0 && (all & 0x6000000000000000) == 0
                && !self.board.possible_check(60, true) && !self.board.possible_check(61, true) {
                    moves.push((60, 62, None));
                }
            if (self.castling & 0b0001) != 0 && (all & 0xE00000000000000) == 0
                && !self.board.possible_check(60, true) && !self.board.possible_check(59, true) {
                    moves.push((60, 58, None));
                }
        }
    }
}
