use crate::board::Bitboard;

pub fn print_board(board: &Bitboard) {
    for rank in (0..8).rev() {
        for file in 0..8 {
            let index = rank * 8 + file;
            let piece = piece_symbol(board, index);
            print!("{piece} ");
        }
        println!("           ROW {}", rank + 1);
    }
    println!("\na b c d e f g h");
}

fn piece_symbol(board: &Bitboard, index: u64) -> char {
    let mask = 1u64 << index;
    match () {
        _ if board.white_king & mask != 0 => '♚',
        _ if board.white_queen & mask != 0 => '♛',
        _ if board.white_rook & mask != 0 => '♜',
        _ if board.white_bishop & mask != 0 => '♝',
        _ if board.white_knight & mask != 0 => '♞',
        _ if board.white_pawns & mask != 0 => '♟',
        _ if board.black_king & mask != 0 => '♔',
        _ if board.black_queen & mask != 0 => '♕',
        _ if board.black_rook & mask != 0 => '♖',
        _ if board.black_bishop & mask != 0 => '♗',
        _ if board.black_knight & mask != 0 => '♘',
        _ if board.black_pawns & mask != 0 => '♙',
        _ => '.',
    }
}
