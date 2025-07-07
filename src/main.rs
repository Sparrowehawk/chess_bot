pub mod bitboard;
pub use bitboard::Bitboard;

pub mod parser;
pub use parser::parse_move;
use std::io::{self, Write};

fn main() {
    let mut board = Bitboard::new();
    let mut is_white_turn = true;

    loop {
        // 1. Print the board
        board.print_board();

        // 2. Prompt for input
        let player = if is_white_turn { "White" } else { "Black" };
        print!("\n{player}> ");
        io::stdout().flush().unwrap(); // Ensure the prompt appears before input

        // 3. Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        // 4. Parse the input
        let trimmed_input = input.trim();
        if trimmed_input == "exit" {
            break;
        }

        match parse_move(trimmed_input) {
            Some((from, to, _promo)) => {
                // 5. Attempt to make the move using the dispatcher
                if board.make_move(from, to, is_white_turn) {
                    // 6. If successful, switch turns
                    is_white_turn = !is_white_turn;
                } else {
                    println!("\n*** Illegal move! Try again. ***");
                }
            }
            None => {
                println!("\n*** Invalid format. Use algebraic notation (e.g., 'e2e4'). ***");
            }
        }
    }
}