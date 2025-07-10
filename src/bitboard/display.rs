use crate::bitboard::Bitboard;

impl Bitboard{
    // For us to see
    pub fn print_board(&self) {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let index = rank * 8 + file;
                let piece = self.piece_symbol(index);
                print!("{piece} ");
            }
            println!("           ROW {}", rank + 1);
        }
        println!("\na b c d e f g h");
    }

    fn piece_symbol(&self ,index: u64) -> char {
        let mask = 1u64 << index;
        match () {
            _ if self.white_king & mask != 0 => '♚',
            _ if self.white_queen & mask != 0 => '♛',
            _ if self.white_rook & mask != 0 => '♜',
            _ if self.white_bishop & mask != 0 => '♝',
            _ if self.white_knight & mask != 0 => '♞',
            _ if self.white_pawns & mask != 0 => '♟',
            _ if self.black_king & mask != 0 => '♔',
            _ if self.black_queen & mask != 0 => '♕',
            _ if self.black_rook & mask != 0 => '♖',
            _ if self.black_bishop & mask != 0 => '♗',
            _ if self.black_knight & mask != 0 => '♘',
            _ if self.black_pawns & mask != 0 => '♙',
            _ => '.',
        }
    }
}