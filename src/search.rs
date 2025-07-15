use crate::bitboard::Piece;
use crate::game::Game;
use crate::transposition_table::Flag;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

const MAX_LMR_DEPTH: usize = 64;
const MAX_LMR_MOVES: usize = 64;

const MATE_SCORE: i32 = 1000000;
const MATE_THRESHOLD: i32 = MATE_SCORE / 2;
const TEMPO_BONUS: i32 = 10;

// const PROMOTION_SCORE: i32 = 900_000;
// const CAPTURE_SCORE: i32 = 800_000;
const KILLER_MOVE_SCORE: i32 = 700_000;

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000];
const MAX_PLY: usize = 64;
type KillerMove = Option<(usize, usize, Option<Piece>)>;

static LMR_TABLE: [[u8; MAX_LMR_MOVES]; MAX_LMR_DEPTH] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/lmr.bin"))) };
pub struct Search {
    killer_moves: [[KillerMove; 2]; MAX_PLY],
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
// Gonna use PeSTO tables and whatnot for now

const PHASE_WEIGHTS: [i32; 6] = [0, 1, 1, 2, 4, 0]; // pawn to king
const MAX_PHASE: i32 = 24;

const MG_PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];

const EG_PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53, 82,
    84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1, -8, 13,
    8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
];

const MG_KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];

const EG_KNIGHT_TABLE: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9,
    -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1,
    15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50,
    -64,
];

const MG_BISHOP_TABLE: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];

const EG_BISHOP_TABLE: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0,
    4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
];

const MG_ROOK_TABLE: [i32; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];

const EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3, 13,
    1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6, 0, 2, -9,
    -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
];

const MG_QUEEN_TABLE: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];

const EG_QUEEN_TABLE: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17, 10, 5,
    -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
];

const MG_KING_TABLE: [i32; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6,
    22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14,
    -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24,
    14,
];

const EG_KING_TABLE: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

const EG_TABLE: [[i32; 64]; 6] = [
    EG_PAWN_TABLE,
    EG_KNIGHT_TABLE,
    EG_BISHOP_TABLE,
    EG_ROOK_TABLE,
    EG_QUEEN_TABLE,
    EG_KING_TABLE,
];

const MG_TABLE: [[i32; 64]; 6] = [
    MG_PAWN_TABLE,
    MG_KNIGHT_TABLE,
    MG_BISHOP_TABLE,
    MG_ROOK_TABLE,
    MG_QUEEN_TABLE,
    MG_KING_TABLE,
];

impl Game {
    pub fn find_best_move(
        &mut self,
        max_depth: u8,
        stop_signal: &Arc<AtomicBool>,
    ) -> (Option<(usize, usize, Option<Piece>)>, i32) {
        self.tt.lock().unwrap().clear();
        let mut best_move = None;
        let mut best_score = -MATE_SCORE;
        let mut search_helper = Search::new();
        let start_time = Instant::now();

        for depth in 1..=max_depth {
            let score = self.search(
                depth,
                -MATE_SCORE,
                MATE_SCORE,
                stop_signal,
                &mut search_helper,
            );

            let duration = start_time.elapsed();
            best_score = score;

            if stop_signal.load(Ordering::Relaxed) {
                break;
            }

            let mut pv = Vec::new();
            let mut temp_game = self.clone(); // Create a temporary board to walk the PV
            for _ in 0..depth {
                let entry = temp_game.tt.lock().unwrap().probe(temp_game.zobrist_hash);
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

            // The best move is the first move of our reconstructed PV
            best_move = pv.first().copied();

            let pv_string = pv
                .iter()
                .map(|m| self.move_to_uci(*m))
                .collect::<Vec<_>>()
                .join(" ");

            if best_score.abs() >= MATE_THRESHOLD {
                let mate_in = (MATE_SCORE - best_score.abs() + 1) / 2;
                let sign = if best_score > 0 { "" } else { "-" };
                println!(
                    "info depth {depth} score mate {sign}{mate_in} time {} pv {pv_string}",
                    duration.as_millis()
                );
            } else {
                println!(
                    "info depth {depth} score cp {best_score} time {} pv {pv_string}",
                    duration.as_millis()
                );
            }

            if best_score >= MATE_SCORE - (max_depth as i32) {
                break;
            }
        }
        (best_move, best_score)
    }

    pub fn move_to_uci(&self, mov: (usize, usize, Option<Piece>)) -> String {
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

    fn search(
        &mut self,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        stop_signal: &Arc<AtomicBool>,
        search_helper: &mut Search,
    ) -> i32 {
        // self.check_board_integrity("search entry");
        if depth == 0 {
            return self.quiescence_search(alpha, beta, search_helper);
        }

        if stop_signal.load(Ordering::Relaxed) {
            return 0;
        }

        let key = self.zobrist_hash;
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
            return if self.is_in_check() {
                -MATE_SCORE + self.ply() as i32
            } else {
                0
            };
        }

        let ply = self.ply() as usize;
        moves.sort_by_cached_key(|m| -(self.score_move(*m, ply, search_helper)));

        let mut best_move = None;
        let mut flag = Flag::UpperBound;

        for (move_count, m) in moves.iter().enumerate() {
            // Not affected by this
            let is_quiet = m.2.is_none()
                && (1u64 << m.1)
                    & if self.is_white_turn {
                        self.board.black_pieces()
                    } else {
                        self.board.white_pieces()
                    }
                    == 0;

            let mut reduce = 0;
            if is_quiet && depth > 2 && move_count > 1 {
                // Basic reduction from the table
                let d = (depth as usize).min(MAX_LMR_DEPTH - 1);
                let mv_idx = move_count.min(MAX_LMR_MOVES - 1);
                reduce = LMR_TABLE[d][mv_idx];
            }

            // Just for now dw
            let mut temp_game = self.clone();
            if !temp_game.make_move(m.0, m.1, m.2) {
                println!("❌ Warning: move {m:?} rejected by make_move but still being searched");
                // You should skip this move:
                continue;
            }

            let piece = self.get_piece_at(m.0);

            // This is modifying m?
            let undo = self.make_move_unchecked(m.0, m.1, m.2);

            // Not affected by this
            let score = if reduce > 0 && depth > 2 {
                // reduced search
                let reduced_depth = depth.saturating_sub(reduce);
                let reduced_score = -self.search(
                    reduced_depth,
                    -alpha - 1,
                    -alpha,
                    stop_signal,
                    search_helper,
                );
                if reduced_score > alpha && reduced_score < beta {
                    // re-search at full depth
                    -self.search(depth - 1, -beta, -alpha, stop_signal, search_helper)
                } else {
                    reduced_score
                }
            } else {
                // full-depth search
                -self.search(depth - 1, -beta, -alpha, stop_signal, search_helper)
            };

            self.unmake_move(undo);

            if stop_signal.load(Ordering::Relaxed) {
                return 0;
            }

            if score >= beta {
                self.tt
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

        self.tt
            .lock()
            .unwrap()
            .store(key, depth, alpha, flag, best_move.copied());
        alpha
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32, search_helper: &mut Search) -> i32 {
        let stand_pat = self.eval();
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let enemy_pieces = if self.is_white_turn {
            self.board.black_pieces()
        } else {
            self.board.white_pieces()
        };
        let mut moves = self.generate_legal_moves();
        moves.retain(|&(_, to, _)| (1u64 << to) & enemy_pieces != 0);

        let ply = self.ply() as usize;
        moves.sort_by_cached_key(|m| -(self.score_move(*m, ply, search_helper)));

        for m in moves.iter() {
            if self.static_exchange_loses(m.0, m.1) {
                continue;
            }

            let undo = self.make_move_unchecked(m.0, m.1, m.2);
            let score = -self.quiescence_search(-beta, -alpha, search_helper);
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

    fn static_exchange_loses(&self, from: usize, to: usize) -> bool {
        let mut occupied = self.board.all_pieces();
        let mut white_attackers = self.board.attackers_to(to, true);
        let mut black_attackers = self.board.attackers_to(to, false);

        let mut gain = [0i32; 32];
        let mut depth = 0;

        // Who's turn is it?
        let mut side = self.is_white_turn;

        // Get initial attacker and victim
        let mut piece = self.get_piece_at(from).unwrap();
        gain[0] = PIECE_VALUES[self.get_piece_at(to).unwrap() as usize];

        // Remove the initial attacker from the board
        occupied &= !(1u64 << from);
        if side {
            white_attackers &= !(1u64 << from);
        } else {
            black_attackers &= !(1u64 << from);
        }

        side = !side;

        loop {
            depth += 1;
            gain[depth] = PIECE_VALUES[piece as usize] - gain[depth - 1];

            // Decide which side moves next
            let next_attackers = if side {
                white_attackers
            } else {
                black_attackers
            };

            if next_attackers == 0 {
                break;
            }

            // Pick least valuable attacker
            let next_attacker_sq = Self::least_valuable_piece(self, next_attackers, side);
            if let Some(sq) = next_attacker_sq {
                piece = self.get_piece_at(sq).unwrap();
                occupied &= !(1u64 << sq);

                if side {
                    white_attackers &= !(1u64 << sq);
                } else {
                    black_attackers &= !(1u64 << sq);
                }

                // Recalculate x-ray attackers (simplified)
                // If sliding pieces are behind the target square, re-add them
                // (Optional and complex, can be skipped for speed)

                side = !side;
            } else {
                break;
            }
        }

        // Backward evaluation: assume opponent makes best choices
        while depth > 0 {
            depth -= 1;
            gain[depth] = -gain[depth + 1].max(-gain[depth]);
        }

        gain[0] < 0
    }

    fn least_valuable_piece(&self, attackers: u64, is_white: bool) -> Option<usize> {
        let board = &self.board;
        let piece_order = [
            (
                Piece::Pawn,
                if is_white {
                    board.white_pawns
                } else {
                    board.black_pawns
                },
            ),
            (
                Piece::Knight,
                if is_white {
                    board.white_knight
                } else {
                    board.black_knight
                },
            ),
            (
                Piece::Bishop,
                if is_white {
                    board.white_bishop
                } else {
                    board.black_bishop
                },
            ),
            (
                Piece::Rook,
                if is_white {
                    board.white_rook
                } else {
                    board.black_rook
                },
            ),
            (
                Piece::Queen,
                if is_white {
                    board.white_queen
                } else {
                    board.black_queen
                },
            ),
            (
                Piece::King,
                if is_white {
                    board.white_king
                } else {
                    board.black_king
                },
            ),
        ];

        for (_piece, bb) in piece_order.iter() {
            let masked = attackers & bb;
            if masked != 0 {
                return Some(masked.trailing_zeros() as usize);
            }
        }

        None
    }

    fn score_move(
        &self,
        mov: (usize, usize, Option<Piece>),
        ply: usize,
        search_helper: &mut Search,
    ) -> i32 {
        let (from, to, promo) = mov;

        if let Some(p) = promo {
            return match p {
                Piece::Queen => 20000 + get_piece_value(p),
                Piece::Knight => 15000 + get_piece_value(p),
                _ => 10000 + get_piece_value(p),
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

        if ply < MAX_PLY {
            if Some(mov) == search_helper.killer_moves[ply][0] {
                return KILLER_MOVE_SCORE + 2;
            }
            if Some(mov) == search_helper.killer_moves[ply][1] {
                return KILLER_MOVE_SCORE + 1;
            }
        }

        if let Some(piece) = self.get_piece_at(from) {
            return search_helper.history[piece as usize][to];
        }

        0
    }

    pub fn get_piece_at(&self, square: usize) -> Option<Piece> {
        let mask = 1u64 << square;
        if (self.board.white_pawns | self.board.black_pawns) & mask != 0 {
            return Some(Piece::Pawn);
        }
        if (self.board.white_knight | self.board.black_knight) & mask != 0 {
            return Some(Piece::Knight);
        }
        if (self.board.white_bishop | self.board.black_bishop) & mask != 0 {
            return Some(Piece::Bishop);
        }
        if (self.board.white_rook | self.board.black_rook) & mask != 0 {
            return Some(Piece::Rook);
        }
        if (self.board.white_queen | self.board.black_queen) & mask != 0 {
            return Some(Piece::Queen);
        }
        if (self.board.white_king | self.board.black_king) & mask != 0 {
            return Some(Piece::King);
        }
        None
    }

    pub fn eval(&self) -> i32 {
        let (white_mg, white_eg, white_phase) = self.calculate_score(true);
        let (black_mg, black_eg, black_phase) = self.calculate_score(false);

        let total_phase = (white_phase + black_phase).min(MAX_PHASE);
        let mg_score = white_mg - black_mg;
        let eg_score = white_eg - black_eg;

        let blended_score =
            (mg_score * total_phase + eg_score * (MAX_PHASE - total_phase)) / MAX_PHASE;

        let perspective = if self.is_white_turn { 1 } else { -1 };
        (blended_score + TEMPO_BONUS) * perspective
    }

    fn calculate_score(&self, is_white: bool) -> (i32, i32, i32) {
        let mut mg_score = 0;
        let mut eg_score = 0;
        let mut phase_score = 0;

        let board = &self.board;

        // Define friend and foe pieces for clarity
        let (
            friend_pawns,
            friend_knights,
            friend_bishops,
            friend_rooks,
            friend_queens,
            friend_king,
        ) = if is_white {
            (
                board.white_pawns,
                board.white_knight,
                board.white_bishop,
                board.white_rook,
                board.white_queen,
                board.white_king,
            )
        } else {
            (
                board.black_pawns,
                board.black_knight,
                board.black_bishop,
                board.black_rook,
                board.black_queen,
                board.black_king,
            )
        };

        let foe_pawns = if is_white {
            board.black_pawns
        } else {
            board.white_pawns
        };

        let friend_bitboards = [
            friend_pawns,
            friend_knights,
            friend_bishops,
            friend_rooks,
            friend_queens,
            friend_king,
        ];

        // 1. Material and PST scores (your existing logic, with a small bug fix)
        for (piece_idx, &bb) in friend_bitboards.iter().enumerate() {
            let piece_count = bb.count_ones() as i32;

            mg_score += piece_count * PIECE_VALUES[piece_idx];
            eg_score += piece_count * PIECE_VALUES[piece_idx];
            phase_score += piece_count * PHASE_WEIGHTS[piece_idx];

            let mut temp_bb = bb;
            while temp_bb != 0 {
                let square = temp_bb.trailing_zeros() as usize;
                // The board flip for white is correct
                let pst_idx = if is_white { square ^ 56 } else { square };

                mg_score += MG_TABLE[piece_idx][pst_idx];
                eg_score += EG_TABLE[piece_idx][pst_idx];

                temp_bb &= temp_bb - 1; // Clear the least significant bit
            }
        }

        // 2. Bishop Pair Bonus
        if friend_bishops.count_ones() >= 2 {
            mg_score += 30; // Middlegame bonus
            eg_score += 50; // Endgame bonus
        }

        // 3. Pawn Structure Evaluation
        let (pawn_mg, pawn_eg) = self.evaluate_pawn_structure(friend_pawns, foe_pawns, is_white);
        mg_score += pawn_mg;
        eg_score += pawn_eg;

        // 4. Rook on Open/Semi-Open File Bonus
        let (rook_mg, rook_eg) = self.evaluate_rooks(friend_rooks, friend_pawns, foe_pawns);
        mg_score += rook_mg;
        eg_score += rook_eg;

        // Future additions could include:
        // - King Safety
        // - Mobility
        // - Threats

        (mg_score, eg_score, phase_score)
    }

    fn evaluate_pawn_structure(
        &self,
        friend_pawns: u64,
        foe_pawns: u64,
        is_white: bool,
    ) -> (i32, i32) {
        let mut mg = 0;
        let mut eg = 0;

        // Penalty for doubled pawns
        let doubled_penalty_mg = -10;
        let doubled_penalty_eg = -20;
        for &file_mask in &FILE_MASKS {
            if (friend_pawns & file_mask).count_ones() > 1 {
                mg += doubled_penalty_mg;
                eg += doubled_penalty_eg;
            }
        }

        let mut temp_pawns = friend_pawns;
        while temp_pawns != 0 {
            let square = temp_pawns.trailing_zeros() as usize;
            let file = square % 8;

            // Penalty for isolated pawns (no friendly pawns on adjacent files)
            if (friend_pawns & ADJACENT_FILES_MASKS[file]) == 0 {
                mg += -15;
                eg += -25;
            }

            // Bonus for passed pawns (no enemy pawns in front)
            if self.is_passed(square, is_white, foe_pawns) {
                let rank = if is_white {
                    square / 8
                } else {
                    7 - (square / 8)
                };
                // Bonus increases dramatically as the pawn advances
                mg += [0, 10, 20, 30, 50, 75, 100, 0][rank];
                eg += [0, 20, 30, 45, 65, 90, 120, 0][rank];
            }

            temp_pawns &= temp_pawns - 1;
        }

        (mg, eg)
    }

    fn is_passed(&self, square: usize, is_white: bool, foe_pawns: u64) -> bool {
        let file = square % 8;
        let rank = square / 8;

        let mut front_mask = FILE_MASKS[file];
        let mut adjacent_mask = ADJACENT_FILES_MASKS[file];

        if is_white {
            front_mask <<= (rank + 1) * 8;
            adjacent_mask <<= (rank + 1) * 8;
        } else {
            front_mask >>= (8 - rank) * 8;
            adjacent_mask >>= (8 - rank) * 8;
        }

        let combined_mask = front_mask | adjacent_mask;
        (foe_pawns & combined_mask) == 0
    }

    fn evaluate_rooks(&self, friend_rooks: u64, friend_pawns: u64, foe_pawns: u64) -> (i32, i32) {
        let mut mg = 0;
        let mut eg = 0;

        let open_file_bonus = 20;
        let semi_open_file_bonus = 10;

        let mut temp_rooks = friend_rooks;
        while temp_rooks != 0 {
            let square = temp_rooks.trailing_zeros() as usize;
            let file = square % 8;

            // Check for open file
            if (friend_pawns | foe_pawns) & FILE_MASKS[file] == 0 {
                mg += open_file_bonus;
                eg += open_file_bonus;
            }
            // Check for semi-open file
            else if friend_pawns & FILE_MASKS[file] == 0 {
                mg += semi_open_file_bonus;
                eg += semi_open_file_bonus;
            }

            temp_rooks &= temp_rooks - 1;
        }

        (mg, eg)
    }

    fn ply(&self) -> u8 {
        self.position_history.len() as u8
    }
}

const FILE_MASKS: [u64; 8] = [
    0x0101010101010101,
    0x0202020202020202,
    0x0404040404040404,
    0x0808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];

const ADJACENT_FILES_MASKS: [u64; 8] = [
    FILE_MASKS[1],
    FILE_MASKS[0] | FILE_MASKS[2],
    FILE_MASKS[1] | FILE_MASKS[3],
    FILE_MASKS[2] | FILE_MASKS[4],
    FILE_MASKS[3] | FILE_MASKS[5],
    FILE_MASKS[4] | FILE_MASKS[6],
    FILE_MASKS[5] | FILE_MASKS[7],
    FILE_MASKS[6],
];
