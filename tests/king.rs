use chess_bot::bitboard::Bitboard;


fn setup_board(king_pos: usize, opponent_pos: Option<usize>, friendly_pos: Option<usize>) -> Bitboard {
    let mut board = Bitboard {
        white_king: 0, white_queen: 0, white_rook: 0, white_bishop: 0,
        white_knight: 0, white_pawns: 0, black_king: 0, black_queen: 0,
        black_rook: 0, black_bishop: 0, black_knight: 0, black_pawns: 0,
    };
    
    // We'll test with the white king by default
    board.white_king |= 1u64 << king_pos;

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
fn test_king_valid_moves_all_directions() {
    let center_square = 27; // d4
    let expected_moves = [
        center_square - 9, center_square - 8, center_square - 7,
        center_square - 1,                      center_square + 1,
        center_square + 7, center_square + 8, center_square + 9,
    ];

    for &target_square in &expected_moves {
        let mut board = setup_board(center_square, None, None);
        assert!(
            board.move_king(center_square, target_square, true),
            "King should move from {center_square} to {target_square}"
        );
        assert_eq!(board.white_king, 1u64 << target_square, "King should be at the new position");
    }
}

#[test]
fn test_king_valid_captures_all_directions() {
    let center_square = 27; // d4
    let opponent_squares = [
        center_square - 9, center_square - 8, center_square - 7,
        center_square - 1,                      center_square + 1,
        center_square + 7, center_square + 8, center_square + 9,
    ];

    for &target_square in &opponent_squares {
        let mut board = setup_board(center_square, Some(target_square), None);
        assert!(
            board.move_king(center_square, target_square, true),
            "King should capture from {center_square} to {target_square}"
        );
        assert_eq!(board.white_king, 1u64 << target_square, "King should be at the new position");
        assert_eq!(board.black_pawns, 0, "Opponent piece should be captured");
    }
}

#[test]
fn test_black_king_moves() {
    let mut board = Bitboard::new(); // Standard setup
    // Clear path for black king e8 -> f7
    board.black_pawns &= !(1u64 << 53); // remove f7 pawn
    board.black_knight &= !(1u64 << 57); // remove g8 knight
    
    assert!(board.move_king(60, 53, false));
    assert_eq!(board.black_king, 1u64 << 53);
}


// --- INVALID MOVES ---

#[test]
fn test_king_cannot_move_two_squares() {
    let mut board = setup_board(27, None, None); // King at d4
    assert!(!board.move_king(27, 43, true)); // d4 to d6
    assert!(!board.move_king(27, 29, true)); // d4 to f4
}

#[test]
fn test_king_cannot_move_to_friendly_piece() {
    let mut board = setup_board(27, None, Some(28)); // King d4, friendly pawn e4
    assert!(!board.move_king(27, 28, true));
    assert_eq!(board.white_king, 1u64 << 27); // King should not have moved
    assert_eq!(board.white_pawns, 1u64 << 28); // Friendly piece should still be there
}

#[test]
fn test_king_wraparound_bug_h_to_a() {
    // King on h4 (31) cannot move to a4 (24), even though 31-7 = 24
    let mut board = setup_board(31, None, None); 
    assert!(!board.move_king(31, 24, true));
}

#[test]
fn test_king_wraparound_bug_a_to_h() {
    // King on a4 (24) cannot move to h5 (39), even though 24+15 is not a valid move, let's test 24+9 to b5
    // A better test: a4(24) to h3(23) -> diff is -1, which is a valid offset, but invalid move
    let mut board = setup_board(24, None, None);
    assert!(!board.move_king(24, 23, true)); // This should be a5->h4, let's re-verify indices
    // a4 is 24. h3 is 23. from 24 to 23 is a valid move (a4->h3 is not, it should be a4->b3)
    // The indices are: a4=24, h3=23. This is not a wraparound.
    // Let's test a1(0) to h8(63). The difference is large.
    // A correct wraparound test: a1(0) to h2(15).
    let mut board_a1 = setup_board(0, None, None);
    assert!(!board_a1.move_king(0, 15, true)); // a1 to h2
}


#[test]
fn test_move_non_existent_king() {
    let mut board = setup_board(27, None, None); // King at d4
    assert!(!board.move_king(0, 1, true)); // Try to move from a1 where there is no king
}