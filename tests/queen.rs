use chess_bot::bitboard::Bitboard;


fn setup_board(queen_pos: usize, opponent_pos: Option<usize>, friendly_pos: Option<usize>) -> Bitboard {
    let mut board = Bitboard {
        white_king: 0, white_queen: 0, white_rook: 0, white_bishop: 0,
        white_knight: 0, white_pawns: 0, black_king: 0, black_queen: 0,
        black_rook: 0, black_bishop: 0, black_knight: 0, black_pawns: 0,
    };
    
    // We'll test with a white queen by default
    board.white_queen |= 1u64 << queen_pos;

    if let Some(pos) = opponent_pos {
        board.black_pawns |= 1u64 << pos;
    }
    if let Some(pos) = friendly_pos {
        board.white_pawns |= 1u64 << pos;
    }
    board
}

// --- VALID MOVES & CAPTURES ---

#[test]
fn test_queen_valid_straight_moves() {
    let center_square = 27; // d4
    let valid_destinations = [
        3, 24, 31, 59 // d1, a4, h4, d8
    ];
    for &target in &valid_destinations {
        let mut board = setup_board(center_square, None, None);
        assert!(board.move_queen(center_square, target, true));
        assert_eq!(board.white_queen, 1u64 << target);
    }
}

#[test]
fn test_queen_valid_diagonal_moves() {
    let center_square = 27; // d4
    let valid_destinations = [
        0, 9, 45, 63, 6, 20, 34, 48
    ];
    for &target in &valid_destinations {
        let mut board = setup_board(center_square, None, None);
        assert!(board.move_queen(center_square, target, true));
        assert_eq!(board.white_queen, 1u64 << target);
    }
}

#[test]
fn test_queen_valid_captures() {
    let center_square = 27; // d4
    // Test one straight and one diagonal capture
    let opponent_squares = [29, 45]; // f4 and f6

    for &target in &opponent_squares {
        let mut board = setup_board(center_square, Some(target), None);
        assert!(board.move_queen(center_square, target, true));
        assert_eq!(board.white_queen, 1u64 << target);
        assert_eq!(board.black_pawns, 0);
    }
}

// --- INVALID MOVES ---

#[test]
fn test_queen_invalid_knight_move() {
    let mut board = setup_board(27, None, None); // Queen at d4
    assert!(!board.move_queen(27, 42, true)); // d4 to f5 (knight L-shape)
    assert_eq!(board.white_queen, 1u64 << 27); // Queen has not moved
}

#[test]
fn test_queen_cannot_move_to_friendly_piece() {
    let mut board = setup_board(27, None, Some(45)); // Queen d4, friendly pawn f6
    assert!(!board.move_queen(27, 45, true));
    assert_eq!(board.white_queen, 1u64 << 27);
    assert_eq!(board.white_pawns, 1u64 << 45);
}

// --- BLOCKED PATHS ---

#[test]
fn test_queen_path_blocked_straight() {
    // Vertical block
    let mut board_v = setup_board(27, None, Some(35)); // Queen d4, friendly d5
    assert!(!board_v.move_queen(27, 51, true)); // Cannot move to d7

    // Horizontal block
    let mut board_h = setup_board(27, Some(29), None); // Queen d4, opponent f4
    assert!(!board_h.move_queen(27, 31, true)); // Cannot move to h4
}

#[test]
fn test_queen_path_blocked_diagonal() {
    // Up-right block
    let mut board_ur = setup_board(27, None, Some(36)); // Queen d4, friendly e5
    assert!(!board_ur.move_queen(27, 45, true)); // Cannot move to f6

    // Up-left block
    let mut board_ul = setup_board(27, Some(18), None); // Queen d4, opponent c5
    assert!(!board_ul.move_queen(27, 9, true)); // Cannot move to b6
}

#[test]
fn test_black_queen_moves_from_start() {
    let mut board = Bitboard::new(); // Standard setup
    // Clear path for black queen d8 -> a5
    board.black_pawns &= !(1u64 << 51); // remove d7 pawn
    
    // d8(59) -> a5(32) is a diagonal move. Path is d8,c7,b6,a5.
    // Blocked by c7 pawn (50).
    board.black_pawns &= !(1u64 << 50); // remove c7 pawn
    
    assert!(board.move_queen(59, 32, false)); // d8 to c5
    assert_eq!(board.black_queen, 1u64 << 32);
}