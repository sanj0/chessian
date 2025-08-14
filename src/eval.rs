use chess::*;

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 333;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20000;

pub const PIECE_VALUES: [i32; 6] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];

pub const DOUBLE_PAWN_SANCTION: i32 = 45;

struct BitBoardIter {
    bb: BitBoard,
}

// 25 to 40 % faster !!!
pub fn eval(board: &Board) -> i32 {
    let mut result = 0;
    let mut is_endgame = board.combined().popcnt() < 20;

    let white_pieces = board.color_combined(Color::White);
    let black_pieces = board.color_combined(Color::Black);
    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let kings = board.pieces(Piece::King);

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
        result -= ((white_pawns & file_bb).popcnt() as i32 - (black_pawns & file_bb).popcnt() as i32) * DOUBLE_PAWN_SANCTION;
    }

    result
}

pub fn old_eval(board: &Board) -> i32 {
    let mut result = 0;
    let mut is_endgame = board.combined().popcnt() < 20;

    for i in 0..64 {
        let square = unsafe { Square::new(i as u8) };
        let Some((piece, color)) = board.piece_on(square).zip(board.color_on(square)) else {
            continue;
        };
        if color == Color::White {
            result += SQUARE_SCORES[color.to_index()][piece.to_index()][i]
                + PIECE_VALUES[piece.to_index()];
        } else {
            result -= SQUARE_SCORES[color.to_index()][piece.to_index()][i]
                + PIECE_VALUES[piece.to_index()];
        }
        if piece == Piece::Pawn && is_endgame {
            result += ENDGAME_PAWN_SCORES[color.to_index()][i];
        }
    }

    // sanction double pawns
    let pawns = board.pieces(Piece::Pawn);
    let white_pawns = board.color_combined(Color::White) & pawns;
    let black_pawns = board.color_combined(Color::Black) & pawns;

    for file in ALL_FILES {
        let file_bb = get_file(file);
        result -= ((white_pawns & file_bb).popcnt() as i32 - 1) * DOUBLE_PAWN_SANCTION;
        result += ((black_pawns & file_bb).popcnt() as i32 - 1) * DOUBLE_PAWN_SANCTION;
    }

    result
}

pub fn eval_material(board: &Board) -> i32 {
    let mut result = 0;

    let white_pieces = board.color_combined(Color::White);
    let black_pieces = board.color_combined(Color::Black);
    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let kings = board.pieces(Piece::King);

    result += ((pawns & white_pieces).0.count_ones() as i32
        - (pawns & black_pieces).0.count_ones() as i32)
        * PAWN_VALUE;
    result += ((knights & white_pieces).0.count_ones() as i32
        - (knights & black_pieces).0.count_ones() as i32)
        * KNIGHT_VALUE;
    result += ((bishops & white_pieces).0.count_ones() as i32
        - (bishops & black_pieces).0.count_ones() as i32)
        * BISHOP_VALUE;
    result += ((rooks & white_pieces).0.count_ones() as i32
        - (rooks & black_pieces).0.count_ones() as i32)
        * ROOK_VALUE;
    result += ((queens & white_pieces).0.count_ones() as i32
        - (queens & black_pieces).0.count_ones() as i32)
        * QUEEN_VALUE;

    result
}

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

impl BitBoardIter {
    pub fn new(bb: BitBoard) -> Self {
        Self { bb }
    }
}

impl Iterator for BitBoardIter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.bb.0 == 0 {
            None
        } else {
            let index = self.bb.0.trailing_zeros() as usize;
            self.bb.0 &= self.bb.0 - 1;
            Some(index)
        }
    }
}
