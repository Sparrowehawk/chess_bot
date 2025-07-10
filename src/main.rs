use std::time::Instant;
use chess_bot::{test_runner::run_tests, Game};
fn main() {
    let start = Instant::now();
    
    // let mut game = Game::new();
    // game.perft_divide(5);
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    // match Game::from_fen(fen) {
    //     Ok(mut game) => {
    //         println!("Successfully loaded position from FEN.");
    //         game.board.print_board();

    //         game.perft_divide(1);
    //     }
    //     Err(e) => {
    //         println!("Failed to load FEN: {}", e);
    //     }
    // }

    let file_path = "./src/perftsuite.txt";
    println!("--- Running Perft Test Suite from '{file_path}' ---");

    if let Err(e) = run_tests(file_path) {
        eprintln!("Error running test suite: {e}");
    }

    let duration = start.elapsed();
    println!("Time taken: {:.3?}", duration);
}

