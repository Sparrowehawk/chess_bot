use chess_bot::bitboard::Bitboard;


#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a clean board for each test
    fn setup_board() -> Bitboard {
        Bitboard {
            white_queen: 0,
            white_rook: 0,
            white_bishop: 0,
            white_knight: 0,
            black_queen: 0,
            black_rook: 0,
            black_bishop: 0,
            black_knight: 0,
            white_king: 0,
            white_pawns: 0,
            black_king: 0,
            black_pawns: 0,
        }
    }

    #[test]
    fn test_valid_move_horizontal() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 5;   // f1
        board.white_rook = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_rook, 1u64 << to);
        assert_eq!(board.all_pieces() & (1u64 << from), 0);
    }

    #[test]
    fn test_valid_move_vertical() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 56;  // a8
        board.white_rook = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_rook, 1u64 << to);
    }

    #[test]
    fn test_valid_capture() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 56;  // a8
        board.white_rook = 1u64 << from;
        board.black_pawns = 1u64 << to;
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_rook, 1u64 << to);
        assert_eq!(board.black_pawns, 0); // Opponent pawn should be gone
    }
    
    #[test]
    fn test_black_rook_move() {
        let mut board = setup_board();
        let from = 63; // h8
        let to = 56;   // a8
        board.black_rook = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, false, &mut castling);

        assert!(result);
        assert_eq!(board.black_rook, 1u64 << to);
    }

    #[test]
    fn test_invalid_move_diagonal() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 9;   // b2
        board.white_rook = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(!result);
        assert_eq!(board.white_rook, 1u64 << from); // Position should not change
    }

    #[test]
    fn test_invalid_move_through_piece_horizontal() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 7;   // h1
        board.white_rook = 1u64 << from;
        board.white_pawns = 1u64 << 3; // Place a friendly pawn at d1
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(!result);
        assert_eq!(board.white_rook, 1u64 << from);
    }

    #[test]
    fn test_invalid_move_through_piece_vertical() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 56;  // a8
        board.white_rook = 1u64 << from;
        board.black_pawns = 1u64 << 24; // Place an enemy pawn at a4
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(!result);
        assert_eq!(board.white_rook, 1u64 << from);
    }

    #[test]
    fn test_invalid_move_to_friendly_piece() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 7;   // h1
        board.white_rook = (1u64 << from) | (1u64 << to); // Two white rooks
        let mut castling = 0b1111;

        let result = board.move_rook(from, to, true, &mut castling);

        assert!(!result);
        assert_eq!(board.white_rook, (1u64 << from) | (1u64 << to));
    }

    #[test]
    fn test_move_white_queenside_rook_removes_castling_right() {
        let mut board = setup_board();
        let from = 0; // a1
        let to = 1;   // b1
        board.white_rook = 1u64 << from;
        let mut castling = 0b1111; // All rights available

        board.move_rook(from, to, true, &mut castling);

        // White queen-side right (bit 2) should be removed. 0b1111 -> 0b1011
        assert_eq!(castling, 0b1011);
    }
    
    #[test]
    fn test_move_white_kingside_rook_removes_castling_right() {
        let mut board = setup_board();
        let from = 7; // h1
        let to = 6;   // g1
        board.white_rook = 1u64 << from;
        let mut castling = 0b1111; // All rights available

        board.move_rook(from, to, true, &mut castling);

        // White king-side right (bit 3) should be removed. 0b1111 -> 0b0111
        assert_eq!(castling, 0b0111);
    }

    #[test]
    fn test_move_black_queenside_rook_removes_castling_right() {
        let mut board = setup_board();
        let from = 56; // a8
        let to = 57;   // b8
        board.black_rook = 1u64 << from;
        let mut castling = 0b1111; // All rights available

        board.move_rook(from, to, false, &mut castling);

        // Black queen-side right (bit 0) should be removed. 0b1111 -> 0b1110
        assert_eq!(castling, 0b1110);
    }

    #[test]
    fn test_move_black_kingside_rook_removes_castling_right() {
        let mut board = setup_board();
        let from = 63; // h8
        let to = 62;   // g8
        board.black_rook = 1u64 << from;
        let mut castling = 0b1111; // All rights available

        board.move_rook(from, to, false, &mut castling);

        // Black king-side right (bit 1) should be removed. 0b1111 -> 0b1101
        assert_eq!(castling, 0b1101);
    }

    #[test]
    fn test_move_non_castling_rook_does_not_affect_rights() {
        let mut board = setup_board();
        board.white_rook = 1u64 << 27; // Rook at d4
        let mut castling = 0b1111;

        board.move_rook(27, 35, true, &mut castling); // d4 to d5

        // Castling rights should be unchanged
        assert_eq!(castling, 0b1111);
    }
}
