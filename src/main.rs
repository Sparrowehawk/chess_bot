// use std::time::Instant;
use chess_bot::{test_runner::run_tests, uci::uci_loop, Game};
fn main() {
    // let start = Instant::now();
    
    // let mut game = Game::new();
    // game.perft_divide(6);
    // let fen = "4r2k/pp1q1p2/2p1r3/3b1N1p/3P1b1P/1PQ3N1/P4PP1/3RR1K1 w - - 1 27";
    // match Game::from_fen(fen) {
    //     Ok(mut game) => {
    //         println!("Successfully loaded position from FEN.");
    //         game.board.print_board();

    //         let (best_move, eval) = game.find_best_move(6);
    //         println!("{best_move:?}, {eval}");
    //     }
    //     Err(e) => {
    //         println!("Failed to load FEN: {e}");
    //     }
    // }

    // // let file_path = "./src/perftsuite.txt";
    // // println!("--- Running Perft Test Suite from '{file_path}' ---");

    // // if let Err(e) = run_tests(file_path) {
    // //     eprintln!("Error running test suite: {e}");
    // // }

    // let duration = start.elapsed();
    // println!("Time taken: {duration:.3?}");
    uci_loop();
}

