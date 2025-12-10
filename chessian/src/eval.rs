use chess::*;

use crate::bbiter::BitBoardIter;

/// Value of a pawn in centipawns
pub const PAWN_VALUE: i32 = 100;
/// Value of a knight in centipawns
pub const KNIGHT_VALUE: i32 = 320;
/// Value of a bishop in centipawns
pub const BISHOP_VALUE: i32 = 333;
/// Value of a rook in centipawns
pub const ROOK_VALUE: i32 = 500;
/// Value of a queen in centipawns
pub const QUEEN_VALUE: i32 = 900;
/// Value of a king in centipawns
pub const KING_VALUE: i32 = 20000;

/// Array of the piece values in centipawns in the canonical order pawn, knight, bishop, rook,
/// queen, king.
pub const PIECE_VALUES: [i32; 6] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];

/// The sanction, in centipawns, of having a double pawn.
pub const DOUBLE_PAWN_SANCTION: i32 = 45;


pub fn eval(board: &Board) -> i32 {
    let mut result = 0;
    let is_endgame = board.combined().popcnt() < 20;

    let white_pieces = board.color_combined(Color::White);
    let black_pieces = board.color_combined(Color::Black);
    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let kings = board.pieces(Piece::King);

    /// Adds or subtracts the values for the given piece type from the tally.
    macro_rules! piece_values {
        ($op:tt, $bb_col:expr, $bb_pieces:expr, $color_index:literal, $piece_index:literal) => {
            for i in BitBoardIter::new($bb_col & $bb_pieces) {
                result $op SQUARE_SCORES[$color_index][$piece_index][i] + PIECE_VALUES[$piece_index];
            }
        };
        (pawns: $op:tt, $bb_col:expr, $bb_pieces:expr, $color_index:literal) => {
            if is_endgame {
                for i in BitBoardIter::new($bb_col & $bb_pieces) {
                    result $op SQUARE_SCORES[$color_index][0][i] + PIECE_VALUES[0];
                    result $op ENDGAME_PAWN_SCORES[$color_index][i];
                }
            } else {
                piece_values![$op, $bb_col, $bb_pieces, $color_index, 0]
            }
        };
        (kings: $op:tt, $bb_col:expr, $bb_pieces:expr, $color_index:literal) => {
            if is_endgame {
                for i in BitBoardIter::new($bb_col & $bb_pieces) {
                    result $op ENDGAME_KING_SCORES[$color_index][i];
                }
            } else {
                piece_values![$op, $bb_col, $bb_pieces, $color_index, 5]
            }
        }
    }

    piece_values![pawns: +=, white_pieces, pawns, 0];
    piece_values![+=, white_pieces, knights, 0, 1];
    piece_values![+=, white_pieces, bishops, 0, 2];
    piece_values![+=, white_pieces, rooks, 0, 3];
    piece_values![+=, white_pieces, queens, 0, 4];
    piece_values![kings: +=, white_pieces, kings, 0];

    piece_values![pawns: -=, black_pieces, pawns, 1];
    piece_values![-=, black_pieces, knights, 1, 1];
    piece_values![-=, black_pieces, bishops, 1, 2];
    piece_values![-=, black_pieces, rooks, 1, 3];
    piece_values![-=, black_pieces, queens, 1, 4];
    piece_values![kings: -=, black_pieces, kings, 1];

    // sanction double pawns
    let white_pawns = white_pieces & pawns;
    let black_pawns = black_pieces & pawns;

    for file in ALL_FILES {
        let file_bb = get_file(file);
        result -= ((white_pawns & file_bb).popcnt() as i32
            - (black_pawns & file_bb).popcnt() as i32)
            * DOUBLE_PAWN_SANCTION;
    }

    result
}

/// Piece-square-value table.
#[rustfmt::skip]
pub const SQUARE_SCORES: [[[i32; 64]; 6]; 2] = [
    [
        [
              0,   0,   0,   0,   0,   0,   0,   0,
              5,  10,  10, -20, -20,  10,  10,   5,
              5,  -5, -10,   0,   0, -10,  -5,   5,
              0,   0,   0,  20,  20,   0,   0,   0,
              5,   5,  10,  25,  25,  10,   5,   5,
             10,  10,  20,  30,  30,  20,  10,  10,
             50,  50,  50,  50,  50,  50,  50,  50,
              0,   0,   0,   0,   0,   0,   0,   0,
        ],
        [
            -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15,
            15, 10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0,
            10, 15, 15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30,
            -40, -50,
        ],
        [
            -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10, 10, 10,
            10, 10, 10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0,
            5, 10, 10, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -10, -10, -10, -10,
            -20,
        ],
        [
            0, 0, 0, 10, 10, 0, 0, 0, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 0,
            0, 0, 0, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, 10, 20, 20, 20, 20,
            20, 20, 10, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5,
            0, -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
        ],
        [
            10, 20, 10, 0, 0, 10, 20, 10, 10, 10, 0, 0, 0, 0, 10, 10, -10, -20, -20, -20, -20, -20,
            -20, -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40,
            -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
            -30, -40, -40, -50, -50, -40, -40, -30,
        ],
    ],
    [
        [
            0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10,
            5, 5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5,
            10, 10, -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15,
            15, 10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0,
            10, 15, 15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30,
            -40, -50,
        ],
        [
            -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10,
            5, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10,
            10, 10, 10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10,
            -20,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0,
            0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
            0, 0, -5, 0, 0, 0, 5, 5, 0, 0, 0,
        ],
        [
            -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5,
            0, -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10,
            -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
        ],
        [
            -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30,
            -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30,
            -30, -40, -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 10, 10, 0, 0, 0,
            0, 10, 10, 10, 20, 10, 0, 0, 10, 20, 10,
        ],
    ],
];

pub const ENDGAME_PAWN_SCORES: [[i32; 64]; 2] = [
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 15, 15, 15, 15, 15, 15, 15, 20, 20, 20,
        20, 20, 20, 20, 20, 25, 25, 25, 25, 25, 25, 25, 25, 30, 30, 30, 30, 30, 30, 30, 30, 35, 35,
        35, 35, 35, 35, 35, 35, 40, 40, 40, 40, 40, 40, 40, 40,
    ],
    [
        40, 40, 40, 40, 40, 40, 40, 40, 35, 35, 35, 35, 35, 35, 35, 35, 30, 30, 30, 30, 30, 30, 30,
        30, 25, 25, 25, 25, 25, 25, 25, 25, 20, 20, 20, 20, 20, 20, 20, 20, 15, 15, 15, 15, 15, 15,
        15, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

pub const ENDGAME_KING_SCORES: [[i32; 64]; 2] = [
    [
        -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20,
        30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10,
        -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30,
        -30, -30, -30, -30, -50,
    ],
    [
        -50, -30, -30, -30, -30, -30, -30, -50, -30, -30, 0, 0, 0, 0, -30, -30, -30, -10, 20, 30,
        30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30,
        -30, -10, 20, 30, 30, 20, -10, -30, -30, -20, -10, 0, 0, -10, -20, -30, -50, -40, -30, -20,
        -20, -30, -40, -50,
    ],
];
