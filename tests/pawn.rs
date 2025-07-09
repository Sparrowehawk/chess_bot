use chess_bot::bitboard::{Bitboard, Piece};

fn setup_board(
    white_pawn_pos: Option<usize>,
    black_pawn_pos: Option<usize>,
    white_piece_pos: Option<usize>,
    black_piece_pos: Option<usize>,
) -> Bitboard {
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

    if let Some(pos) = white_pawn_pos {
        board.white_pawns |= 1u64 << pos;
    }
    if let Some(pos) = black_pawn_pos {
        board.black_pawns |= 1u64 << pos;
    }
    if let Some(pos) = white_piece_pos {
        board.white_knight |= 1u64 << pos;
    }
    if let Some(pos) = black_piece_pos {
        board.black_rook |= 1u64 << pos; // Using rook for variety
    }
    board
}

// --- VALID PUSHES ---

#[test]
fn test_white_pawn_single_push() {
    let mut board = setup_board(Some(12), None, None, None); // e2
    let mut new_ep_target = None;
    assert!(board.move_pawn(12, 20, true, None, None, &mut new_ep_target));
    assert_eq!(board.white_pawns, 1u64 << 20);
    assert_eq!(new_ep_target, None); // Single push should not set ep target
}

#[test]
fn test_white_pawn_double_push() {
    let mut board = setup_board(Some(12), None, None, None); // e2
    let mut new_ep_target = None;
    assert!(board.move_pawn(12, 28, true, None, None, &mut new_ep_target));
    assert_eq!(board.white_pawns, 1u64 << 28);
}

// --- EN PASSANT TESTS ---

#[test]
fn test_white_pawn_double_push_sets_en_passant_target() {
    let mut board = setup_board(Some(12), None, None, None); // e2
    let mut new_ep_target = None;
    board.move_pawn(12, 28, true, None, None, &mut new_ep_target);
    assert_eq!(new_ep_target, Some(20)); // e3 is the new target
}

#[test]
fn test_black_pawn_double_push_sets_en_passant_target() {
    let mut board = setup_board(None, Some(51), None, None); // d7
    let mut new_ep_target = None;
    board.move_pawn(51, 35, false, None, None, &mut new_ep_target);
    assert_eq!(new_ep_target, Some(43)); // d6 is the new target
}

#[test]
fn test_white_pawn_en_passant_capture() {
    // White pawn on e5, black pawn on d5. EP target is d6 (43).
    let mut board = setup_board(Some(36), Some(35), None, None);
    let en_passant_for_this_turn = Some(43);
    let mut new_ep_target = None;

    assert!(board.move_pawn(36, 43, true, None, en_passant_for_this_turn, &mut new_ep_target));
    assert_eq!(board.white_pawns, 1u64 << 43); // White pawn moved to d6
    assert_eq!(board.black_pawns, 0); // Black pawn at d5 was captured
}

#[test]
fn test_black_pawn_en_passant_capture() {
    let mut board = setup_board(Some(26), Some(27), None, None);
    let en_passant_for_this_turn = Some(18);
    let mut new_ep_target = None;

    let move_was_legal =
        board.move_pawn(27, 18, false, None, en_passant_for_this_turn, &mut new_ep_target);

    assert!(move_was_legal);
    assert_eq!(board.black_pawns, 1u64 << 18); // The black pawn should now be at c3 (18).
    assert_eq!(board.white_pawns, 0); // The captured white pawn at c4 (26) should be gone.
}

// --- INVALID PUSHES & CAPTURES ---

#[test]
fn test_white_pawn_double_push_from_wrong_rank() {
    let mut board = setup_board(Some(20), None, None, None); // e3
    assert!(!board.move_pawn(20, 36, true, None, None, &mut None));
}

#[test]
fn test_white_pawn_push_blocked_by_piece() {
    let mut board = setup_board(Some(12), Some(20), None, None); // White e2, Black e3
    assert!(!board.move_pawn(12, 20, true, None, None, &mut None));
}

#[test]
fn test_pawn_cannot_capture_empty_square_diagonally() {
    let mut board = setup_board(Some(12), None, None, None); // White e2
    assert!(!board.move_pawn(12, 19, true, None, None, &mut None));
}

// --- PROMOTION TESTS ---

#[test]
fn test_white_pawn_promo_by_push_to_queen() {
    let mut board = setup_board(Some(52), None, None, None); // e7
    let promo = Some(Piece::Queen);
    let result = board.move_pawn(52, 60, true, promo, None, &mut None);

    assert!(result, "Promotion by push should be a valid move.");
    assert_eq!(board.white_pawns, 0, "Original pawn should be removed.");
    assert_eq!(board.white_queen, 1u64 << 60, "A new white queen should be on the target square.");
}

#[test]
fn test_white_pawn_promo_by_capture_to_knight() {
    let mut board = setup_board(Some(52), None, None, Some(61)); // White pawn e7, black piece f8
    let promo = Some(Piece::Knight);
    let result = board.move_pawn(52, 61, true, promo, None, &mut None);

    assert!(result, "Promotion by capture should be a valid move.");
    assert_eq!(board.white_pawns, 0, "Original pawn should be removed.");
    assert_eq!(board.black_rook, 0, "Captured piece should be removed.");
    assert_eq!(board.white_knight, 1u64 << 61, "A new white knight should be on the target square.");
}

#[test]
fn test_black_pawn_promo_by_push_to_rook() {
    let mut board = setup_board(None, Some(12), None, None); // Black pawn e2
    let promo = Some(Piece::Rook);
    let result = board.move_pawn(12, 4, false, promo, None, &mut None);

    assert!(result, "Black promotion by push should be valid.");
    assert_eq!(board.black_pawns, 0, "Original black pawn should be gone.");
    assert_eq!(board.black_rook, 1u64 << 4, "A new black rook should be on the target square.");
}

#[test]
fn test_black_pawn_promo_by_capture_to_bishop() {
    let mut board = setup_board(None, Some(12), Some(5), None); // Black pawn e2, white piece f1
    let promo = Some(Piece::Bishop);
    let result = board.move_pawn(12, 5, false, promo, None, &mut None);

    assert!(result, "Black promotion by capture should be valid.");
    assert_eq!(board.black_pawns, 0, "Original black pawn should be gone.");
    assert_eq!(board.white_knight, 0, "Captured white piece should be gone.");
    assert_eq!(board.black_bishop, 1u64 << 5, "A new black bishop should be on the target square.");
}

#[test]
fn test_invalid_promo_when_no_piece_is_specified() {
    let mut board = setup_board(Some(52), None, None, None); // e7
    let result = board.move_pawn(52, 60, true, None, None, &mut None);

    assert!(!result, "Promotion should be invalid if no piece is specified.");
    assert_eq!(board.white_pawns, 1u64 << 52, "Board state should not change.");
}

#[test]
fn test_invalid_promo_on_non_promotion_rank_push() {
    let mut board = setup_board(Some(44), None, None, None); // e6
    let promo = Some(Piece::Queen); // Attempting to promote early
    let result = board.move_pawn(44, 52, true, promo, None, &mut None);
    
    assert!(!result, "Should not be able to specify a promotion piece on a non-promotion rank push.");
}

#[test]
fn test_invalid_promo_on_non_promotion_rank_capture() {
    let mut board = setup_board(Some(44), Some(53), None, None); // White e6, Black f7
    let promo = Some(Piece::Queen); // Attempting to promote early
    let result = board.move_pawn(44, 53, true, promo, None, &mut None);
    
    assert!(!result, "Should not be able to specify a promotion piece on a non-promotion rank capture.");
}
