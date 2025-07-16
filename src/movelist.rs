use crate::bitboard::Piece;
use std::slice::Iter;

#[derive(Clone, Debug)]
pub struct MoveList {
    moves: Vec<(usize, usize, Option<Piece>)>,
}

impl Default for MoveList {
    fn default() -> Self {
        MoveList {
            moves: Vec::with_capacity(256),
        }
    }
}

impl MoveList {
    /// Creates a new move list with a pre-allocated capacity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a move to the list.
    pub fn add(&mut self, from: usize, to: usize, promo: Option<Piece>) {
        self.moves.push((from, to, promo));
    }

    /// Clears the move list without de-allocating the buffer.
    pub fn clear(&mut self) {
        self.moves.clear();
    }
    
    /// Returns the number of moves in the list.
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Provides an iterator over the moves.
    pub fn iter(&self) -> Iter<'_, (usize, usize, Option<Piece>)> {
        self.moves.iter()
    }

    /// Allows sorting the moves, needed for move ordering in search.
    pub fn sort_by_cached_key<F, K>(&mut self, f: F)
    where
        F: FnMut(&(usize, usize, Option<Piece>)) -> K,
        K: Ord,
    {
        self.moves.sort_by_cached_key(f);
    }
    
    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&(usize, usize, Option<Piece>)) -> bool,
    {
        self.moves.retain(f);
    }
}
