// Optimized search.rs module
use crate::bitboard::Piece;
use crate::game::Game;
use crate::transposition_table::Flag;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

const MATE_SCORE: i32 = 1000000;
const MATE_THRESHOLD: i32 = MATE_SCORE / 2;
const TEMPO_BONUS: i32 = 10;

const PROMOTION_SCORE: i32 = 900_000;
const CAPTURE_SCORE: i32 = 800_000;
const KILLER_MOVE_SCORE: i32 = 700_000;


const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000];
const MAX_PLY: usize = 64; 

pub struct Search {
    // Stores two "killer moves" for each ply. These are quiet moves that caused a beta cutoff.
    killer_moves: [[Option<(usize, usize, Option<Piece>)>; 2]; MAX_PLY],
    // History heuristic table: [piece][to_square]
    // Scores quiet moves based on how often they are successful.
    history: [[i32; 64]; 12],
}

impl Default for Search {
    fn default() -> Self {
        Self {
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


fn get_piece_value(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize]
}
// --- Piece-Square Tables (PSTs) ---
// These tables give positional bonuses or penalties to pieces based on their location.
// Values are oriented from White's perspective. Black's values are mirrored.

const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 15, 15,  5,  5,-10,
    -10,  0, 10, 15, 15, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_PST: [i32; 64] = [
      0,  0,  0,  0,  0,  0,  0,  0,
      5, 10, 10, 10, 10, 10, 10,  5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
      0,  0,  0,  5,  5,  0,  0,  0
];

const QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

const KING_PST_MIDDLE_GAME: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

const PIECE_SQUARE_TABLES: [[i32; 64]; 6] = [
    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST_MIDDLE_GAME
];


impl Game {

    pub fn find_best_move(
        &mut self,
        max_depth: u8,
        stop_signal: &Arc<AtomicBool>,
    ) -> (Option<(usize, usize, Option<Piece>)>, i32) {
        let mut best_move = None;
        let mut best_score = -MATE_SCORE;

        for depth in 1..=max_depth {
            let start_time = Instant::now();
            let score = self.search(depth, -MATE_SCORE, MATE_SCORE, stop_signal);
            let duration = start_time.elapsed();
            best_score = score;

            if stop_signal.load(Ordering::Relaxed) {
                break;
            }

            if let Some(entry) = self.tt.lock().unwrap().probe(self.hash_position()) {
                if let Some(m) = entry.best_move {
                    best_move = Some(m);
                }
            }

            if best_score.abs() >= MATE_THRESHOLD {
                let mate_in = (MATE_SCORE - best_score.abs() + 1) / 2;
                let sign = if best_score > 0 { "" } else { "-" };
                println!("info depth {depth} score mate {sign}{mate_in} time {}", duration.as_millis());
            } else {
                println!("info depth {depth} score cp {best_score} time {}", duration.as_millis());
            }

            if best_score >= MATE_SCORE - (max_depth as i32) {
                break;
            }
        }
        (best_move, best_score)
    }

    fn search(&mut self, depth: u8, mut alpha: i32, beta: i32, stop_signal: &Arc<AtomicBool>) -> i32 {
        if depth == 0 {
            return self.quiescence_search(alpha, beta);
        }

        if stop_signal.load(Ordering::Relaxed) {
            return 0;
        }

        let key = self.hash_position();
        if let Some(entry) = self.tt.lock().unwrap().probe(key) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => return entry.score,
                    Flag::LowerBound if entry.score >= beta => return beta,
                    Flag::UpperBound if entry.score <= alpha => return alpha,
                    _ => {}
                }
            }
        }

        let mut moves = self.generate_legal_moves();
        if moves.is_empty() {
            return if self.is_in_check() { -MATE_SCORE + self.ply() as i32 } else { 0 };
        }

        moves.sort_by_cached_key(|m| -(self.score_move(*m)));

        let mut best_move = None;
        let mut flag = Flag::UpperBound;

        for m in moves.iter() {
            let undo = self.make_move_unchecked(m.0, m.1, m.2);
            let score = -self.search(depth - 1, -beta, -alpha, stop_signal);
            self.unmake_move(undo);

            if stop_signal.load(Ordering::Relaxed) {
                return 0;
            }

            if score >= beta {
                self.tt.lock().unwrap().store(key, depth, beta, Flag::LowerBound, Some(*m));
                return beta;
            }
            if score > alpha {
                alpha = score;
                best_move = Some(m);
                flag = Flag::Exact;
            }
        }

        self.tt.lock().unwrap().store(key, depth, alpha, flag, best_move.map(|m| *m));
        alpha
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.eval();
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let enemy_pieces = if self.is_white_turn { self.board.black_pieces() } else { self.board.white_pieces() };
        let mut moves = self.generate_legal_moves();
        moves.retain(|&(_, to, _)| (1u64 << to) & enemy_pieces != 0);
        moves.sort_by_cached_key(|m| -(self.score_move(*m)));

        for m in moves.iter() {
            let undo = self.make_move_unchecked(m.0, m.1, m.2);
            let score = -self.quiescence_search(-beta, -alpha);
            self.unmake_move(undo);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    fn score_move(&self, mov: (usize, usize, Option<Piece>)) -> i32 {
        let (from, to, promo) = mov;
        if let Some(p) = promo {
            return match p {
                Piece::Queen => 20000 + get_piece_value(p),
                Piece::Knight => 15000 + get_piece_value(p),
                _ => 10000 + get_piece_value(p)
            };
        }
        let enemy_pieces = if self.is_white_turn {
            self.board.black_pieces()
        } else {
            self.board.white_pieces()
        };
        if (1u64 << to) & enemy_pieces != 0 {
            let attacker = self.get_piece_at(from).unwrap_or(Piece::Pawn);
            let victim = self.get_piece_at(to).unwrap_or(Piece::Pawn);
            return 10000 + get_piece_value(victim) - get_piece_value(attacker);
        }
        0
    }

    pub fn get_piece_at(&self, square: usize) -> Option<Piece> {
        let mask = 1u64 << square;
        if (self.board.white_pawns | self.board.black_pawns) & mask != 0 { return Some(Piece::Pawn); }
        if (self.board.white_knight | self.board.black_knight) & mask != 0 { return Some(Piece::Knight); }
        if (self.board.white_bishop | self.board.black_bishop) & mask != 0 { return Some(Piece::Bishop); }
        if (self.board.white_rook | self.board.black_rook) & mask != 0 { return Some(Piece::Rook); }
        if (self.board.white_queen | self.board.black_queen) & mask != 0 { return Some(Piece::Queen); }
        if (self.board.white_king | self.board.black_king) & mask != 0 { return Some(Piece::King); }
        None
    }

    fn eval(&self) -> i32 {
        let white_score = self.calculate_score(true);
        let black_score = self.calculate_score(false);
        let perspective = if self.is_white_turn { 1 } else { -1 };
        (white_score - black_score + TEMPO_BONUS) * perspective
    }

    fn calculate_score(&self, is_white: bool) -> i32 {
        let mut total_score = 0;
        let board = &self.board;
        let piece_bitboards = if is_white {
            [board.white_pawns, board.white_knight, board.white_bishop, board.white_rook, board.white_queen, board.white_king]
        } else {
            [board.black_pawns, board.black_knight, board.black_bishop, board.black_rook, board.black_queen, board.black_king]
        };
        for (piece_idx, &bb) in piece_bitboards.iter().enumerate() {
            let mut bb = bb;
            total_score += bb.count_ones() as i32 * PIECE_VALUES[piece_idx];
            while bb != 0 {
                let square = bb.trailing_zeros() as usize;
                let pst_idx = if is_white { square ^ 56 } else { square };
                total_score += PIECE_SQUARE_TABLES[piece_idx][pst_idx];
                bb &= bb - 1;
            }
        }
        total_score
    }

    fn ply(&self) -> u8 {
        self.position_history.len() as u8
    }
}
