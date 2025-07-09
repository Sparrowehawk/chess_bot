pub mod bitboard;
pub mod parser;
pub mod game;
pub mod perft;
pub mod legal_moves;
pub mod const_moves;
pub mod fen;

// pub use parser::Parser;
pub use bitboard::Bitboard;
pub use game::Game;