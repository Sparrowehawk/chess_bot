use chess_bot::bitboard::Bitboard;

// Helper to create a board with specific pieces for bishop testing.
fn setup_board(bishop_pos: usize, opponent_pos: Option<usize>, friendly_pos: Option<usize>) -> Bitboard {
    let mut board = Bitboard {
        white_king: 0, white_queen: 0, white_rook: 0, white_bishop: 0,
        white_knight: 0, white_pawns: 0, black_king: 0, black_queen: 0,
        black_rook: 0, black_bishop: 0, black_knight: 0, black_pawns: 0,
    };
    
    // We'll test with a white bishop by default
    board.white_bishop |= 1u64 << bishop_pos;

    if let Some(pos) = opponent_pos {
        // Use a black pawn as the opponent piece
        board.black_pawns |= 1u64 << pos;
    }
    if let Some(pos) = friendly_pos {
        // Use a white pawn as the friendly piece
        board.white_pawns |= 1u64 << pos;
    }
    board
}

// --- VALID MOVES & CAPTURES ---

#[test]
fn test_bishop_valid_moves() {
    let center_square = 27; // d4
    let valid_destinations = [
        0, 9, 45, 63, // Up-right and down-left diagonal (A1-H8)
        6, 20, 34, 48 // Up-left and down-right diagonal (G1-A7)
    ];

    for &target in &valid_destinations {
        let mut board = setup_board(center_square, None, None);
        assert!(
            board.move_bishop(center_square, target, true),
            "Bishop should move from {center_square} to {target}"
        );
        assert_eq!(board.white_bishop, 1u64 << target);
    }
}

#[test]
fn test_bishop_valid_captures() {
    let center_square = 27; // d4
    let opponent_squares = [9, 45, 20, 48]; // Some diagonal squares

    for &target in &opponent_squares {
        let mut board = setup_board(center_square, Some(target), None);
        assert!(
            board.move_bishop(center_square, target, true),
            "Bishop should capture from {center_square} to {target}"
        );
        assert_eq!(board.white_bishop, 1u64 << target);
        assert_eq!(board.black_pawns, 0);
    }
}

#[test]
fn test_black_bishop_moves() {
    let mut board = Bitboard::new(); // Standard setup
    // The move is from c8(58) to a6(40). The path is blocked by the b7 pawn (49).
    board.black_pawns &= !(1u64 << 49); // Correctly remove the b7 pawn
    
    assert!(board.move_bishop(58, 40, false)); // Move c8 -> a6
    
    // Assert that the bishop is now at a6 (40) and the other black bishop at f8 (61) is untouched.
    assert_eq!(board.black_bishop, (1u64 << 40) | (1u64 << 61)); 
}
// --- INVALID MOVES ---

#[test]
fn test_bishop_invalid_straight_move() {
    let mut board = setup_board(27, None, None); // Bishop at d4
    assert!(!board.move_bishop(27, 28, true)); // d4 to e4 (horizontal)
    assert!(!board.move_bishop(27, 35, true)); // d4 to d5 (vertical)
    assert_eq!(board.white_bishop, 1u64 << 27); // Bishop has not moved
}

#[test]
fn test_bishop_invalid_knight_move() {
    let mut board = setup_board(27, None, None); // Bishop at d4
    assert!(!board.move_bishop(27, 42, true)); // d4 to f5 (knight L-shape)
}

#[test]
fn test_bishop_cannot_move_to_friendly_piece() {
    let mut board = setup_board(27, None, Some(45)); // Bishop d4, friendly pawn f6
    assert!(!board.move_bishop(27, 45, true));
    assert_eq!(board.white_bishop, 1u64 << 27);
    assert_eq!(board.white_pawns, 1u64 << 45);
}

// --- BLOCKED PATHS ---

#[test]
fn test_bishop_path_blocked_up_right() {
    let mut board = setup_board(0, None, Some(9)); // Bishop a1, friendly e5
    assert!(!board.move_bishop(0, 18, true)); // Cannot move to c3
}

#[test]
fn test_bishop_path_blocked_down_left() {
    let mut board = setup_board(63, Some(54), None); // Bishop h8, opponent g7
    assert!(!board.move_bishop(63, 45, true)); // Cannot move to f6
}

#[test]
fn test_bishop_path_blocked_up_left() {
    let mut board = setup_board(28, None, Some(21)); // Bishop e4, friendly d5
    assert!(!board.move_bishop(28, 14, true)); // Cannot move to c6
}

#[test]
fn test_bishop_path_blocked_down_right() {
    let mut board = setup_board(20, Some(13), None); // Bishop e3, opponent f2
    assert!(!board.move_bishop(20, 6, true)); // Cannot move to g1
}