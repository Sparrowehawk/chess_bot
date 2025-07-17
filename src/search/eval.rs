use super::see;
use super::tt::Flag;
use crate::game::Game;
use crate::search::pst::{
    ADJACENT_FILES_MASKS, EG_TABLE, FILE_MASKS, MG_TABLE, PASSED_BLACK_MASKS, PASSED_WHITE_MASKS,
    get_piece_at,
};
use crate::search::{MAX_PHASE, PHASE_WEIGHTS, PIECE_VALUES, Search, TEMPO_BONUS, pst};
use crate::{Bitboard, Piece}; 

pub fn quiescence_search(
    game: &mut Game,
    mut alpha: i32,
    beta: i32,
    search_helper: &mut Search,
) -> i32 {
    search_helper.nodes_searched += 1;
    let tt_entry = game.tt.lock().unwrap().probe(game.zobrist_hash);
    let stand_pat = if let Some(entry) = tt_entry {
        if entry.flag == Flag::Exact {
            entry.score
        } else {
            eval(game) // Fallback to fresh evaluation
        }
    } else {
        eval(game) // No TT entry, so evaluate from scratch
    };

    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let enemy_pieces = if game.is_white_turn {
        game.board.black_pieces()
    } else {
        game.board.white_pieces()
    };
    let mut moves = game.generate_legal_moves();
    moves.retain(|&(_, to, promo)| ((1u64 << to) & enemy_pieces != 0) || promo.is_some());

    let ply = pst::ply(game) as usize;
    moves.sort_by_cached_key(|m| -(see::score_move(game, *m, ply, search_helper)));

    for m in moves.iter() {
        if see::static_exchange_exchange(game, m.0, m.1) < 0 {
            continue;
        }

        let undo = game.make_move_unchecked(m.0, m.1, m.2);
        let score = -quiescence_search(game, -beta, -alpha, search_helper);
        game.unmake_move(undo);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

pub fn eval(game: &Game) -> i32 {
    let (white_mg, white_eg, white_phase) = calculate_score(game, true);
    let (black_mg, black_eg, black_phase) = calculate_score(game, false);

    let total_phase = (white_phase + black_phase).clamp(0, MAX_PHASE);
    let mg_score = white_mg - black_mg;
    let eg_score = white_eg - black_eg;

    let blended_score = (mg_score * total_phase + eg_score * (MAX_PHASE - total_phase)) / MAX_PHASE;

    let perspective = if game.is_white_turn { 1 } else { -1 };
    (blended_score + TEMPO_BONUS) * perspective
}

fn calculate_score(game: &Game, is_white: bool) -> (i32, i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;
    let mut phase_score = 0;

    let board = &game.board;

    let (friend_pawns, friend_knights, friend_bishops, friend_rooks, friend_queens, friend_king) =
        if is_white {
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

    for (piece_idx, &bb) in friend_bitboards.iter().enumerate() {
        let piece_count = bb.count_ones() as i32;

        mg_score += piece_count * PIECE_VALUES[piece_idx];
        eg_score += piece_count * PIECE_VALUES[piece_idx];
        phase_score += piece_count * PHASE_WEIGHTS[piece_idx];

        let mut temp_bb = bb;
        while temp_bb != 0 {
            let square = temp_bb.trailing_zeros() as usize;
            let pst_idx = if is_white { square ^ 56 } else { square };

            mg_score += MG_TABLE[piece_idx][pst_idx];
            eg_score += EG_TABLE[piece_idx][pst_idx];

            temp_bb &= temp_bb - 1; // Clear the least significant bit
        }
    }

    if friend_bishops.count_ones() >= 2 {
        mg_score += 30; 
        eg_score += 50; 
    }

    let (pawn_mg, pawn_eg) = evaluate_pawn_structure(game, friend_pawns, is_white);
    mg_score += pawn_mg;
    eg_score += pawn_eg;

    let (rook_mg, rook_eg) = evaluate_rooks(game, friend_rooks, friend_pawns, foe_pawns);
    mg_score += rook_mg;
    eg_score += rook_eg;

    let (threats_mg, threats_eg) = evaluate_threats(game, friend_pawns, is_white);
    mg_score += threats_mg;
    eg_score += threats_eg;

    let (hanging_mg, hanging_eg) = evaluate_hanging_pieces(game, is_white);
    mg_score += hanging_mg;
    eg_score += hanging_eg;

    let (pin_mg, pin_eg) = evaluate_pins(game, is_white);
    mg_score += pin_mg;
    eg_score += pin_eg;

    let (king_mg, king_eg) = evaluate_king_safety(game, is_white);
    mg_score += king_mg;
    eg_score += king_eg;

    (mg_score, eg_score, phase_score)
}

fn evaluate_pawn_structure(game: &Game, friend_pawns: u64, is_white: bool) -> (i32, i32) {
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

        // Penalty for isolated pawns 
        if (friend_pawns & ADJACENT_FILES_MASKS[file]) == 0 {
            mg += -15;
            eg += -25;
        }

        // Bonus for passed pawns 
        if is_passed(game, square, is_white) {
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

fn is_passed(game: &Game, square: usize, is_white: bool) -> bool {
    let file = square % 8;
    let rank = square / 8;
    let foe_pawns = if is_white {
        game.board.black_pawns
    } else {
        game.board.white_pawns
    };

    let mask = if is_white {
        PASSED_WHITE_MASKS[file][rank]
    } else {
        PASSED_BLACK_MASKS[file][rank]
    };

    (foe_pawns & mask) == 0
}

fn evaluate_rooks(
    _game: &Game,
    friend_rooks: u64,
    friend_pawns: u64,
    foe_pawns: u64,
) -> (i32, i32) {
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

fn evaluate_threats(game: &Game, friend_pawns: u64, is_white: bool) -> (i32, i32) {
    let board = &game.board;

    let (foe_knights, foe_bishops, foe_rooks, foe_queens) = if is_white {
        (
            board.black_knight,
            board.black_bishop,
            board.black_rook,
            board.black_queen,
        )
    } else {
        (
            board.white_knight,
            board.white_bishop,
            board.white_rook,
            board.white_queen,
        )
    };

    let attacks = if is_white {
        (friend_pawns << 7 & !FILE_MASKS[7]) | (friend_pawns << 9 & !FILE_MASKS[0])
    } else {
        (friend_pawns >> 7 & !FILE_MASKS[0]) | (friend_pawns >> 9 & !FILE_MASKS[7])
    };

    let mut mg_score = 0;
    let mut eg_score = 0;

    // Heuristic threat bonuses for mg and eg
    let knight_threat = (15, 10);
    let bishop_threat = (20, 15);
    let rook_threat = (30, 25);
    let queen_threat = (40, 35);

    mg_score += knight_threat.0 * (attacks & foe_knights).count_ones() as i32;
    eg_score += knight_threat.1 * (attacks & foe_knights).count_ones() as i32;

    mg_score += bishop_threat.0 * (attacks & foe_bishops).count_ones() as i32;
    eg_score += bishop_threat.1 * (attacks & foe_bishops).count_ones() as i32;

    mg_score += rook_threat.0 * (attacks & foe_rooks).count_ones() as i32;
    eg_score += rook_threat.1 * (attacks & foe_rooks).count_ones() as i32;

    mg_score += queen_threat.0 * (attacks & foe_queens).count_ones() as i32;
    eg_score += queen_threat.1 * (attacks & foe_queens).count_ones() as i32;

    (mg_score, eg_score)
}

fn evaluate_hanging_pieces(game: &Game, is_white: bool) -> (i32, i32) {
    let mut mg_penalty = 0;
    let mut eg_penalty = 0;

    let friendly_pieces = if is_white {
        game.board.white_pieces()
    } else {
        game.board.black_pieces()
    };

    let mut temp_bb = friendly_pieces;
    while temp_bb != 0 {
        let sq = temp_bb.trailing_zeros() as usize;
        let opponent_attackers = attackers_to(&game.board, sq, !is_white);
        if opponent_attackers != 0 {
            // If it is attacked, check if it has any defenders
            let friendly_defenders = attackers_to(&game.board, sq, is_white);

            if friendly_defenders == 0 {
                // Piece is hanging, apply penalty
                if let Some(piece) = get_piece_at(game, sq) {

                    let penalty = match piece {
                        Piece::Knight => -50,
                        Piece::Bishop => -60,
                        Piece::Rook => -85,
                        Piece::Queen => -120,
                        _ => 0,
                    };
                    mg_penalty += penalty;
                    eg_penalty += penalty;
                }
            }
        }

        temp_bb &= temp_bb - 1; // Move to the next piece.
    }

    (mg_penalty, eg_penalty)
}

pub fn attackers_to(board: &Bitboard, square: usize, is_white: bool) -> u64 {
    let occupied = board.all_pieces();
    let (pawns, knights, bishops, rooks, queens, king) = if is_white {
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

    let mut attackers = 0;

    attackers |= pawns & Bitboard::get_pawn_attacks(is_white as usize, square);
    attackers |= knights & board.get_knight_attacks(square);
    attackers |= king & board.get_king_attacks(square);
    attackers |= bishops & Bitboard::get_bishop_attacks(square, occupied);
    attackers |= rooks & Bitboard::get_rook_attacks(square, occupied);
    attackers |= queens
        & (Bitboard::get_rook_attacks(square, occupied) | Bitboard::get_bishop_attacks(square, occupied));

    attackers
}

fn attackers_to_with_occupied(board: &Bitboard, square: usize, is_white: bool, occupied: u64) -> u64 {
    let (pawns, knights, bishops, rooks, queens, king) = if is_white {
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

    let mut attackers = 0;

    attackers |= pawns & Bitboard::get_pawn_attacks(is_white as usize, square);
    attackers |= knights & board.get_knight_attacks(square);
    attackers |= king & board.get_king_attacks(square);
    attackers |= bishops & Bitboard::get_bishop_attacks(square, occupied);
    attackers |= rooks & Bitboard::get_rook_attacks(square, occupied);
    attackers |= queens
        & (Bitboard::get_rook_attacks(square, occupied) | Bitboard::get_bishop_attacks(square, occupied));

    attackers
}

fn evaluate_pins(game: &Game, is_white: bool) -> (i32, i32) {
    let mut mg_penalty = 0;
    let mut eg_penalty = 0;

    let (friendly_pieces, king_sq) = if is_white {
        (
            game.board.white_pieces(),
            game.board.white_king.trailing_zeros() as usize,
        )
    } else {
        (
            game.board.black_pieces(),
            game.board.black_king.trailing_zeros() as usize,
        )
    };

    // Identify sliding pieces
    let foe_sliders = if is_white {
        game.board.black_rook | game.board.black_bishop | game.board.black_queen
    } else {
        game.board.white_rook | game.board.white_bishop | game.board.white_queen
    };

    let occupied = game.board.all_pieces();

    // Iterate through each friendly piece (excluding the king)
    let mut potential_pinned = friendly_pieces & !(1u64 << king_sq);
    while potential_pinned != 0 {
        let piece_sq = potential_pinned.trailing_zeros() as usize;

        // Create a hypothetical board with this piece removed and find attackers
        let occupied_without_piece = occupied & !(1u64 << piece_sq);
        let attackers_to_king =
            attackers_to_with_occupied(&game.board, king_sq, !is_white, occupied_without_piece);

        // Is attacked by sliding piece?
        if (attackers_to_king & foe_sliders) != 0 {
            mg_penalty -= 25;
            eg_penalty -= 25;
        }

        potential_pinned &= potential_pinned - 1; // Move to the next piece
    }

    (mg_penalty, eg_penalty)
}

fn evaluate_king_safety(game: &Game, is_white: bool) -> (i32, i32) {
    let king_sq = if is_white {
        game.board.white_king.trailing_zeros() as usize
    } else {
        game.board.black_king.trailing_zeros() as usize
    };

    let king_file = king_sq % 8;
    let rank = king_sq / 8;

    // if king is on back rank, check pawn shield
    let shield_rank = if is_white { 1 } else { 6 };
    if rank != 0 && rank != 7 {
        return (0, 0); // king is in the center or midboard (not yet castled)
    }

    let shield = if is_white {
        game.board.white_pawns & (0b111 << (shield_rank * 8 + king_file.saturating_sub(1)))
    } else {
        game.board.black_pawns & (0b111 << (shield_rank * 8 + king_file.saturating_sub(1)))
    };

    let penalty = 20 * (3 - shield.count_ones() as i32); // Max 3 missing pawns
    (-penalty, -penalty)
}
