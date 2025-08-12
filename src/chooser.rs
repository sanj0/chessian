use std::collections::HashMap;
use std::io::{self, Write, BufWriter};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Instant;

use chess::*;

use crate::eval::*;

pub const MATE_SCORE: i32 = 30_000;
pub const INF: i32 = MATE_SCORE * 2;

#[derive(Clone, Debug)]
pub enum TimeControl {
    MoveTime(u128),
    Depth(usize),
    Infinite {
        stop_flag: Arc<AtomicBool>,
    },
}

pub struct ChooserResult {
    pub best_move: ChessMove,
    pub response: Option<ChessMove>,
    pub deep_eval: i32,
    pub reached_depth: usize,
    pub millis: u128,
}

pub fn best_move(
    board: &Board,
    time_control: TimeControl,
    exclude_moves: &[ChessMove],
    mut uci_sink: impl Write,
    mut log: impl Write,
) -> Option<ChooserResult> {
    let mut candidates: Vec<_> = MoveGen::new_legal(board)
        .filter(|m| !exclude_moves.contains(m))
        .collect();
    let num_candidates = candidates.len();
    if num_candidates == 1 {
        return Some(ChooserResult::new(candidates[0], None, -1, 0, 0));
    }
    let mut best_move = None;
    let mut best_alpha = -INF;
    let mut response = None;

    sort_moves(&mut candidates, board);

    let t0 = Instant::now();
    let mut depth = 1;
    'outer: loop {
        let mut alpha = -INF;
        let mut curr_best_move = None;
        let mut curr_response = None;
        let mut curr_best_move_index = 0;
        write!(log, "\ndepth {depth}");
        for (i, m) in candidates.iter().enumerate() {
            let after_move = board.make_move_new(*m);
            let (alpha_opt, response_opt) = negamax(
                &after_move,
                depth,
                -INF,
                -alpha,
                &time_control,
                &t0,
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
        depth += 2;
        if curr_best_move.is_some() {
            let m = candidates.remove(curr_best_move_index);
            candidates.insert(0, m);
        }
        best_move = curr_best_move;
        response = curr_response;
        best_alpha = alpha;
        if time_control.should_stop(t0.elapsed().as_millis(), depth - 2) {
            break;
        }
    }
    if let Some(m) = best_move {
        writeln!(log, "chose {m} at depth {depth}\n");
    }
    best_move
        .map(|m| ChooserResult::new(m, response, best_alpha, depth - 2, t0.elapsed().as_millis()))
}

// None if ran out of time
fn negamax(
    board: &Board,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    time_control: &TimeControl,
    t0: &Instant,
) -> (Option<i32>, Option<ChessMove>) {
    if depth == 0 {
        return (
            Some(if board.side_to_move() == Color::White {
                eval(board)
            } else {
                -eval(board)
            }),
            None,
        );
    }
    // Claim 0 depth because depth stopping only happens in the root search
    if time_control.should_stop(t0.elapsed().as_millis(), 0) {
        return (None, None);
    }
    match board.status() {
        BoardStatus::Checkmate => (Some(-MATE_SCORE - depth as i32), None),
        BoardStatus::Stalemate => {
            let eval = if board.side_to_move() == Color::White {
                eval(board)
            } else {
                -eval(board)
            };
            (
                Some(if eval < -(PIECE_VALUES[2]) {
                    -(MATE_SCORE / 2)
                } else {
                    MATE_SCORE / 2
                }),
                None,
            )
        }
        BoardStatus::Ongoing => {
            let mut moves = MoveGen::new_legal(board).collect::<Vec<_>>();
            if depth != 1 {
                sort_moves(&mut moves, board);
            }
            let mut response = None;
            for m in moves {
                let after_move = board.make_move_new(m);
                let value = negamax(
                    &after_move,
                    depth - 1,
                    -beta,
                    -alpha,
                    time_control,
                    t0,
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
    if before.make_move_new(*m).checkers().0 != 0 {
        PIECE_VALUES[5] * 10 - PIECE_VALUES[before.piece_on(m.get_source()).unwrap().to_index()]
    } else {
        let pos_score = SQUARE_SCORES[before.side_to_move().to_index()]
            [get_piece(m, before).to_index()][m.get_dest().to_index()];
        pos_score
            + get_capture(m, before)
                .map(|p| PIECE_VALUES[p.to_index()])
                .unwrap_or(-1000)
    }
}

fn sort_moves(moves: &mut [ChessMove], context: &Board) {
    moves.sort_by(|a, b| get_move_prio(b, context).cmp(&get_move_prio(a, context)));
}

impl TimeControl {
    pub fn game_time(base: u128, increment: u128, left: u128) -> Self {
        Self::MoveTime(u128::min(base / 20 + increment / 2, left))
    }

    pub fn should_stop(&self, elapsed: u128, reached_depth: usize) -> bool {
        match self {
            Self::MoveTime(millis) => elapsed >= *millis,
            Self::Depth(depth) => reached_depth >= *depth,
            Self::Infinite {
                stop_flag,
            } => stop_flag.load(Ordering::Relaxed),
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
