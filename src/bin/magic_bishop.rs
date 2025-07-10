use rand::Rng;

// --- Helper Functions ---

/// Generates a bitboard mask for all squares that could block a bishop on a given square.
/// Excludes the edges of the board.
fn bishop_blocker_mask(square: usize) -> u64 {
    let mut result = 0u64;
    let r = (square / 8) as i8;
    let f = (square % 8) as i8;

    // Iterate outwards from the piece in all 4 diagonal directions
    for (dr, df) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let mut cr = r + dr;
        let mut cf = f + df;
        // Keep going until one square before the edge
        while cr > 0 && cr < 7 && cf > 0 && cf < 7 {
            result |= 1u64 << (cr * 8 + cf) as usize;
            cr += dr;
            cf += df;
        }
    }
    result
}

fn generate_blocker_permutations(mask: u64) -> Vec<u64> {
    let mut bits = vec![];
    for i in 0..64 {
        if (mask & (1u64 << i)) != 0 {
            bits.push(i);
        }
    }

    let mut result = vec![];
    let num_bits = bits.len();
    for i in 0..(1 << num_bits) {
        let mut b = 0;
        for (j, &bit) in bits.iter().enumerate() {
            if (i & (1 << j)) != 0 {
                b |= 1u64 << bit;
            }
        }
        result.push(b);
    }
    result
}

/// Calculates bishop attacks for a given blocker set.
fn calculate_bishop_attacks(square: usize, blockers: u64) -> u64 {
    let mut attack = 0u64;
    let r_start = (square / 8) as i8;
    let f_start = (square % 8) as i8;

    for (dr, df) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let mut r = r_start + dr;
        let mut f = f_start + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let idx = (r * 8 + f) as usize;
            attack |= 1u64 << idx;
            if (blockers & (1u64 << idx)) != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }
    attack
}

fn find_bishop_magic_and_table(square: usize) -> (u64, Vec<u64>, u8) {
    let mask = bishop_blocker_mask(square);
    let permutations = generate_blocker_permutations(mask);
    let num_relevant_bits = mask.count_ones() as u8;
    let shift = 64 - num_relevant_bits;
    let table_size = 1 << num_relevant_bits;

    let reference_attacks: Vec<u64> = permutations
        .iter()
        .map(|&blockers| calculate_bishop_attacks(square, blockers))
        .collect();

    let mut rng = rand::rng();

    for _ in 0..100_000_000 {
        let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();
        let mut attack_table = vec![0u64; table_size];
        let mut collision = false;

        for (i, &blockers) in permutations.iter().enumerate() {
            let index = (blockers.wrapping_mul(magic) >> shift) as usize;

            if attack_table[index] == 0 {
                attack_table[index] = reference_attacks[i];
            } else if attack_table[index] != reference_attacks[i] {
                collision = true;
                break;
            }
        }

        if !collision {
            return (magic, attack_table, num_relevant_bits);
        }
    }

    panic!("Failed to find magic number for square {square}");
}

// --- Main Data Generation Logic ---

struct MagicData {
    mask: u64,
    magic: u64,
    shift: u8,
    attack_table: Vec<u64>,
}

pub fn generate_bishop_data() {
    println!("// --- Bishop Magic Bitboard Data (Auto-Generated) ---");

    // generate and collect all data first
    let all_data: Vec<MagicData> = (0..64)
        .map(|sq| {
            let (magic, table, num_bits) = find_bishop_magic_and_table(sq);
            MagicData {
                mask: bishop_blocker_mask(sq),
                magic,
                shift: 64 - num_bits,
                attack_table: table,
            }
        })
        .collect();

    // calculate offsets and flatten the attack tables
    let mut offsets = [0; 64];
    let mut flat_attacks = Vec::new();
    let mut current_offset = 0;

    for i in 0..64 {
        offsets[i] = current_offset;
        let table = &all_data[i].attack_table;
        flat_attacks.extend_from_slice(table);
        current_offset += table.len();
    }

    // print all the constants

    println!("pub const BISHOP_MASKS: [u64; 64] = [");
    for (i, data) in all_data.iter().enumerate() {
        if i % 4 == 0 { print!("    "); }
        print!("0x{:016x}, ", data.mask);
        if (i + 1) % 4 == 0 { println!(); }
    }
    println!("];\n");

    // MAGICS
    println!("pub const BISHOP_MAGICS: [u64; 64] = [");
    for (i, data) in all_data.iter().enumerate() {
        if i % 4 == 0 { print!("    "); }
        print!("0x{:016x}, ", data.magic);
        if (i + 1) % 4 == 0 { println!(); }
    }
    println!("];\n");

    // SHIFTS
    println!("pub const BISHOP_SHIFTS: [u8; 64] = [");
    for (i, data) in all_data.iter().enumerate() {
        if i % 8 == 0 { print!("    "); }
        print!("{}, ", data.shift);
        if (i + 1) % 8 == 0 { println!(); }
    }
    println!("];\n");

    // OFFSETS
    println!("pub const BISHOP_OFFSETS: [usize; 64] = [");
    for (i, offset) in offsets.iter().enumerate() {
        if i % 8 == 0 { print!("    "); }
        print!("{offset:>4}, ");
        if (i + 1) % 8 == 0 { println!(); }
    }
    println!("];\n");

    // The fat ATTACKS table
    println!("pub static BISHOP_ATTACKS: [u64; {}] = [", flat_attacks.len());
    for (i, &attack) in flat_attacks.iter().enumerate() {
        if i % 4 == 0 { print!("    "); }
        print!("0x{attack:016x}, ");
        if (i + 1) % 4 == 0 { println!(); }
    }
    println!("];");
}

fn main() {
    println!("// This file is auto-generated. Do not edit manually.");
    println!("#![allow(clippy::all)]\n");
    generate_bishop_data();
}
