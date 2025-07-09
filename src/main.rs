use std::time::Instant;
use chess_bot::{parser::parse_move, Game};
fn main() {
    let start = Instant::now();

    let fen = "8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1";
    match Game::from_fen(fen) {
        Ok(game) => {
            println!("Successfully loaded position from FEN.");
            game.board.print_board();

            println!("\nIt is Black's turn. Generating legal moves...");

            let legal_moves = game.generate_legal_moves();
            for (from, to, promo) in &legal_moves {
                let move_str = format!(
                    "{}{}{}",
                    game.square_index_to_coord(*from),
                    game.square_index_to_coord(*to),
                    game.promo_to_char(*promo)
                );
                println!("{}", move_str);
            }

            println!("\nTotal legal moves found: {}", legal_moves.len());
        }
        Err(e) => {
            println!("Failed to load FEN: {}", e);
        }
    }

    let duration = start.elapsed();
    println!("Time taken: {:.3?}", duration);
}
