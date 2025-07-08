use chess_bot::bitboard::Bitboard;

fn setup_board(
    white_pawn_pos: Option<usize>,
    black_pawn_pos: Option<usize>,
    friendly_piece_pos: Option<usize>,
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
    if let Some(pos) = friendly_piece_pos {
        board.white_knight |= 1u64 << pos;
    }
    board
}

// --- VALID PUSHES ---

#[test]
fn test_white_pawn_single_push() {
    let mut board = setup_board(Some(12), None, None); // e2
    let mut new_ep_target = None;
    assert!(board.move_pawn(12, 20, true, None, &mut new_ep_target));
    assert_eq!(board.white_pawns, 1u64 << 20);
    assert_eq!(new_ep_target, None); // Single push should not set ep target
}

#[test]
fn test_white_pawn_double_push() {
    let mut board = setup_board(Some(12), None, None); // e2
    let mut new_ep_target = None;
    assert!(board.move_pawn(12, 28, true, None, &mut new_ep_target));
    assert_eq!(board.white_pawns, 1u64 << 28);
}

// --- EN PASSANT TESTS ---

#[test]
fn test_white_pawn_double_push_sets_en_passant_target() {
    let mut board = setup_board(Some(12), None, None); // e2
    let mut new_ep_target = None;
    board.move_pawn(12, 28, true, None, &mut new_ep_target);
    assert_eq!(new_ep_target, Some(20)); // e3 is the new target
}

#[test]
fn test_black_pawn_double_push_sets_en_passant_target() {
    let mut board = setup_board(None, Some(51), None); // d7
    let mut new_ep_target = None;
    board.move_pawn(51, 35, false, None, &mut new_ep_target);
    assert_eq!(new_ep_target, Some(43)); // d6 is the new target
}

#[test]
fn test_white_pawn_en_passant_capture() {
    // White pawn on e5, black pawn on d5. EP target is d6 (43).
    let mut board = setup_board(Some(36), Some(35), None);
    let en_passant_for_this_turn = Some(43);
    let mut new_ep_target = None;

    assert!(board.move_pawn(36, 43, true, en_passant_for_this_turn, &mut new_ep_target));
    assert_eq!(board.white_pawns, 1u64 << 43); // White pawn moved to d6
    assert_eq!(board.black_pawns, 0); // Black pawn at d5 was captured
}

#[test]
fn test_black_pawn_en_passant_capture() {
    let mut board = setup_board(Some(26), Some(27), None);
    let en_passant_for_this_turn = Some(18);
    let mut new_ep_target = None;

    let move_was_legal =
        board.move_pawn(27, 18, false, en_passant_for_this_turn, &mut new_ep_target);

    assert!(move_was_legal);

    // The black pawn should now be at c3 (18).
    assert_eq!(board.black_pawns, 1u64 << 18);

    // The captured white pawn at c4 (26) should be gone.
    assert_eq!(board.white_pawns, 0);
}

// --- INVALID PUSHES & CAPTURES ---

#[test]
fn test_white_pawn_double_push_from_wrong_rank() {
    let mut board = setup_board(Some(20), None, None); // e3
    assert!(!board.move_pawn(20, 36, true, None, &mut None));
}

#[test]
fn test_white_pawn_push_blocked_by_piece() {
    let mut board = setup_board(Some(12), Some(20), None); // White e2, Black e3
    assert!(!board.move_pawn(12, 20, true, None, &mut None));
}

#[test]
fn test_pawn_cannot_capture_empty_square_diagonally() {
    let mut board = setup_board(Some(12), None, None); // White e2
    assert!(!board.move_pawn(12, 19, true, None, &mut None));
}
