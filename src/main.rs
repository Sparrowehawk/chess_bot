// use std::time::Instant;

// use std::time::Instant;
// use chess_bot::Game;

use chess_bot::uci::uci_loop;

fn main() {
    // let start = Instant::now();
    
    // let mut game = Game::new();
    //             use std::sync::{Arc, atomic::AtomicBool};
    //         let stop_flag = Arc::new(AtomicBool::new(false));
    // game.find_best_move(8, &stop_flag);
    
    // game.perft_divide(6);
    // let fen = "r1bqk3/2pp2p1/p2N2N1/n2Q1rp1/Pp5P/8/1PP2PP1/R4K1R b - - 0 24";
    // match Game::from_fen(fen) {
    //     Ok(mut game) => {
    //         println!("Successfully loaded position from FEN.");
    //         game.board.print_board();
    //         println!("Eval in this pos is: {}", game.eval());

    //         // game.perft_divide(6);
    //         // println!("{}", game.perft(6));

    //         use std::sync::{Arc, atomic::AtomicBool};
    //         let stop_flag = Arc::new(AtomicBool::new(false));
    //         let (best_move, eval) = game.find_best_move(6, &stop_flag); // Added the second argument
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

