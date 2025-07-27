use std::time::Instant;
use std::io::{self, Write};
use std::collections::HashMap;

use chess::*;

use crate::eval::*;

pub const MATE_SCORE: i32 = 30_000;
pub const INF: i32 = MATE_SCORE * 2;

pub struct ChooserResult {
    pub best_move: ChessMove,
    pub deep_eval: i32,
    pub reached_depth: usize,
    pub millis: u128,
}

pub fn best_move(board: &Board, start_depth: usize, millis: u128) -> Option<ChooserResult> {
    let mut candidates: Vec<_> = MoveGen::new_legal(board).collect();
    let num_candidates = candidates.len();
    if num_candidates == 1 {
        return Some(ChooserResult::new(candidates[0], -1, 0, 0));
    }
    let mut best_move = None;
    let mut best_alpha = -INF;
    let cutoff_millis = millis / 2;

    sort_moves(&mut candidates, board);

    let t0 = Instant::now();
    let mut depth = start_depth;
    'outer: loop {
        let mut alpha = -INF;
        let mut curr_best_move = None;
        let mut curr_best_move_index = 0;
        print!("\ndepth {depth}");
        for (i, m) in candidates.iter().enumerate() {
            let after_move = board.make_move_new(*m);
            let Some(its_alpha) = negamax(&after_move, depth - 1, -INF, -alpha, &millis, &t0, i > 2*num_candidates/3).map(|i| -i) else {
                println!("\nout of time!");
                break 'outer;
            };
            print!("\r{:.2} % depth {depth}", (i+1) as f32 / num_candidates as f32 * 100.0);
            let _ = io::stdout().flush();
            if its_alpha > alpha {
                curr_best_move = Some(*m);
                curr_best_move_index = i;
                alpha = its_alpha;
            }
            if alpha >= MATE_SCORE {
                println!("!!! MATE AT DEPTH {} !!!", depth);
                best_move = curr_best_move;
                best_alpha = alpha;
                break 'outer;
            }
        }
        println!("\nbest move position: {} / {num_candidates}", curr_best_move_index+1);
        depth += 1;
        if alpha <= -MATE_SCORE {
            println!("!!! WE LOSE IN MATE IN {} !!!", depth);
            break;
        }
        if curr_best_move.is_some() {
            let m = candidates.remove(curr_best_move_index);
            candidates.insert(0, m);
        }
        best_move = curr_best_move;
        best_alpha = alpha;
    }
    if let Some(m) = best_move {
        println!("chose {m} at depth {depth}\n");
    }
    best_move.map(|m| ChooserResult::new(m, best_alpha, depth - 1, t0.elapsed().as_millis()))
}

// None if ran out of time
fn negamax(board: &Board, depth: usize, mut alpha: i32, beta: i32, millis: &u128, t0: &Instant, ignore_time: bool) -> Option<i32> {
    if !ignore_time && t0.elapsed().as_millis() >= *millis {
        return None;
    }
    if depth == 0 {
        return Some(if board.side_to_move() == Color::White {
            eval(board)
        } else {
            -eval(board)
        });
    }
    Some(match board.status() {
        BoardStatus::Checkmate => -MATE_SCORE - depth as i32,
        BoardStatus::Stalemate => {
            let eval = if board.side_to_move() == Color::White {
                eval(board)
            } else {
                -eval(board)
            };
            if eval < -(PIECE_VALUES[2]) {
                -(MATE_SCORE / 2)
            } else {
                MATE_SCORE / 2
            }
        }
        BoardStatus::Ongoing => {
            for m in MoveGen::new_legal(board) {
                let after_move = board.make_move_new(m);
                alpha = i32::max(alpha, -negamax(&after_move, depth - 1, -beta, -alpha, millis, t0, ignore_time)?);
                if alpha >= beta {
                    break;
                }
            }
            alpha
        }
    })
}

fn is_quiet(m: &ChessMove, board: &Board) -> bool {
    get_capture(m, board).is_none()
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

fn get_move_prio(m: &ChessMove, before: &Board) -> i32 {
    if before.make_move_new(*m).checkers().0 != 0 {
        PIECE_VALUES[5] * 10 - PIECE_VALUES[before.piece_on(m.get_source()).unwrap().to_index()]
    } else {
        let pos_score = SQUARE_SCORES[before.side_to_move().to_index()][get_piece(m, before).to_index()][m.get_dest().to_index()];
        pos_score + get_capture(m, before)
            .map(|p| PIECE_VALUES[p.to_index()])
            .unwrap_or(-1000)
    }
}

fn sort_moves(moves: &mut [ChessMove], context: &Board) {
    moves.sort_by(|a, b| get_move_prio(b, context).cmp(&get_move_prio(a, context)));
}

impl ChooserResult {
    pub fn new(best_move: ChessMove, deep_eval: i32, reached_depth: usize, millis: u128) -> Self {
        Self {
            best_move,
            deep_eval,
            reached_depth,
            millis,
        }
    }
}
