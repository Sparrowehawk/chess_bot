use chess_bot::bitboard::Bitboard;

#[cfg(test)]
mod castling_tests {
    // Assuming the king.rs code is in the parent module
    use super::*; 

    // A helper function to create a new, empty board for testing purposes.
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
    fn test_white_kingside_castle_invalid_piece_in_way() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 6;   // g1
        board.white_king = 1u64 << from;
        board.white_rook = 1u64 << 7; // Rook on h1
        board.white_bishop = 1u64 << 5; // Bishop on f1
        let mut castling = 1 << 3;

        let result = board.move_king(from, to, true, &mut castling);

        assert!(!result, "Castling should be invalid with a piece in the way");
        assert_eq!(board.white_king, 1u64 << from, "King should not have moved");
        assert_eq!(board.white_rook, 1u64 << 7, "Rook should not have moved");
    }
    
    #[test]
    fn test_white_queenside_castle_invalid_no_rook() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 2;   // c1
        board.white_king = 1u64 << from;
        // No rook on a1
        let mut castling = 1 << 2;

        let result = board.move_king(from, to, true, &mut castling);

        assert!(!result, "Castling should be invalid without a rook");
        assert_eq!(board.white_king, 1u64 << from, "King should not have moved");
    }
    
    #[test]
    fn test_white_kingside_castle_invalid_no_rights() {
        let mut board = setup_board();
        let from = 4; // e1
        let to = 6;   // g1
        board.white_king = 1u64 << from;
        board.white_rook = 1u64 << 7;
        let mut castling = 0; // No castling rights

        let result = board.move_king(from, to, true, &mut castling);

        assert!(!result, "Castling should be invalid without rights");
    }

    // --- Black Castling Tests ---


    #[test]
    fn test_black_kingside_castle_invalid_piece_in_way() {
        let mut board = setup_board();
        let from = 60; // e8
        let to = 62;   // g8
        board.black_king = 1u64 << from;
        board.black_rook = 1u64 << 63; // Rook on h8
        board.black_knight = 1u64 << 61; // Knight on f8
        let mut castling = 1 << 1;

        let result = board.move_king(from, to, false, &mut castling);

        assert!(!result, "Castling should be invalid with a piece in the way");
        assert_eq!(board.black_king, 1u64 << from, "King should not have moved");
        assert_eq!(board.black_rook, 1u64 << 63, "Rook should not have moved");
    }
}

#[test]
fn test_white_king_side_castle() {
    let mut board = Bitboard {
        white_king: 1 << 4, // e1
        white_rook: 1 << 7, // h1
        white_queen: 0,
        white_bishop: 0,
        white_knight: 0,
        white_pawns: 0,
        black_king: 0,
        black_rook: 0,
        black_bishop: 0,
        black_knight: 0,
        black_queen: 0,
        black_pawns: 0,
    };

    let mut castling = 0b111111;

    let result = board.move_king(4, 6, true, &mut castling);
    assert!(result, "White king-side castling should succeed");
    assert_eq!(board.white_king, 1 << 6, "King should be on g1");
    assert_eq!(board.white_rook, 1 << 5, "Rook should be on f1");
}

#[test]
fn test_white_queen_side_castle() {
    let mut board = Bitboard::default();
    board.white_king = 1 << 4; // e1
    board.white_rook = 1 << 0; // a1
    board.white_queen = 0;
    board.white_bishop = 0;
    board.white_knight = 0;
    board.white_pawns = 0;
    board.black_king = 0;
    board.black_rook = 0;
    board.black_bishop = 0;
    board.black_knight = 0;
    board.black_queen = 0;
    board.black_pawns = 0;

    let mut castling = 0b111111;

    let result = board.move_king(4, 2, true, &mut castling);
    assert!(result, "White queen-side castling should succeed");
    assert_eq!(board.white_king, 1 << 2, "King should be on c1");
    assert_eq!(board.white_rook, 1 << 3, "Rook should be on d1");
}

#[test]
fn test_black_king_side_castle() {
    let mut board = Bitboard {
        black_king: 1 << 60, // e8
        black_rook: 1 << 63, // h8
        black_queen: 0,
        black_bishop: 0,
        black_knight: 0,
        black_pawns: 0,
        white_king: 0,
        white_rook: 0,
        white_bishop: 0,
        white_knight: 0,
        white_queen: 0,
        white_pawns: 0,
    };

    let mut castling = 0b111111;

    let result = board.move_king(60, 62, false, &mut castling);
    assert!(result, "Black king-side castling should succeed");
    assert_eq!(board.black_king, 1 << 62, "King should be on g8");
    assert_eq!(board.black_rook, 1 << 61, "Rook should be on f8");
}

#[test]
fn test_black_queen_side_castle() {
    let mut board = Bitboard {
        black_king: 1 << 60, // e8
        black_rook: 1 << 56, // a8
        black_queen: 0,
        black_bishop: 0,
        black_knight: 0,
        black_pawns: 0,
        white_king: 0,
        white_rook: 0,
        white_bishop: 0,
        white_knight: 0,
        white_queen: 0,
        white_pawns: 0,
    };

    let mut castling = 0b111111;

    let result = board.move_king(60, 58, false, &mut castling);
    assert!(result, "Black queen-side castling should succeed");
    assert_eq!(board.black_king, 1 << 58, "King should be on c8");
    assert_eq!(board.black_rook, 1 << 59, "Rook should be on d8");
}
