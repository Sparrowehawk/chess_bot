#[cfg(test)]
mod tests {
    use chess_bot::bitboard::Bitboard;

    fn setup_board(rook_pos: usize, opponent_pos: Option<usize>) -> Bitboard {
        let mut board = Bitboard {
            white_king: 0,
            white_queen: 0,
            white_rook: 0,
            white_bishop: 0,
            white_knight: 0,
            white_pawns: 0,
            black_king: 0,
            black_queen: 0,
            black_rook: 0,
            black_bishop: 0,
            black_knight: 0,
            black_pawns: 0,
        };

        board.white_rook |= 1u64 << rook_pos;

        if let Some(pos) = opponent_pos {
            board.black_pawns |= 1u64 << pos;
        }
        board
    }

    // --- VALID MOVES (PUSHES) ---

    #[test]
    fn test_rook_move_horizontal_empty_board() {
        let mut board = setup_board(27, None); // Rook at d4
        assert!(board.move_rook(27, 31, true)); // Move to h4
        assert_eq!(board.white_rook, 1u64 << 31);
        assert_eq!(board.all_pieces(), 1u64 << 31);
    }

    #[test]
    fn test_rook_move_vertical_empty_board() {
        let mut board = setup_board(27, None); // Rook at d4
        assert!(board.move_rook(27, 3, true)); // Move to d1
        assert_eq!(board.white_rook, 1u64 << 3);
    }

    #[test]
    fn test_black_rook_move_vertical() {
        // Set up a custom board instead of the default one
        let mut board = Bitboard {
            white_king: 0,
            white_queen: 0,
            white_rook: 0,
            white_bishop: 0,
            white_knight: 0,
            white_pawns: 0,
            black_king: 0,
            black_queen: 0,
            black_rook: 1u64 << 63, // Black rook at h8
            black_bishop: 0,
            black_knight: 0,
            black_pawns: 0,
        };

        // Now the move is on an empty path and should succeed
        assert!(board.move_rook(63, 47, false));
        assert_eq!(board.black_rook, 1u64 << 47);
    }

    // --- VALID CAPTURES ---

    #[test]
    fn test_rook_capture_horizontal() {
        let mut board = setup_board(27, Some(31)); // White rook at d4, black pawn at h4
        assert!(board.move_rook(27, 31, true));
        assert_eq!(board.white_rook, 1u64 << 31); // Rook moved
        assert_eq!(board.black_pawns, 0); // Pawn was captured
        assert_eq!(board.all_pieces(), 1u64 << 31);
    }

    #[test]
    fn test_rook_capture_vertical() {
        let mut board = setup_board(27, Some(59)); // White rook at d4, black pawn at d8
        assert!(board.move_rook(27, 59, true));
        assert_eq!(board.white_rook, 1u64 << 59);
        assert_eq!(board.black_pawns, 0);
    }

    // --- INVALID MOVES ---

    #[test]
    fn test_rook_invalid_diagonal_move() {
        let mut board = setup_board(27, None); // Rook at d4
        assert!(!board.move_rook(27, 36, true)); // Attempt to move to e5
        assert_eq!(board.white_rook, 1u64 << 27); // Rook has not moved
    }

    #[test]
    fn test_rook_move_to_friendly_piece() {
        let mut board = setup_board(27, None);
        board.white_pawns |= 1u64 << 29; // Friendly pawn at f4
        assert!(!board.move_rook(27, 29, true)); // Attempt to move to f4
        assert_eq!(board.white_rook, 1u64 << 27); // Rook has not moved
    }

    #[test]
    fn test_move_non_existent_rook() {
        let mut board = setup_board(27, None); // Rook at d4
        assert!(!board.move_rook(0, 8, true)); // No rook at a1
    }

    // --- BLOCKED PATHS ---

    #[test]
    fn test_rook_path_blocked_horizontal_by_opponent() {
        let mut board = setup_board(24, Some(27)); // Rook at a4, opponent at d4
        assert!(!board.move_rook(24, 31, true)); // Attempt to move to h4 (blocked)
    }

    #[test]
    fn test_rook_path_blocked_vertical_by_friendly() {
        let mut board = setup_board(4, None); // Rook at e1
        board.white_pawns |= 1u64 << 20; // Friendly pawn at e3
        assert!(!board.move_rook(4, 36, true)); // Attempt to move to e5 (blocked)
    }

    #[test]
    fn test_rook_path_blocked_at_destination_by_friendly() {
        let mut board = setup_board(0, None); // Rook at a1
        board.white_pawns |= 1u64 << 56; // Friendly pawn at a8
        assert!(!board.move_rook(0, 56, true)); // Attempt to move to a8
    }

    #[test]
    fn test_rook_path_clear_for_capture() {
        // Path is clear right up to the capture square
        let mut board = setup_board(24, Some(31)); // Rook a4, opponent h4
        assert!(board.move_rook(24, 31, true));
        assert_eq!(board.white_rook, 1u64 << 31);
    }
}
