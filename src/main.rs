use std::time::Instant;

// use std::time::Instant;
use chess_bot::{test_runner::run_tests, Game};

use chess_bot::uci::uci_loop;

fn main() {
    // let start = Instant::now();
    
    // let mut game = Game::new();
    
    // game.perft_divide(6);
    let fen = "rnbqk1nr/1pp1bpp1/p2p3p/4p3/3PP3/P2B4/1PPQ1PPP/RNB1K1NR w KQkq - 2 6";
    match Game::from_fen(fen) {
        Ok(mut game) => {
            println!("Successfully loaded position from FEN.");
            game.board.print_board();
            println!("Eval in this pos is: {}", game.eval());

            use std::sync::{Arc, atomic::AtomicBool};
            let stop_flag = Arc::new(AtomicBool::new(false));
            let (best_move, eval) = game.find_best_move(7, &stop_flag); // Added the second argument
            println!("{best_move:?}, {eval}");
        }
        Err(e) => {
            println!("Failed to load FEN: {e}");
        }
    }

    // let file_path = "./src/perftsuite.txt";
    // println!("--- Running Perft Test Suite from '{file_path}' ---");

    // if let Err(e) = run_tests(file_path) {
    //     eprintln!("Error running test suite: {e}");
    // }

    // let duration = start.elapsed();
    // println!("Time taken: {duration:.3?}");
    // uci_loop();
}

