use chess_bot::bitboard::Bitboard;

fn setup_board(white_pawn_pos: Option<usize>, black_pawn_pos: Option<usize>, friendly_piece_pos: Option<usize>) -> Bitboard {
    let mut board = Bitboard {
        white_king: 0, white_queen: 0, white_rook: 0, white_bishop: 0,
        white_knight: 0, white_pawns: 0, black_king: 0, black_queen: 0,
        black_rook: 0, black_bishop: 0, black_knight: 0, black_pawns: 0,
    };

    if let Some(pos) = white_pawn_pos {
        board.white_pawns |= 1u64 << pos;
    }
    if let Some(pos) = black_pawn_pos {
        board.black_pawns |= 1u64 << pos;
    }
    if let Some(pos) = friendly_piece_pos {
        // Use a knight as the friendly piece for simplicity
        board.white_knight |= 1u64 << pos;
    }
    board
}


// --- VALID PUSHES ---

#[test]
fn test_white_pawn_single_push() {
    let mut board = setup_board(Some(12), None, None); // e2
    assert!(board.move_pawn(12, 20, true)); // e3
    assert_eq!(board.white_pawns, 1u64 << 20);
}

#[test]
fn test_white_pawn_double_push() {
    let mut board = setup_board(Some(12), None, None); // e2
    assert!(board.move_pawn(12, 28, true)); // e4
    assert_eq!(board.white_pawns, 1u64 << 28);
}

#[test]
fn test_black_pawn_single_push() {
    let mut board = setup_board(None, Some(51), None); // d7
    assert!(board.move_pawn(51, 43, false)); // d6
    assert_eq!(board.black_pawns, 1u64 << 43);
}

#[test]
fn test_black_pawn_double_push() {
    let mut board = setup_board(None, Some(51), None); // d7
    assert!(board.move_pawn(51, 35, false)); // d5
    assert_eq!(board.black_pawns, 1u64 << 35);
}


// --- INVALID PUSHES ---

#[test]
fn test_white_pawn_double_push_from_wrong_rank() {
    let mut board = setup_board(Some(20), None, None); // e3
    assert!(!board.move_pawn(20, 36, true)); // e5 is illegal from e3
}

#[test]
fn test_black_pawn_double_push_from_wrong_rank() {
    let mut board = setup_board(None, Some(43), None); // d6
    assert!(!board.move_pawn(43, 27, false)); // d4 is illegal from d6
}

#[test]
fn test_white_pawn_push_blocked_by_piece() {
    let mut board = setup_board(Some(12), Some(20), None); // White e2, Black e3
    assert!(!board.move_pawn(12, 20, true)); // Cannot push to e3
}

#[test]
fn test_white_pawn_double_push_blocked_by_piece() {
    let mut board = setup_board(Some(12), Some(20), None); // White e2, Black e3
    assert!(!board.move_pawn(12, 28, true)); // Cannot double push over e3
}


// --- VALID CAPTURES ---

#[test]
fn test_white_pawn_captures() {
    // Pawn at e4, opponents at d5 and f5
    let mut board = setup_board(Some(28), Some(35), None);
    board.black_pawns |= 1u64 << 37;

    // Capture d5
    let mut board_for_d5_capture = board.clone();
    assert!(board_for_d5_capture.move_pawn(28, 35, true));
    assert_eq!(board_for_d5_capture.white_pawns, 1u64 << 35);
    assert_eq!(board_for_d5_capture.black_pawns, 1u64 << 37); // f5 pawn remains

    // Capture f5
    let mut board_for_f5_capture = board.clone();
    assert!(board_for_f5_capture.move_pawn(28, 37, true));
    assert_eq!(board_for_f5_capture.white_pawns, 1u64 << 37);
    assert_eq!(board_for_f5_capture.black_pawns, 1u64 << 35); // d5 pawn remains
}

#[test]
fn test_black_pawn_captures() {
    // Pawn at d5, opponents at c4 and e4
    let mut board = setup_board(Some(26), Some(35), None);
    board.white_pawns |= 1u64 << 28;

    // Capture c4
    let mut board_for_c4_capture = board.clone();
    assert!(board_for_c4_capture.move_pawn(35, 26, false));
    assert_eq!(board_for_c4_capture.black_pawns, 1u64 << 26);
    assert_eq!(board_for_c4_capture.white_pawns, 1u64 << 28); // e4 pawn remains

    // Capture e4
    let mut board_for_e4_capture = board.clone();
    assert!(board_for_e4_capture.move_pawn(35, 28, false));
    assert_eq!(board_for_e4_capture.black_pawns, 1u64 << 28);
    assert_eq!(board_for_e4_capture.white_pawns, 1u64 << 26); // c4 pawn remains
}


// --- INVALID CAPTURES & OTHER MOVES ---

#[test]
fn test_pawn_cannot_capture_forward() {
    let mut board = setup_board(Some(12), Some(20), None); // White e2, Black e3
    assert!(!board.move_pawn(12, 20, true));
}

#[test]
fn test_pawn_cannot_capture_empty_square_diagonally() {
    let mut board = setup_board(Some(12), None, None); // White e2
    assert!(!board.move_pawn(12, 19, true)); // Attempt to "capture" d3
}

#[test]
fn test_pawn_cannot_capture_friendly_piece() {
    let mut board = setup_board(Some(12), None, Some(19)); // White e2, friendly knight d3
    assert!(!board.move_pawn(12, 19, true));
}

#[test]
fn test_pawn_cannot_move_backwards() {
    let mut board = setup_board(Some(28), None, None); // White e4
    assert!(!board.move_pawn(28, 20, true)); // Attempt to move to e3
}

#[test]
fn test_pawn_cannot_move_sideways() {
    let mut board = setup_board(Some(28), None, None); // White e4
    assert!(!board.move_pawn(28, 29, true)); // Attempt to move to f4
}