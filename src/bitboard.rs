pub mod display;
pub mod moves;

pub struct Bitboard {
    white_king: u64,
    white_queen: u64,
    white_rook: u64,
    white_bishop: u64,
    white_knight: u64,
    white_pawns: u64,
    black_king: u64,
    black_queen: u64,
    black_rook: u64,
    black_bishop: u64,
    black_knight: u64,
    black_pawns: u64,
}
pub enum Piece {
    King, Queen, Rook, Bishop, Knight, Pawn, 
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
