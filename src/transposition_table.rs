use crate::bitboard::Piece;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Flag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub score: i32, // Eval
    pub flag: Flag,
    pub best_move: Option<(usize, usize, Option<Piece>)>,
}

#[derive(Clone, Debug)]
pub struct TranspositionTable {
    table: HashMap<u64, TTEntry>,
    capacity: u64,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::with_capacity(2000000),
            capacity: 2000000,
        }
    }

    pub fn with_capacity(capacity: u64) -> Self {
        TranspositionTable {
            table: HashMap::with_capacity(capacity as usize),
            capacity,
        }
    }

    pub fn store(&mut self, key: u64, depth: u8, score: i32, flag: Flag, best_move: Option<(usize, usize, Option<Piece>)>) {
        let entry = TTEntry {
            key,
            depth,
            score,
            flag,
            best_move,
        };
        self.table.insert(key % self.capacity, entry);
    }

    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        if let Some(entry) = self.table.get(&(key % self.capacity)){
            if entry.key == key {
                return Some(*entry);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}
