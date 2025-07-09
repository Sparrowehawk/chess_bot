use crate::game::Game;
use crate::bitboard::Piece;

impl Game {
    pub fn perft(&mut self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        let moves = self.generate_legal_moves();
        let mut nodes = 0;

        for (from, to, promo) in moves {
            let mut new_game = self.clone();
            if new_game.make_move(from, to, promo) {
                nodes += new_game.perft(depth - 1);
            }
        }

        nodes
    }

    pub fn perft_divide(&mut self, depth: u32) {
        if depth == 0 {
            println!("Total nodes at depth {depth} : 0");
            return;
        }

        let moves = self.generate_legal_moves();
        let mut total_nodes = 0;

        for (from, to, promo) in moves {
            let mut new_game = self.clone();
            if new_game.make_move(from, to, promo) {
                let nodes = new_game.perft(depth - 1);
                let move_str = format!(
                    "{}{}{}",
                    self.square_index_to_coord(from),
                    self.square_index_to_coord(to),
                    self.promo_to_char(promo)
                );
                println!("{move_str}: {nodes}");
                total_nodes += nodes;
            }
        }

        println!("Total nodes at depth {depth}: {total_nodes}");
    }

    pub fn square_index_to_coord(&self, index: usize) -> String {
        let file = (b'a' + (index % 8) as u8) as char;
        let rank = (index / 8) + 1;
        format!("{file}{rank}")
    }

    pub fn promo_to_char(&self, promo: Option<Piece>) -> &'static str {
        match promo {
            Some(Piece::Queen) => "q",
            Some(Piece::Rook) => "r",
            Some(Piece::Bishop) => "b",
            Some(Piece::Knight) => "n",
            _ => "",
        }
    }

    pub fn perft_debug(&mut self, depth: u32, history: &mut Vec<String>) -> u64 {
        if depth == 0 {
            // When we hit a leaf node, print the move history that got us here.
            println!("{}", history.join(" "));
            return 1;
        }
    
        let moves = self.generate_legal_moves();
        let mut nodes = 0;
    
        for (from, to, promo) in moves {
            let mut new_game = self.clone();
            if new_game.make_move(from, to, promo) {
                // Add the current move to the history before recursing
                let move_str = format!(
                    "{}{}{}",
                    self.square_index_to_coord(from),
                    self.square_index_to_coord(to),
                    self.promo_to_char(promo)
                );
                history.push(move_str);
    
                nodes += new_game.perft_debug(depth - 1, history);
    
                // IMPORTANT: Remove the move from history after the recursion returns.
                history.pop();
            }
        }
        nodes
    }
    

}
