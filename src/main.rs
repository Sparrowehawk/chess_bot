use std::time::Instant;
use chess_bot::{test_runner::run_tests, Game};
fn main() {
    let start = Instant::now();
    
    let mut game = Game::new();
    game.perft_divide(6);
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/2R1K2R w Kkq - 0 2";
    // match Game::from_fen(fen) {
    //     Ok(mut game) => {
    //         println!("Successfully loaded position from FEN.");
    //         game.board.print_board();

    //         game.perft_divide(1);
    //     }
    //     Err(e) => {
    //         println!("Failed to load FEN: {e}");
    //     }
    // }

    // let file_path = "./src/perftsuite.txt";
    // println!("--- Running Perft Test Suite from '{file_path}' ---");

    // if let Err(e) = run_tests(file_path) {
    //     eprintln!("Error running test suite: {e}");
    // }

    let duration = start.elapsed();
    println!("Time taken: {duration:.3?}");
}

