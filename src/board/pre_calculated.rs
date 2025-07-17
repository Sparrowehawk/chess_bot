pub mod bishop_magics;
pub mod rook_magics;

// Precaluated knight and king on compile time
// Sliders are compiled and calculated pre compile into magic bitmaps

pub const KNIGHT_ATTACKS: [u64; 64] = precalculate_knight_attacks();
pub const KING_ATTACKS: [u64; 64] = precalculate_king_attacks();
pub const PAWN_ATTACKS: [[u64; 64]; 2] = precalculate_pawn_attacks();
pub const PAWN_PUSHES: [[u64; 64]; 2] = precalculate_pawn_pushes();

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;
const RANK_2: u64 = 0x000000000000FF00;
const RANK_7: u64 = 0x00FF000000000000;

const fn shift(bitboard: u64, shift: i8) -> u64 {
    if shift > 0 {
        bitboard << shift
    } else {
        bitboard >> -shift
    }
}

const fn precalculate_pawn_attacks() -> [[u64; 64]; 2] {
    let mut attacks = [[0u64; 64]; 2];
    let mut i = 0;
    while i < 64 {
        let square = 1u64 << i;

        // White captures
        let left = shift(square & !FILE_A, 7);
        let right = shift(square & !FILE_H, 9);
        attacks[0][i] = left | right;

        // Black captures
        let left = shift(square & !FILE_A, -9);
        let right = shift(square & !FILE_H, -7);
        attacks[1][i] = left | right;

        i += 1;
    }
    attacks
}

const fn precalculate_pawn_pushes() -> [[u64; 64]; 2] {
    let mut pushes = [[0u64; 64]; 2];
    let mut i = 0;
    while i < 64 {
        let square = 1u64 << i;

        // White pushes
        pushes[0][i] = shift(square, 8);
        if square & RANK_2 != 0 {
            pushes[0][i] |= shift(square, 16);
        }

        // Black pushes
        pushes[1][i] = shift(square, -8);
        if square & RANK_7 != 0 {
            pushes[1][i] |= shift(square, -16);
        }

        i += 1;
    }
    pushes
}

const fn precalculate_knight_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        let mut attack_map = 0u64;
        let pos = 1u64 << i;

        // Up 2, Right 1 (North-North-East)
        if (pos << 17) & !FILE_A != 0 {
            attack_map |= pos << 17;
        }
        // Up 2, Left 1 (North-North-West)
        if (pos << 15) & !FILE_H != 0 {
            attack_map |= pos << 15;
        }
        // Up 1, Right 2 (North-East-East)
        if (pos << 10) & !FILE_A & !FILE_B != 0 {
            attack_map |= pos << 10;
        }
        // Up 1, Left 2 (North-West-West)
        if (pos << 6) & !FILE_H & !FILE_G != 0 {
            attack_map |= pos << 6;
        }

        // Down 2, Right 1 (South-South-East)
        if (pos >> 15) & !FILE_A != 0 {
            attack_map |= pos >> 15;
        }
        // Down 2, Left 1 (South-South-West)
        if (pos >> 17) & !FILE_H != 0 {
            attack_map |= pos >> 17;
        }
        // Down 1, Right 2 (South-East-East)
        if (pos >> 6) & !FILE_A & !FILE_B != 0 {
            attack_map |= pos >> 6;
        }
        // Down 1, Left 2 (South-West-West)
        if (pos >> 10) & !FILE_H & !FILE_G != 0 {
            attack_map |= pos >> 10;
        }

        attacks[i] = attack_map;
        i += 1;
    }
    attacks
}

const fn precalculate_king_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        let mut attack_map = 0u64;
        let pos = 1u64 << i;

        // North
        attack_map |= pos << 8;
        // South
        attack_map |= pos >> 8;

        // East (prevent wrap)
        if (pos & !FILE_H) != 0 {
            attack_map |= pos << 1; // East
            attack_map |= pos << 9; // North-East
            attack_map |= pos >> 7; // South-East
        }
        // West (prevent wrap)
        if (pos & !FILE_A) != 0 {
            attack_map |= pos >> 1; // West
            attack_map |= pos << 7; // North-West
            attack_map |= pos >> 9; // South-West
        }

        attacks[i] = attack_map;
        i += 1;
    }
    attacks
}

