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


    let success = board.move_pawn(51, 35, false);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }

    let success = board.move_pawn(28, 35, true);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }

    let success = board.move_pawn(10, 26, true);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }


    let success = board.move_pawn(50, 42, false);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }

    let success = board.move_pawn(35, 43, true);
    if success {
        Bitboard::print_board(&board);
    } else {
        println!("Illegal move");
    }
}
