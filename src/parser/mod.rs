use crate::bitboard::Piece;

pub fn parse_move(input: &str) -> Option<(usize, usize, Option<Piece>)> {
    if input.len() != 4 && input.len() != 5 {
        return None;
    }

    let from_file = input.chars().next()?;
    let from_rank = input.chars().nth(1)?;
    let to_file = input.chars().nth(2)?;
    let to_rank = input.chars().nth(3)?;

    let from_file_idx = (from_file as u8).checked_sub(b'a')? as usize;
    let from_rank_idx = (from_rank as u8).checked_sub(b'1')? as usize;
    let to_file_idx = (to_file as u8).checked_sub(b'a')? as usize;
    let to_rank_idx = (to_rank as u8).checked_sub(b'1')? as usize;

    if from_file_idx > 7 || from_rank_idx > 7 || to_file_idx > 7 || to_rank_idx > 7 {
        return None;
    }

    let from_square = from_rank_idx * 8 + from_file_idx;
    let to_square = to_rank_idx * 8 + to_file_idx;

    let promotion = if input.len() == 5 {
        match input.chars().nth(4)? {
            'q' => Some(Piece::Queen),
            'r' => Some(Piece::Rook),
            'b' => Some(Piece::Bishop),
            'n' => Some(Piece::Knight),
            _ => None,
        }
    } else {
        None
    };

    Some((from_square, to_square, promotion))
}