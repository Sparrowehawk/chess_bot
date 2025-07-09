use crate::Bitboard;

const KNIGHT_ATTACKS: [u64; 64] = precalculate_knight_attacks();
const KING_ATTACKS: [u64; 64] = precalculate_king_attacks();

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

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

impl Bitboard {
    pub fn get_knight_attacks(&self, from: usize) -> u64 {
        KNIGHT_ATTACKS[from]
    }

    pub fn get_king_attacks(&self, from: usize) -> u64 {
        KING_ATTACKS[from]
    }
}
