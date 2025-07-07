pub mod bitboard;
pub use bitboard::Bitboard;

fn main() {
    println!("Hello");
    let mut board = Bitboard::new();
    let success = board.move_pawn(51, 43, false);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }

    let success = board.move_bishop(58, 40, false);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }


}
