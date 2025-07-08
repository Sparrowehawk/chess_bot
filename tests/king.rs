use chess_bot::bitboard::Bitboard;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a new board for testing
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
    fn test_move_king_forward() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 12; // e2
        board.white_king = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_king(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_king, 1u64 << to);
        assert_eq!(board.all_pieces() & (1u64 << from), 0);
    }

    #[test]
    fn test_move_king_diagonal() {
        let mut board = setup_board();
        let from = 28; // e4
        let to = 35; // d5
        board.white_king = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_king(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_king, 1u64 << to);
    }
    
    #[test]
    fn test_move_king_sideways() {
        let mut board = setup_board();
        let from = 28; // e4
        let to = 27; // d4
        board.white_king = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_king(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_king, 1u64 << to);
    }

    #[test]
    fn test_invalid_move_too_far() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 20; // e3
        board.white_king = 1u64 << from;
        let mut castling = 0b1111;

        let initial_king_pos = board.white_king;
        let result = board.move_king(from, to, true, &mut castling);

        assert!(!result);
        assert_eq!(board.white_king, initial_king_pos);
    }

    #[test]
    fn test_invalid_move_to_friendly_piece() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 12; // e2
        board.white_king = 1u64 << from;
        board.white_pawns = 1u64 << to;
        let mut castling = 0b1111;

        let initial_king_pos = board.white_king;
        let result = board.move_king(from, to, true, &mut castling);

        assert!(!result);
        assert_eq!(board.white_king, initial_king_pos);
        assert_eq!(board.white_pawns, 1u64 << to);
    }

    #[test]
    fn test_valid_capture() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 12; // e2
        board.white_king = 1u64 << from;
        board.black_pawns = 1u64 << to;
        let mut castling = 0b1111;

        let result = board.move_king(from, to, true, &mut castling);

        assert!(result);
        assert_eq!(board.white_king, 1u64 << to);
        assert_eq!(board.black_pawns, 0); // Pawn should be captured
    }

    #[test]
    fn test_move_black_king() {
        let mut board = setup_board();
        let from = 60; // e8
        let to = 52; // e7
        board.black_king = 1u64 << from;
        let mut castling = 0b1111;

        let result = board.move_king(from, to, false, &mut castling);

        assert!(result);
        assert_eq!(board.black_king, 1u64 << to);
        assert_eq!(board.all_pieces() & (1u64 << from), 0);
    }

    #[test]
    fn test_move_white_king_removes_castling_rights() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 5;   // f1
        board.white_king = 1u64 << from;
        // All castling rights are initially available
        let mut castling = 0b1111;

        board.move_king(from, to, true, &mut castling);

        // White's castling rights (queen-side and king-side) should be removed.
        // Binary: 0b0011
        assert_eq!(castling, 3); 
    }

    #[test]
    fn test_move_black_king_removes_castling_rights() {
        let mut board = setup_board();
        let from = 60; // e8
        let to = 59; // d8
        board.black_king = 1u64 << from;
        // All castling rights are initially available
        let mut castling = 0b1111;

        board.move_king(from, to, false, &mut castling);
        
        // Black's castling rights (queen-side and king-side) should be removed.
        // Binary: 0b1100
        assert_eq!(castling, 12);
    }
    
    #[test]
    fn test_move_from_empty_square() {
        let mut board = setup_board();
        let from = 28; // e4
        let to = 36;   // e5
        // No king is placed on the 'from' square
        let mut castling = 0b1111;

        let result = board.move_king(from, to, true, &mut castling);
        
        assert!(!result);
        assert_eq!(board.white_king, 0); // Board remains unchanged
    }
}
