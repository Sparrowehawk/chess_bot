pub mod display;
pub mod moves;

#[derive(Clone)] // Mainly used in tests
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
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Bitboard {
    pub fn new() -> Self {
        Self::default()
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

    pub fn possible_check(&self, position: usize, attacked_is_white: bool) -> bool {
        let (
            opponent_pawn,
            opponent_knight,
            opponent_bishop,
            opponent_rook,
            opponent_queen,
            opponent_king,
        ) = if attacked_is_white {
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


        let pawn_attacks = if attacked_is_white { [7, 9] } else { [-7, -9] };
        for attacks in pawn_attacks {
            let from_square = (position as isize) + attacks;
            if (0..64).contains(&from_square)
                && (opponent_pawn & (1u64 << from_square)) != 0 {
                    // Wrap around check
                    let from_file = (from_square as usize) % 8;
                    let to_file = position % 8;
                    if (from_file as i8 - to_file as i8).abs() == 1 {
                        return true;
                    }
                }
        }


        let knight_attacks = [-17, -15, -10, -6, 6, 10, 15, 17];
        for attacks in knight_attacks {
            let from_square = (position as isize) + attacks;
            if (0..64).contains(&from_square)
                && (opponent_knight & (1u64 << from_square)) != 0 {
                    // Wrap around check
                    let from_file = (from_square as usize) % 8;
                    let to_file = position % 8;
                    if (from_file as i8 - to_file as i8).abs() == 1 {
                        return true;
                    }
                }
        }


        // Sliding attack
        let all_pieces = self.all_pieces();
        let bishop_queen = opponent_bishop | opponent_queen;
        let rook_queen = opponent_rook | opponent_queen;

        let directions = [-8, 8, 1, -1, -7, -9, 7, 9];
        for (i, &dir) in directions.iter().enumerate() {
            for n in 1..8 {
                let target = (position as isize) +dir * n;
                if !(0..64).contains(&target) {
                    break;
                }

                let from_file = ((target - dir) as usize) % 8;
                let to_file = ((target - dir) as usize) % 8;
                if (from_file as i8 - to_file as i8).abs() > 1 {
                    break;
                }

                let target_mask = 1u64 << target;
                if (all_pieces & target_mask) != 0 {
                    if i < 4 {
                        if (rook_queen & target_mask) != 0 {
                            println!("{n}");
                            return true;
                        }
                    } else if (bishop_queen & target_mask) != 0 {
                        return true;
                    }
                    break;
                }
            }
        }


        // King moves
        for offset in directions {
            let from_square = { position as isize } + offset;
            if from_square > 0 && from_square < 64
                && (opponent_king & (1u64 << from_square)) != 0 {
                    let from_file = (from_square as usize) % 8;
                    let to_file = position % 8;
                    if (from_file as i8 - to_file as i8).abs() <= 1 {
                        return true;
                    }
                }
        }
        false
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
