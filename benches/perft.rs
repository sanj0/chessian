use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

use chessian::eval::eval;
use chess::*;

fn perft(board: Board, depth: usize) -> usize {
    black_box(eval(&board));
    if depth == 0 {
        1
    } else if depth == 1 {
        MoveGen::new_legal(&board).collect::<Vec<_>>().len()
    } else {
        let mut result = 0;
        for m in MoveGen::new_legal(&board) {
            let board_then = board.make_move_new(m);
            result += perft(board_then, depth - 1);
        }
        result
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let board = black_box(Board::default());
    c.bench_function("perft 1", |b| b.iter(|| perft(black_box(board.clone()), 1)));
    c.bench_function("perft 2", |b| b.iter(|| perft(black_box(board.clone()), 2)));
    c.bench_function("perft 3", |b| b.iter(|| perft(black_box(board.clone()), 3)));
    c.bench_function("perft 4", |b| b.iter(|| perft(black_box(board.clone()), 4)));
    c.bench_function("perft 5", |b| b.iter(|| perft(black_box(board.clone()), 5)));
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
