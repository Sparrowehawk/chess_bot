use chess_bot::{bitboard::Bitboard, Game};

fn setup_board(knight_pos: usize, opponent_pos: Option<usize>, friendly_pos: Option<usize>) -> Bitboard {
    let mut board = Bitboard {
        white_king: 0, white_queen: 0, white_rook: 0, white_bishop: 0,
        white_knight: 0, white_pawns: 0, black_king: 0, black_queen: 0,
        black_rook: 0, black_bishop: 0, black_knight: 0, black_pawns: 0,
    };
    
    board.white_knight |= 1u64 << knight_pos;

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
fn test_knight_moves_from_start() {
    let game = Game::new();
    let moves = game.generate_legal_moves();
    let knight_moves: Vec<_> = moves
        .iter()
        .filter(|(from, _, _)| *from == 1 || *from == 6) // b1 or g1
        .collect();

    for (from, to, _) in &knight_moves {
        println!("Knight move: {from} -> {to}");
    }

    assert_eq!(knight_moves.len(), 4); // b1→a3/c3, g1→f3/h3
}

#[test]
fn test_knight_valid_moves_all_directions() {
    let center_square = 27; // d4
    let valid_destinations = [
        10, 12, 17, 21, 33, 37, 42, 44 // All 8 L-moves from d4
    ];

    for &target in &valid_destinations {
        let mut board = setup_board(center_square, None, None);
        assert!(
            board.move_knight(center_square, target, true),
            "Knight should move from {center_square} to {target}"
        );
        assert_eq!(board.white_knight, 1u64 << target);
    }
}

#[test]
fn test_knight_valid_captures() {
    let center_square = 27; // d4
    let target_square = 42; // f5

    let mut board = setup_board(center_square, Some(target_square), None);
    assert!(board.move_knight(center_square, target_square, true));
    assert_eq!(board.white_knight, 1u64 << target_square);
    assert_eq!(board.black_pawns, 0, "Opponent piece should be captured");
}

#[test]
fn test_black_knight_moves_from_start() {
    let mut board = Bitboard::new(); // Standard setup
    // b8 (57) -> a6 (40)
    assert!(board.move_knight(57, 40, false));
    // The other black knight at g8 (62) should remain
    assert_eq!(board.black_knight, (1u64 << 40) | (1u64 << 62));
}

// --- JUMPING ABILITY ---

#[test]
fn test_knight_can_jump_over_pieces() {
    let mut board = Bitboard::new(); // Standard setup
    // White knight g1 (6) -> f3 (21)
    // This move jumps over the g2 and h2 pawns.
    assert!(board.move_knight(6, 21, true));
    assert_eq!(board.white_knight, (1u64 << 1) | (1u64 << 21)); // b1 knight remains
}

// --- INVALID MOVES ---

#[test]
fn test_knight_invalid_moves() {
    let mut board = setup_board(27, None, None); // Knight at d4
    // Straight move
    assert!(!board.move_knight(27, 28, true));
    // Diagonal move
    assert!(!board.move_knight(27, 36, true));
    // Other invalid move
    assert!(!board.move_knight(27, 26, true));
    
    assert_eq!(board.white_knight, 1u64 << 27, "Knight should not have moved");
}

#[test]
fn test_knight_cannot_move_to_friendly_piece() {
    let mut board = setup_board(27, None, Some(42)); // Knight d4, friendly pawn f5
    assert!(!board.move_knight(27, 42, true));
    assert_eq!(board.white_knight, 1u64 << 27);
    assert_eq!(board.white_pawns, 1u64 << 42);
}