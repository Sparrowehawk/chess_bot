pub mod bitboard;
pub use bitboard::Bitboard;

fn main() {
    println!("Hello");
    let mut board = Bitboard::new();
    let success = board.move_pawn(12, 28, true);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }


}
