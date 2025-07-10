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
#[derive(Debug, Clone, Copy)]
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

    pub fn empty() -> Self {
        Bitboard {
            white_king: 0, white_queen: 0, white_rook: 0, white_bishop: 0, white_knight: 0, white_pawns: 0,
            black_king: 0, black_queen: 0, black_rook: 0, black_bishop: 0, black_knight: 0, black_pawns: 0,
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

        let mut pawn_attacks: u64 = 0;
        if !attacker_is_white { // Attacker is Black, target is White
            if (position % 8 != 0) && position >= 9 { pawn_attacks |= 1u64 << (position - 9); }
            if (position % 8 != 7) && position >= 7 { pawn_attacks |= 1u64 << (position - 7); }
        } else { // Attacker is White, target is Black
            if (position % 8 != 0) && position < 56 { pawn_attacks |= 1u64 << (position + 7); }
            if (position % 8 != 7) && position < 56 { pawn_attacks |= 1u64 << (position + 9); }
        }

        if pawn_attacks & opponent_pawn != 0 {
            return  true;
        }

            
        if (self.get_knight_attacks(position) & opponent_knight) != 0 { return true; }
        if (self.get_king_attacks(position) & opponent_king) != 0 { return true; }


        // Sliding attack
        let all_pieces = self.all_pieces();
        let bishop_queen = opponent_bishop | opponent_queen;
        let rook_queen = opponent_rook | opponent_queen;


        if (Self::get_bishop_attacks(position, all_pieces) & bishop_queen) != 0 { return true; }
        if (Self::get_rook_attacks(position, all_pieces) & rook_queen) != 0 { return true; }



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
