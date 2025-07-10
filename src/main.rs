use std::time::Instant;
use chess_bot::test_runner::run_tests;
fn main() {
    let start = Instant::now();

    let file_path = "./src/perftsuite.txt";
    println!("--- Running Perft Test Suite from '{file_path}' ---");

    if let Err(e) = run_tests(file_path) {
        eprintln!("Error running test suite: {e}");
    }

    let duration = start.elapsed();
    println!("Time taken: {duration:.3?}");
}
