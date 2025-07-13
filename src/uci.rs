use crate::game::Game;
use crate::parser::parse_move;
use std::io::{self, BufRead};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Encapsulates the UCI state, including the game and a dedicated search thread.
struct Uci {
    game: Game,
    stop_signal: Arc<AtomicBool>,
    search_thread: Option<JoinHandle<()>>,
}

impl Uci {
    /// Creates a new Uci instance.
    fn new() -> Self {
        Uci {
            game: Game::new(),
            stop_signal: Arc::new(AtomicBool::new(false)),
            search_thread: None,
        }
    }

    /// The main loop that listens for and responds to UCI commands.
    fn uci_loop(&mut self) {
        let stdin = io::stdin();
        let mut reader = stdin.lock();

        loop {
            let mut input = String::new();
            if reader.read_line(&mut input).is_err() {
                break;
            }

            let tokens: Vec<&str> = input.split_whitespace().collect();
            if let Some(&command) = tokens.first() {
                match command {
                    "uci" => self.handle_uci(),
                    "isready" => self.handle_isready(),
                    "ucinewgame" => self.handle_new_game(),
                    "setoption" => self.handle_setoption(&tokens),
                    "position" => self.handle_position(&tokens),
                    "go" => self.handle_go(&tokens),
                    "stop" => self.handle_stop(),
                    "quit" => break,
                    _ => {} // Ignore unknown commands
                }
            }
        }
    }

    /// Responds to the "uci" command by identifying the engine and its options.
    fn handle_uci(&self) {
        println!("id name SparroweEngine");
        println!("id author Sparrowe");
        // Advertise supported UCI options
        println!("option name Move Overhead type spin default 300 min 0 max 1000");
        println!("option name Threads type spin default 1 min 1 max 128");
        println!("option name Hash type spin default 128 min 1 max 2048");
        println!("uciok");
    }

    /// Acknowledges that the engine is ready.
    fn handle_isready(&self) {
        println!("readyok");
    }

    /// Resets the game to the starting position.
    fn handle_new_game(&mut self) {
        self.handle_stop(); // Stop any thinking before starting a new game
        self.game = Game::new();
    }

    /// Handles UCI options set by the GUI.
    fn handle_setoption(&mut self, tokens: &[&str]) {
        if let (Some(&"name"), Some(name), Some(&"value"), Some(value)) =
            (tokens.get(1), tokens.get(2), tokens.get(3), tokens.get(4))
        {
            if name == &"Hash" {
                if let Ok(mb) = value.parse::<u64>() {
                    // This is where you would resize your transposition table.
                    // For example: self.game.tt.resize(mb);
                    eprintln!("info string Hash size set to {mb} MB");
                }
            }
        }
    }

    /// Sets the board position from a FEN string or a sequence of moves.
    fn handle_position(&mut self, tokens: &[&str]) {
        let mut current_index = 1;
        if tokens.get(current_index) == Some(&"startpos") {
            self.game = Game::new();
            current_index += 1;
        } else if tokens.get(current_index) == Some(&"fen") {
            current_index += 1;
            let fen_parts: Vec<&str> = tokens[current_index..]
                .iter()
                .take_while(|&&s| s != "moves")
                .cloned()
                .collect();

            if let Ok(new_game) = Game::from_fen(&fen_parts.join(" ")) {
                self.game = new_game;
            }
            current_index += fen_parts.len();
        }

        if tokens.get(current_index) == Some(&"moves") {
            current_index += 1;
            for move_str in &tokens[current_index..] {
                if let Some((from, to, promo)) = parse_move(move_str) {
                    self.game.make_move(from, to, promo);
                }
            }
        }
    }

    /// Starts a search on a dedicated thread.
    fn handle_go(&mut self, tokens: &[&str]) {
        self.handle_stop(); // Ensure any previous search is stopped and joined.

        // Constants for time management (in milliseconds)
        const MAX_THINK_TIME: u64 = 60_000; // The 60-second hard cap
        const MOVE_OVERHEAD: u64 = 500; // Safety buffer to avoid losing on time

        let depth = find_token_value(tokens, "depth").map_or(u8::MAX, |d| d as u8);

        // Determine the time to think for this move
        // let time_to_think = if let Some(ms) = find_token_value(tokens, "movetime") {
        //     // Case 1: The GUI gives a fixed time for this move. We must obey it.
        //     Some(ms)
        // } else {
        //     // Case 2: Standard time controls (wtime/btime with increments)
        //     let (time, inc) = if self.game.is_white_turn {
        //         (find_token_value(tokens, "wtime"), find_token_value(tokens, "winc"))
        //     } else {
        //         (find_token_value(tokens, "btime"), find_token_value(tokens, "binc"))
        //     };

        //     time.map(|mut t| {
        //         // Ensure we have some time to think and apply the overhead
        //         t = t.saturating_sub(MOVE_OVERHEAD);

        //         // A simple strategy: use 1/30th of the remaining time, plus the increment.
        //         // This aims to have enough time for at least 30 moves in the game.
        //         let ideal_time = (t / 30) + inc.unwrap_or(0);

        //         // Apply the 60-second cap and ensure we don't use more time than we have.
        //         ideal_time.min(MAX_THINK_TIME).min(t)
        //     })
        // };

        let time_to_think = 60_000;

        // Set the stop signal to false before starting a new search
        self.stop_signal.store(false, Ordering::Relaxed);

        // If a thinking time was determined, spawn a timer thread to enforce it.
        // if let Some(think_ms) = time_to_think {
        //     let stop_clone = Arc::clone(&self.stop_signal);
        //     thread::spawn(move || {
        //         thread::sleep(Duration::from_millis(think_ms));
        //         stop_clone.store(true, Ordering::Relaxed);
        //     });
        // }
        let stop_clone = Arc::clone(&self.stop_signal);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(time_to_think));
            stop_clone.store(true, Ordering::Relaxed);
        });

        // --- The rest of the function remains the same ---
        let mut game_clone = self.game.clone();
        let stop_clone = Arc::clone(&self.stop_signal);

        self.search_thread = Some(thread::spawn(move || {
            let (best_move, _) = game_clone.find_best_move(depth, &stop_clone);

            if let Some((from, to, promo)) = best_move {
                let move_str = format!(
                    "{}{}{}",
                    game_clone.square_index_to_coord(from),
                    game_clone.square_index_to_coord(to),
                    game_clone.promo_to_char(promo)
                );
                println!("bestmove {move_str}");
            } else {
                println!("bestmove 0000"); // Fallback if no move is found
            }
        }));
    }

    /// Stops the currently running search and waits for it to terminate.
    fn handle_stop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(handle) = self.search_thread.take() {
            handle.join().unwrap();
        }
    }
}

/// Helper to find a numeric value associated with a UCI token.
fn find_token_value(tokens: &[&str], token: &str) -> Option<u64> {
    tokens
        .iter()
        .position(|&s| s == token)
        .and_then(|i| tokens.get(i + 1))
        .and_then(|s| s.parse().ok())
}

/// The main entry point for the UCI application.
pub fn uci_loop() {
    let mut uci = Uci::new();
    uci.uci_loop();
}
