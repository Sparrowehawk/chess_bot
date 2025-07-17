// use std::time::Instant;

use chess_bot::board::display::print_board;
use chess_bot::search::eval::eval;
use chess_bot::search::find_best_move;
// use std::time::Instant;
use chess_bot::Game;

use chess_bot::uci::uci_loop;

fn main() {
    // let start = Instant::now();

    // let mut game = Game::new();
    //             use std::sync::{Arc, atomic::AtomicBool};
    //         let stop_flag = Arc::new(AtomicBool::new(false));
    // game.find_best_move(8, &stop_flag);

    // game.perft_divide(6);
    // let fen = "rn1q1bnQ/3kpp2/2pp3p/pp4p1/1P2b3/2P2N1P/P2PPPB1/RNB1K2R b KQ - 0 2";
    // match Game::from_fen(fen) {
    //     Ok(mut game) => {
    //         println!("Successfully loaded position from FEN.");
    //         print_board(&game.board);
    //         println!("Eval in this pos is: {}", eval(&game));

    //         game.perft_divide(2);
    //         // println!("{}", game.perft(6));

    //         // use std::sync::{Arc, atomic::AtomicBool};
    //         // let stop_flag = Arc::new(AtomicBool::new(false));
    //         // let (best_move, eval) = find_best_move(&mut game, 5, &stop_flag); // Added the second argument
    //         // println!("{best_move:?}, {eval}");
    //     }
    //     Err(e) => {
    //         println!("Failed to load FEN: {e}");
    //     }
    // }

    let file_path = "./src/utils/perftsuite.txt";
    println!("--- Running Perft Test Suite from '{file_path}' ---");

    if let Err(e) = chess_bot::utils::test_runner::run_tests(file_path) {
        eprintln!("Error running test suite: {e}");
    }

    // let duration = start.elapsed();
    // println!("Time taken: {duration:.3?}");
    // uci_loop();
}
