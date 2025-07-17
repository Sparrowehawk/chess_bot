pub mod eval;
pub mod pst;
pub mod see;
pub mod tt;
pub mod zobrist;

use self::tt::Flag;
use crate::Piece;
use crate::game::Game;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

const MAX_LMR_DEPTH: usize = 64;
const MAX_LMR_MOVES: usize = 64;

const MATE_SCORE: i32 = i32::MAX;
const MATE_THRESHOLD: i32 = MATE_SCORE / 2;
const TEMPO_BONUS: i32 = 10;

pub const PHASE_WEIGHTS: [i32; 6] = [0, 1, 1, 2, 4, 0]; // pawn to king
pub const MAX_PHASE: i32 = 24;

const KILLER_MOVE_SCORE: i32 = 700_000;

pub const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000];
const MAX_PLY: usize = 64;
type KillerMove = Option<(usize, usize, Option<Piece>)>;

static LMR_TABLE: [[u8; MAX_LMR_MOVES]; MAX_LMR_DEPTH] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/lmr.bin"))) };

pub struct Search {
    pub nodes_searched: u64,
    killer_moves: [[KillerMove; 2]; MAX_PLY],
    history: [[i32; 64]; 12],
}

impl Default for Search {
    fn default() -> Self {
        Self {
            nodes_searched: 0,
            killer_moves: [[None; 2]; MAX_PLY],
            history: [[0; 64]; 12],
        }
    }
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a killer move for the current ply, shifting the existing one.
    fn add_killer_move(&mut self, ply: usize, mv: (usize, usize, Option<Piece>)) {
        if ply < MAX_PLY {
            self.killer_moves[ply][1] = self.killer_moves[ply][0];
            self.killer_moves[ply][0] = Some(mv);
        }
    }

    /// Updates the history score for a successful quiet move.
    fn update_history_score(&mut self, piece: Piece, to: usize, depth: i32) {
        // The bonus is squared to heavily reward cutoffs at higher depths.
        self.history[piece as usize][to] += depth * depth;
    }
}

pub fn get_piece_value(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize]
}

pub fn find_best_move(
    game: &mut Game,
    max_depth: u8,
    stop_signal: &Arc<AtomicBool>,
) -> (Option<(usize, usize, Option<Piece>)>, i32) {
    game.tt.lock().unwrap().clear();
    let mut best_move = None;
    let mut best_score = -MATE_SCORE;
    let mut search_helper = Search::new();
    let start_time = Instant::now();

    for depth in 1..=max_depth {
        let score = search(
            game,
            depth,
            -MATE_SCORE,
            MATE_SCORE,
            stop_signal,
            &mut search_helper,
        );

        let duration = start_time.elapsed();
        best_score = score;

        if stop_signal.load(Ordering::Relaxed) {
            println!("STOPPED");
            break;
        }

        let mut pv = Vec::new();
        let mut temp_game = game.clone(); // Create a temporary board to walk the PV
        for _ in 0..depth {
            let entry = game.tt.lock().unwrap().probe(temp_game.zobrist_hash);
            if let Some(entry) = entry {
                if let Some(mv) = entry.best_move {
                    pv.push(mv);
                    temp_game.make_move_unchecked(mv.0, mv.1, mv.2);
                } else {
                    break; // Stop if no best move is found
                }
            } else {
                break; // Stop if position not in TT
            }
        }

        if !pv.is_empty() {
            best_move = pv.first().copied();
        } else if let Some(entry) = game.tt.lock().unwrap().probe(game.zobrist_hash) {
            best_move = entry.best_move;
        }

        let pv_string = pv
            .iter()
            .map(|m| move_to_uci(*m))
            .collect::<Vec<_>>()
            .join(" ");

        if best_score.abs() >= MATE_THRESHOLD {
            let mate_in = (MATE_SCORE - best_score.abs() + 1) / 2;
            let sign = if best_score > 0 { "" } else { "-" };
            println!(
                "depth {depth} score mate {sign}{mate_in} time {} nodes {} pv {pv_string}",
                duration.as_millis(),
                search_helper.nodes_searched
            );
        } else {
            println!(
                "depth {depth} score cp {best_score} time {} nodes {} pv {pv_string}",
                duration.as_millis(),
                search_helper.nodes_searched
            );
        }

        if best_score >= MATE_SCORE - (max_depth as i32) {
            println!("STOPPED2");
            break;
        }
    }
    if best_move.is_none() {
        let legal_moves = game.generate_legal_moves();
        if !legal_moves.is_empty() {
            best_move = legal_moves.iter().next().copied(); // pick first legal move
        } else {
            println!("No legal moves found at the end of search!");
        }
    }

    (best_move, best_score)
}

fn search(
    game: &mut Game,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    stop_signal: &Arc<AtomicBool>,
    search_helper: &mut Search,
) -> i32 {
    search_helper.nodes_searched += 1;
    if depth == 0 {
        return eval::quiescence_search(game, alpha, beta, search_helper);
    }

    if stop_signal.load(Ordering::Relaxed) {
        return 0;
    }

    let key = game.zobrist_hash;
    if let Some(entry) = game.tt.lock().unwrap().probe(key) {
        if entry.depth >= depth {
            match entry.flag {
                Flag::Exact => return entry.score,
                Flag::LowerBound if entry.score >= beta => return beta,
                Flag::UpperBound if entry.score <= alpha => return alpha,
                _ => {}
            }
        }
    }

    let in_check = game.is_in_check();
    if in_check { depth + 1 } else { depth };

    let mut moves = game.generate_legal_moves();

    if moves.is_empty() {
        return if game.is_in_check() {
            -MATE_SCORE + pst::ply(game) as i32
        } else {
            0
        };
    }

    let ply = pst::ply(game) as usize;
    moves.sort_by_cached_key(|m| -(see::score_move(game, *m, ply, search_helper)));

    let mut best_move = None;
    let mut flag = Flag::UpperBound;

    for (move_count, m) in moves.iter().enumerate() {
        // Not affected by this
        let is_quiet = m.2.is_none()
            && (1u64 << m.1)
                & if game.is_white_turn {
                    game.board.black_pieces()
                } else {
                    game.board.white_pieces()
                }
                == 0;

        let mut reduce = 0;
        if is_quiet && depth > 2 && move_count > 1 {
            // Basic reduction from the table
            let d = (depth as usize).min(MAX_LMR_DEPTH - 1);
            let mv_idx = move_count.min(MAX_LMR_MOVES - 1);
            reduce = LMR_TABLE[d][mv_idx];
        }

        let piece = pst::get_piece_at(game, m.0);

        // This is modifying m?
        let undo = game.make_move_unchecked(m.0, m.1, m.2);

        let king_square = if !game.is_white_turn {
            game.board.white_king
        } else {
            game.board.black_king
        }
        .trailing_zeros() as usize;
        if game.board.possible_check(king_square, game.is_white_turn) {
            // King is in check, so this move was illegal. Unmake it and skip.
            game.unmake_move(undo);
            continue;
        }

        // Not affected by this
        let score = if reduce > 0 && depth > 2 {
            // reduced search
            let reduced_depth = depth.saturating_sub(reduce);
            let reduced_score = -search(
                game,
                reduced_depth,
                -alpha - 1,
                -alpha,
                stop_signal,
                search_helper,
            );
            if reduced_score > alpha && reduced_score < beta {
                // re-search at full depth
                -search(game, depth - 1, -beta, -alpha, stop_signal, search_helper)
            } else {
                reduced_score
            }
        } else {
            // full-depth search
            -search(game, depth - 1, -beta, -alpha, stop_signal, search_helper)
        };

        game.unmake_move(undo);

        if stop_signal.load(Ordering::Relaxed) {
            return 0;
        }

        if score >= beta {
            game.tt
                .lock()
                .unwrap()
                .store(key, depth, beta, Flag::LowerBound, Some(*m));
            return beta;
        }
        if score > alpha {
            alpha = score;
            best_move = Some(m);
            flag = Flag::Exact;
        }

        if is_quiet {
            if let Some(piece) = piece {
                search_helper.add_killer_move(ply, *m);
                search_helper.update_history_score(piece, m.1, depth as i32);
            }
        }
    }

    game.tt
        .lock()
        .unwrap()
        .store(key, depth, alpha, flag, best_move.copied());
    alpha
}

pub fn move_to_uci(mov: (usize, usize, Option<Piece>)) -> String {
    let from_sq = mov.0;
    let to_sq = mov.1;
    let promo = mov.2;

    let from_str = format!(
        "{}{}",
        ((from_sq % 8) as u8 + b'a') as char,
        (from_sq / 8) + 1
    );
    let to_str = format!("{}{}", ((to_sq % 8) as u8 + b'a') as char, (to_sq / 8) + 1);

    let promo_str = if let Some(p) = promo {
        match p {
            Piece::Queen => "q",
            Piece::Rook => "r",
            Piece::Bishop => "b",
            Piece::Knight => "n",
            _ => "",
        }
    } else {
        ""
    };

    format!("{from_str}{to_str}{promo_str}")
}
