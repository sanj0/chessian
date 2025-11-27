use std::collections::HashMap;
use std::io::{self, Write, BufWriter};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Instant;

use chess::*;

use crate::eval::*;
use crate::WrappedBoard;

pub const MATE_SCORE: i32 = 30_000;
pub const INF: i32 = MATE_SCORE * 2;

#[derive(Clone, Debug)]
pub struct TimeControl {
    stop_flag: Option<Arc<AtomicBool>>,
    mode: TCMode,
}

#[derive(Clone, Debug)]
pub enum TCMode {
    MoveTime(u128),
    Depth(usize),
    Infinite,
}

pub struct ChooserResult {
    pub best_move: ChessMove,
    pub response: Option<ChessMove>,
    pub deep_eval: i32,
    pub reached_depth: usize,
    pub millis: u128,
}

pub fn best_move(
    board: &WrappedBoard,
    time_control: TimeControl,
    exclude_moves: &[ChessMove],
    mut uci_sink: impl Write,
    mut log: impl Write,
) -> Option<ChooserResult> {
    let mut candidates: Vec<_> = MoveGen::new_legal(&board.board)
        .filter(|m| !exclude_moves.contains(m))
        .collect();
    let num_candidates = candidates.len();
    if num_candidates == 1 {
        return Some(ChooserResult::new(candidates[0], None, -1, 0, 0));
    }
    let mut best_move = None;
    let mut best_alpha = -INF;
    let mut response = None;

    sort_moves(&mut candidates, &board.board);

    let t0 = Instant::now();
    let mut depth = 1;
    'outer: loop {
        let mut node_count = 0;
        let mut alpha = -INF;
        let mut curr_best_move = None;
        let mut curr_response = None;
        let mut curr_best_move_index = 0;
        write!(log, "\ndepth {depth}");
        for (i, m) in candidates.iter().enumerate() {
            let after_move = board.make_move(*m);
            let (alpha_opt, response_opt) = negamax(
                &after_move,
                depth,
                -INF,
                -alpha,
                &time_control,
                &t0,
                &mut node_count,
            );
            let Some(its_alpha) = alpha_opt.map(|i| -i) else {
                write!(log, "\nout of time!");
                if alpha > best_alpha && best_move != curr_best_move {
                    best_move = curr_best_move;
                    response = response_opt;
                    best_alpha = alpha;
                }
                break 'outer;
            };
            write!(
                log,
                "\r{:.2} % depth {depth}",
                (i + 1) as f32 / num_candidates as f32 * 100.0
            );
            let _ = io::stdout().flush();
            if its_alpha > alpha {
                curr_best_move = Some(*m);
                curr_response = response_opt;
                curr_best_move_index = i;
                alpha = its_alpha;
            }
            if alpha >= MATE_SCORE {
                writeln!(log, "!!! MATE AT DEPTH {} !!!", depth);
                best_move = curr_best_move;
                response = response_opt;
                best_alpha = alpha;
                break 'outer;
            }
        }
        writeln!(
            log,
            "\nbest move position: {} / {num_candidates}",
            curr_best_move_index + 1
        );
        if alpha <= -MATE_SCORE {
            writeln!(log, "!!! WE LOSE IN MATE IN {} !!!", depth);
            break;
        }
        let time = t0.elapsed().as_millis();
        writeln!(uci_sink, "info depth 2 seldepth {depth} multipv 1 score cp {alpha}  nodes {node_count} nps {:.0} time {time} pv {} {}", node_count as f32 / (time as f32 / 1000.0), curr_best_move.unwrap(), curr_response.unwrap());
        depth += 1;
        if curr_best_move.is_some() {
            let m = candidates.remove(curr_best_move_index);
            candidates.insert(0, m);
        }
        best_move = curr_best_move;
        response = curr_response;
        best_alpha = alpha;
        if time_control.should_stop(time, depth - 1) {
            break;
        }
    }
    if let Some(m) = best_move {
        writeln!(log, "chose {m} at depth {depth}\n");
    }
    best_move
        .map(|m| ChooserResult::new(m, response, best_alpha, depth - 1, t0.elapsed().as_millis()))
}

// None if ran out of time
fn negamax(
    board: &WrappedBoard,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    time_control: &TimeControl,
    t0: &Instant,
    node_count: &mut usize,
) -> (Option<i32>, Option<ChessMove>) {
    if depth == 0 {
        *node_count += 1;
        let (score, qdepth) = qsearch(board, alpha, beta, 0);
        return (Some(score), None);
        //return (
        //    Some(if board.board.side_to_move() == Color::White {
        //        eval(&board.board)
        //    } else {
        //        -eval(&board.board)
        //    }),
        //    None,
        //);
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
                    (MATE_SCORE / 2)
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

fn qsearch(board: &WrappedBoard, mut alpha: i32, beta: i32, reached_depth: usize) -> (i32, usize) {
    match board.status() {
        BoardStatus::Checkmate => (-MATE_SCORE, reached_depth),
        BoardStatus::Stalemate => {
            let eval = if board.board.side_to_move() == Color::White {
                eval(&board.board)
            } else {
                -eval(&board.board)
            };
            if eval < -(PIECE_VALUES[2]) {
                ((MATE_SCORE / 2), reached_depth)
            } else {
                (-(MATE_SCORE / 2), reached_depth)
            }
        }
        BoardStatus::Ongoing => {
            let stand_pat = if board.board.side_to_move() == Color::White {
                eval(&board.board)
            } else {
                -eval(&board.board)
            };
            if stand_pat >= beta {
                return (beta, reached_depth);
            }
            if stand_pat > alpha {
                alpha = stand_pat;
            }
            let mut moves = MoveGen::new_legal(&board.board).filter(|m| !is_quiet(m, board)).collect::<Vec<_>>();
            sort_moves(&mut moves, &board.board);
            let mut reached_depth = reached_depth;
            for m in moves {
                let after_move = board.make_move(m);
                let (mut value, depth) = qsearch(
                    &after_move,
                    -beta,
                    -alpha,
                    reached_depth + 1,
                );
                value = -value;
                reached_depth = usize::max(reached_depth, depth);
                if value >= beta {
                    return (beta, reached_depth);
                }
                if value > alpha {
                    alpha = value;
                }
            }
            (alpha, reached_depth)
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
    moves.sort_by(|a, b| get_move_prio(b, context).cmp(&get_move_prio(a, context)));
}

impl TimeControl {
    pub fn new(stop_flag: Option<Arc<AtomicBool>>, mode: TCMode) -> Self {
        Self {
            stop_flag,
            mode,
        }
    }

    //pub fn game_time(base: u128, increment: u128, left: u128) -> Self {
    //    Self::MoveTime(u128::min(base / 20 + increment / 2, left))
    //}

    pub fn should_stop(&self, elapsed: u128, reached_depth: usize) -> bool {
        if self.stop_flag.as_ref().map(|b| b.load(Ordering::Relaxed)).unwrap_or(false) {
            true
        } else {
            match self.mode {
                TCMode::MoveTime(millis) => elapsed >= millis,
                TCMode::Depth(depth) => reached_depth >= depth,
                TCMode::Infinite => false,
            }
        }
    }
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
