use std::io::{Write};
use std::time::Instant;

use chess::*;

use crate::historyboard::HistoryBoard;
use crate::eval::*;
use crate::timecontrol::*;

pub const MATE_SCORE: i32 = 30_000;
pub const INF: i32 = MATE_SCORE * 2;

pub struct ChooserResult {
    pub best_move: ChessMove,
    pub response: Option<ChessMove>,
    pub deep_eval: i32,
    pub reached_depth: usize,
    pub millis: u128,
}

/// Most important function of the engine: Choose the best from in the given position.
pub fn best_move(
    board: &HistoryBoard,
    time_control: TimeControl,
    mut uci_sink: impl Write,
    mut log: impl Write,
) -> Option<ChooserResult> {
    let mut candidates: Vec<_> = MoveGen::new_legal(&board.board).collect();
    let mut best_move = None;
    let mut best_alpha = -INF;
    let mut response = None;

    sort_moves(&mut candidates, &board.board);

    let t0 = Instant::now();
    let mut current_depth = 1;
    'outer: loop {
        let mut node_count = 0;
        let mut alpha = -INF;
        let mut curr_best_move = None;
        let mut curr_response = None;
        let mut curr_best_move_index = 0;
        for (i, m) in candidates.iter().enumerate() {
            let board_after_move = board.make_move(*m);
            let (alpha_opt, response_opt) = negamax(
                &board_after_move,
                current_depth,
                -INF,
                -alpha,
                &time_control,
                &t0,
                &mut node_count,
            );
            let Some(current_move_alpha) = alpha_opt.map(|i| -i) else {
                let _ = write!(log, "\nout of time!");
                if alpha > best_alpha && best_move != curr_best_move {
                    best_move = curr_best_move;
                    response = response_opt;
                    best_alpha = alpha;
                }
                break 'outer;
            };
            if current_move_alpha > alpha {
                curr_best_move = Some(*m);
                curr_response = response_opt;
                curr_best_move_index = i;
                alpha = current_move_alpha;
            }
            if alpha >= MATE_SCORE {
                let _ = writeln!(log, "!!! MATE AT DEPTH {} !!!", current_depth);
                best_move = curr_best_move;
                response = response_opt;
                best_alpha = alpha;
                break 'outer;
            }
        }
        if alpha <= -MATE_SCORE {
            let _ = writeln!(log, "!!! WE LOSE IN MATE IN {} !!!", current_depth);
            break;
        }
        let time = t0.elapsed().as_millis();
        let _ =  writeln!(
            uci_sink,
            "info depth 2 seldepth {current_depth} multipv 1 score cp {alpha} nodes {node_count} nps {:.0} time {time} pv {} {}",
            node_count as f32 / (time as f32 / 1000.0),
            curr_best_move.unwrap(),
            curr_response.unwrap()
        );
        current_depth += 1;
        candidates.swap(0, curr_best_move_index);
        best_move = curr_best_move;
        response = curr_response;
        best_alpha = alpha;
        if time_control.should_stop(time, current_depth - 1) {
            break;
        }
    }
    best_move
        .map(|m| ChooserResult::new(m, response, best_alpha, current_depth - 1, t0.elapsed().as_millis()))
}

// None if ran out of time
fn negamax(
    board: &HistoryBoard,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    time_control: &TimeControl,
    t0: &Instant,
    node_count: &mut usize,
) -> (Option<i32>, Option<ChessMove>) {
    if depth == 0 {
        *node_count += 1;
        let score = qsearch(board, alpha, beta);
        return (Some(score), None);
    }
    // Claim 0 depth because depth stopping only happens in the root search
    if time_control.should_stop(t0.elapsed().as_millis(), 0) {
        return (None, None);
    }
    match board.status() {
        BoardStatus::Checkmate => (Some(-MATE_SCORE), None),
        BoardStatus::Stalemate => {
            let eval = if board.board.side_to_move() == Color::White {
                eval(&board.board)
            } else {
                -eval(&board.board)
            };
            (
                Some(if eval < -(PIECE_VALUES[2]) {
                    MATE_SCORE / 2
                } else {
                    -(MATE_SCORE / 2)
                }),
                None,
            )
        }
        BoardStatus::Ongoing => {
            let mut moves = MoveGen::new_legal(&board.board).collect::<Vec<_>>();
            if depth != 1 {
                sort_moves(&mut moves, &board.board);
            }
            let mut response = None;
            for m in moves {
                let after_move = board.make_move(m);
                let value = negamax(
                    &after_move,
                    depth - 1,
                    -beta,
                    -alpha,
                    time_control,
                    t0,
                    node_count,
                );
                let Some(mut value) = value.0 else {
                    return (None, None);
                };
                value = -value;
                if value >= beta {
                    return (Some(beta), None);
                }
                if value > alpha {
                    alpha = value;
                    response = Some(m);
                }
            }
            (Some(alpha), response)
        }
    }
}

fn qsearch(board: &HistoryBoard, mut alpha: i32, beta: i32) -> i32 {
    match board.status() {
        BoardStatus::Checkmate => -MATE_SCORE,
        BoardStatus::Stalemate => {
            let eval = if board.board.side_to_move() == Color::White {
                eval(&board.board)
            } else {
                -eval(&board.board)
            };
            if eval < -(PIECE_VALUES[2]) {
                MATE_SCORE / 2
            } else {
                -(MATE_SCORE / 2)
            }
        }
        BoardStatus::Ongoing => {
            let stand_pat = if board.board.side_to_move() == Color::White {
                eval(&board.board)
            } else {
                -eval(&board.board)
            };
            if stand_pat >= beta {
                return beta;
            }
            if stand_pat > alpha {
                alpha = stand_pat;
            }
            let mut moves = MoveGen::new_legal(&board.board)
                .filter(|m| !is_quiet(m, board))
                .collect::<Vec<_>>();
            sort_moves(&mut moves, &board.board);
            for m in moves {
                let after_move = board.make_move(m);
                let mut value = qsearch(&after_move, -beta, -alpha);
                value = -value;
                if value >= beta {
                    return beta;
                }
                if value > alpha {
                    alpha = value;
                }
            }
            alpha
        }
    }
}

fn is_quiet(m: &ChessMove, board: &Board) -> bool {
    get_relative_capture_value(m, board) < 0
}

fn get_piece(m: &ChessMove, board: &Board) -> Piece {
    board.piece_on(m.get_source()).unwrap()
}

fn get_capture(m: &ChessMove, board: &Board) -> Option<Piece> {
    board.piece_on(m.get_dest())
}

fn get_capture_value(m: &ChessMove, board: &Board) -> i32 {
    get_capture(m, board)
        .map(|p| PIECE_VALUES[p.to_index()])
        .unwrap_or(0)
}

fn get_relative_capture_value(m: &ChessMove, board: &Board) -> i32 {
    get_capture_value(m, board) - PIECE_VALUES[get_piece(m, board).to_index()]
}

fn get_move_prio(m: &ChessMove, before: &Board) -> i32 {
    let pos_score = SQUARE_SCORES[before.side_to_move().to_index()]
        [get_piece(m, before).to_index()][m.get_dest().to_index()];
    pos_score + get_capture_value(m, before)
}

fn sort_moves(moves: &mut [ChessMove], context: &Board) {
    moves.sort_by_key(|m| -get_move_prio(m, context));
}


impl ChooserResult {
    pub fn new(
        best_move: ChessMove,
        response: Option<ChessMove>,
        deep_eval: i32,
        reached_depth: usize,
        millis: u128,
    ) -> Self {
        Self {
            best_move,
            response,
            deep_eval,
            reached_depth,
            millis,
        }
    }
}
