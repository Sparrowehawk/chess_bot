use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use crate::Game;

/// Reads the test file line by line and executes the perft tests.
pub fn run_tests<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut total_tests = 0;
    let mut failed_tests = 0;

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Each line is split into the FEN string and the perft results.
        let parts: Vec<&str> = line.split(';').map(|s| s.trim()).collect();
        let fen = parts[0];

        println!("\n[{}] Testing FEN: {}", index + 1, fen);

        // Create a game from the FEN.
        match Game::from_fen(fen) {
            Ok(mut game) => {
                // Loop through the D1, D2, etc. parts of the line.
                for &test_case in &parts[1..] {
                    total_tests += 1;
                    let test_parts: Vec<&str> = test_case.split_whitespace().collect();
                    if test_parts.len() != 2 {
                        println!("    -> \x1b[93mSKIP\x1b[0m: Malformed test case '{test_case}'");
                        continue;
                    }

                    // Parse the depth and expected node count.
                    let depth_str = &test_parts[0][1..]; // Remove the 'D'
                    let depth: u32 = depth_str.parse().expect("Invalid depth");
                    let expected_nodes: u64 = test_parts[1].parse().expect("Invalid node count");

                    print!("    -> Testing D{depth}: ");

                    // Run the perft test.
                    let start_time = Instant::now();
                    let actual_nodes = game.perft(depth);
                    let duration = start_time.elapsed();

                    // Compare results and print PASS or FAIL.
                    if actual_nodes == expected_nodes {
                        // ANSI escape code for green text
                        println!("\x1b[32mPASS\x1b[0m ({actual_nodes} nodes, {duration:.2?})");
                    } else {
                        // ANSI escape code for red text
                        println!("\x1b[31mFAIL\x1b[0m (Got: {actual_nodes}, Expected: {expected_nodes}, {duration:.2?})");
                        failed_tests += 1;
                    }
                }
            }
            Err(e) => {
                println!("    -> \x1b[93mSKIP\x1b[0m: Could not parse FEN. Error: {e}");
            }
        }
    }

    println!("\n--- Test Suite Finished ---");
    if failed_tests == 0 {
        println!("\x1b[32mAll {total_tests} tests passed!\x1b[0m");
    } else {
        println!("\x1b[31m{failed_tests} out of {total_tests} tests failed.\x1b[0m");
    }

    Ok(())
}