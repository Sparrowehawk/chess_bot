use chess_bot::bitboard::Bitboard;

#[cfg(test)]
mod promotion_tests {
    use chess_bot::bitboard::Piece;

    use super::*;

    // Helper function to create a new, empty board for testing.
    fn setup_board() -> Bitboard {
        Bitboard {
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
        }
    }

    // --- White Pawn Promotion Tests ---

    #[test]
    fn test_white_pawn_promo_by_push_to_queen() {
        let mut board = setup_board();
        let from = 52; // e7
        let to = 60; // e8
        board.white_pawns = 1u64 << from;
        let promo = Some(Piece::Queen);

        let result = board.move_pawn(from, to, true, promo, None, &mut None);

        assert!(result, "Promotion by push should be a valid move.");
        assert_eq!(board.white_pawns, 0, "Original pawn should be removed.");
        assert_eq!(
            board.white_queen,
            1u64 << to,
            "A new white queen should be on the target square."
        );
    }

    #[test]
    fn test_white_pawn_promo_by_capture_to_knight() {
        let mut board = setup_board();
        let from = 52; // e7
        let to = 61; // f8
        board.white_pawns = 1u64 << from;
        board.black_rook = 1u64 << to; // Enemy piece to capture
        let promo = Some(Piece::Knight);

        let result = board.move_pawn(from, to, true, promo, None, &mut None);

        assert!(result, "Promotion by capture should be a valid move.");
        assert_eq!(board.white_pawns, 0, "Original pawn should be removed.");
        assert_eq!(board.black_rook, 0, "Captured piece should be removed.");
        assert_eq!(
            board.white_knight,
            1u64 << to,
            "A new white knight should be on the target square."
        );
    }

    // --- Black Pawn Promotion Tests ---

    #[test]
    fn test_black_pawn_promo_by_push_to_rook() {
        let mut board = setup_board();
        let from = 12; // e2
        let to = 4; // e1
        board.black_pawns = 1u64 << from;
        let promo = Some(Piece::Rook);

        let result = board.move_pawn(from, to, false, promo, None, &mut None);

        assert!(result, "Black promotion by push should be valid.");
        assert_eq!(board.black_pawns, 0, "Original black pawn should be gone.");
        assert_eq!(
            board.black_rook,
            1u64 << to,
            "A new black rook should be on the target square."
        );
    }

    #[test]
    fn test_black_pawn_promo_by_capture_to_bishop() {
        let mut board = setup_board();
        let from = 12; // e2
        let to = 5; // f1
        board.black_pawns = 1u64 << from;
        board.white_bishop = 1u64 << to; // Enemy piece to capture
        let promo = Some(Piece::Bishop);

        let result = board.move_pawn(from, to, false, promo, None, &mut None);

        assert!(result, "Black promotion by capture should be valid.");
        assert_eq!(board.black_pawns, 0, "Original black pawn should be gone.");
        assert_eq!(
            board.white_bishop, 0,
            "Captured white piece should be gone."
        );
        assert_eq!(
            board.black_bishop,
            1u64 << to,
            "A new black bishop should be on the target square."
        );
    }

    // --- Invalid Promotion Tests ---

    #[test]
    fn test_invalid_promo_when_no_piece_is_specified() {
        let mut board = setup_board();
        let from = 52; // e7
        let to = 60; // e8
        board.white_pawns = 1u64 << from;
        let promo = None; // User did not specify a promotion piece

        let result = board.move_pawn(from, to, true, promo, None, &mut None);

        assert!(
            !result,
            "Promotion should be invalid if no piece is specified."
        );
        assert_eq!(
            board.white_pawns,
            1u64 << from,
            "Board state should not change."
        );
    }

    #[test]
    fn test_invalid_promo_on_non_promotion_rank_push() {
        let mut board = setup_board();
        let from = 44; // e6
        let to = 52; // e7
        board.white_pawns = 1u64 << from;
        let promo = Some(Piece::Queen); // Attempting to promote early

        let result = board.move_pawn(from, to, true, promo, None, &mut None);

        // Your new code correctly checks `if promo.is_some() { return false; }`
        // for non-promotion pushes. This test verifies that behavior.
        assert!(
            !result,
            "Should not be able to specify a promotion piece on a non-promotion rank push."
        );
    }

    #[test]
    fn test_invalid_promo_on_non_promotion_rank_capture() {
        let mut board = setup_board();
        let from = 44; // e6
        let to = 53; // f7
        board.white_pawns = 1u64 << from;
        board.black_pawns = 1u64 << to;
        let promo = Some(Piece::Queen); // Attempting to promote early

        let result = board.move_pawn(from, to, true, promo, None, &mut None);

        // Your new code correctly returns false if a promotion piece is specified
        // for a capture that does not land on the final rank.
        assert!(
            !result,
            "Should not be able to specify a promotion piece on a non-promotion rank capture."
        );
    }

    #[test]
    fn test_valid_non_promo_capture() {
        let mut board = setup_board();
        let from = 35; // d5
        let to = 44; // e6
        board.white_pawns = 1u64 << from;
        board.black_pawns = 1u64 << to;
        let promo = None; // No promotion

        let result = board.move_pawn(from, to, true, promo, None, &mut None);

        assert!(result, "A regular pawn capture should be a valid move.");
        assert_eq!(
            board.white_pawns,
            1u64 << to,
            "White pawn should be on the new square."
        );
        assert_eq!(
            board.black_pawns, 0,
            "Black pawn should have been captured."
        );
    }
}
