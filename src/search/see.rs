use crate::{game::Game, search::{eval, get_piece_value, pst, Search, KILLER_MOVE_SCORE, MAX_PLY, PIECE_VALUES}, Piece};

pub fn static_exchange_exchange(game: &Game, from: usize, to: usize) -> i32 {
    let mut gain = [0i32; 32];
    let mut depth = 0;

    let mut occupied = game.board.all_pieces();

    let Some(mut side) = Some(game.is_white_turn) else {
        return 0;
    };

    let Some(attacked_piece) = pst::get_piece_at(game, to) else {
        return 0;
    };
    gain[0] = PIECE_VALUES[attacked_piece as usize];

    let from_mask = 1u64 << from;
    occupied &= !from_mask; // remove attacker from occupancy

    let mut used_attackers = from_mask;

    side = !side; // after initial capture, switch side

    // simulate exchanges
    loop {
        let current_attackers = eval::attackers_to(&game.board ,to, side) & occupied & !used_attackers;

        let Some(sq) = least_valuable_piece(game, current_attackers, side) else {
            break;
        };

        let piece = pst::get_piece_at(game, sq).unwrap();
        depth += 1;

        gain[depth] = PIECE_VALUES[piece as usize] - gain[depth - 1];

        occupied &= !(1u64 << sq);
        used_attackers |= 1u64 << sq;

        side = !side;
    }

    // back-propagate minimax values
    while depth > 0 {
        depth -= 1;
        gain[depth] = -gain[depth + 1].max(-gain[depth]);
    }

    gain[0]
}

fn least_valuable_piece(game: &Game, attackers: u64, is_white: bool) -> Option<usize> {
    let board = &game.board;
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
    ];

    for (_piece, bb) in piece_order.iter() {
        let masked = attackers & bb;
        if masked != 0 {
            return Some(masked.trailing_zeros() as usize);
        }
    }

    None
}

pub fn score_move(
    game: &Game,
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

    let enemy_pieces = if game.is_white_turn {
        game.board.black_pieces()
    } else {
        game.board.white_pieces()
    };

    if (1u64 << to) & enemy_pieces != 0 {
        let attacker = pst::get_piece_at(game, from).unwrap_or(Piece::Pawn);
        let victim = pst::get_piece_at(game, to).unwrap_or(Piece::Pawn);
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

    if let Some(piece) = pst::get_piece_at(game, from) {
        return search_helper.history[piece as usize][to];
    }

    0
}
