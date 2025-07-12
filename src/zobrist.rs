use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::game::ZOBRIST_KEYS;
use crate::Game;

pub const NUM_PIECE_TYPES: usize = 6; // Pawn, Knight, Bishop, Rook, Queen, King
pub const NUM_COLORS: usize = 2;
pub const NUM_SQUARES: usize = 64;

// We will create a static table: piece_keys[color][piece][square]
pub struct ZobristKeys {
    pub piece_keys: [[[u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_COLORS],
    pub castling_keys: [u64; 16],    // castling rights bitmask 0-15
    pub en_passent_keys: [u64; 8],   // en passant file 0-7
    pub side_to_move_key: u64,
}

impl ZobristKeys {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(0x12345678abcdef); // fixed seed for reproducibility

        let mut piece_keys = [[[0u64; NUM_SQUARES]; NUM_PIECE_TYPES]; NUM_COLORS];
        for color in 0..NUM_COLORS {
            for piece in 0..NUM_PIECE_TYPES {
                for sq in 0..NUM_SQUARES {
                    piece_keys[color][piece][sq] = rng.random();
                }
            }
        }

        let mut castling_keys = [0u64; 16];
        for i in 0..16 {
            castling_keys[i] = rng.random();
        }

        let mut en_passent_keys = [0u64; 8];
        for i in 0..8 {
            en_passent_keys[i] = rng.random();
        }

        let side_to_move_key = rng.random();

        Self {
            piece_keys,
            castling_keys,
            en_passent_keys,
            side_to_move_key,
        }
    }
}

impl Game {
    pub fn compute_zobrist_hash(&self) -> u64 {
        let mut hash = 0u64;

        // Map your Bitboard pieces to indices: Pawn=0, Knight=1, Bishop=2, Rook=3, Queen=4, King=5
        // Color: White=0, Black=1

        for sq in 0..64 {
            let mask = 1u64 << sq;

            let color = if (self.board.white_pawns | self.board.white_knight | self.board.white_bishop
                | self.board.white_rook | self.board.white_queen | self.board.white_king) & mask != 0
            {
                0
            } else if (self.board.black_pawns | self.board.black_knight | self.board.black_bishop
                | self.board.black_rook | self.board.black_queen | self.board.black_king) & mask != 0
            {
                1
            } else {
                continue;
            };

            let piece_type = if (self.board.white_pawns | self.board.black_pawns) & mask != 0 {
                0
            } else if (self.board.white_knight | self.board.black_knight) & mask != 0 {
                1
            } else if (self.board.white_bishop | self.board.black_bishop) & mask != 0 {
                2
            } else if (self.board.white_rook | self.board.black_rook) & mask != 0 {
                3
            } else if (self.board.white_queen | self.board.black_queen) & mask != 0 {
                4
            } else if (self.board.white_king | self.board.black_king) & mask != 0 {
                5
            } else {
                continue;
            };

            hash ^= ZOBRIST_KEYS.piece_keys[color][piece_type][sq];
        }

        // Castling rights (4 bits? You store as u8, just use lower 4 bits)
        let castling_index = (self.castling & 0x0F) as usize;
        hash ^= ZOBRIST_KEYS.castling_keys[castling_index];

        // En passant file
        if let Some(ep_square) = self.en_passent {
            let file = ep_square % 8;
            hash ^= ZOBRIST_KEYS.en_passent_keys[file];
        }

        // Side to move
        if self.is_white_turn {
            hash ^= ZOBRIST_KEYS.side_to_move_key;
        }

        hash
    }
}