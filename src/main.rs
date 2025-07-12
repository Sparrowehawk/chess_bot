// use std::time::Instant;

// use std::time::Instant;
// use chess_bot::{test_runner::run_tests, Game};

use chess_bot::uci::uci_loop;

fn main() {
    // let start = Instant::now();
    
    // let mut game = Game::new();
    
    // game.perft_divide(6);
    // let fen = "4r2k/pp1q1p2/2p1r3/3b1N1p/3P1b1P/1PQ3N1/P4PP1/3RR1K1 w - - 1 27";
    // match Game::from_fen(fen) {
    //     Ok(mut game) => {
    //         println!("Successfully loaded position from FEN.");
    //         game.board.print_board();

    //         use std::sync::{Arc, atomic::AtomicBool};
    //         let stop_flag = Arc::new(AtomicBool::new(false));
    //         let (best_move, eval) = game.find_best_move(7, &stop_flag); // Added the second argument
    //         println!("{best_move:?}, {eval}");
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

    // let duration = start.elapsed();
    // println!("Time taken: {duration:.3?}");
    uci_loop();
}

