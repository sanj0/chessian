use std::str::FromStr;
use crate::*;
use crate::chooser::*;

pub struct TestCase {
    pub board: HistoryBoard,
    pub solution: ChessMove,
    pub id: String,
}

pub fn load_test_suite(src: &str) -> Vec<TestCase> {
    src.lines().map(|l| TestCase::parse(l).unwrap()).collect()
}

impl TestCase {
    // r1bqk1r1/1p1p1n2/p1n2pN1/2p1b2Q/2P1Pp2/1PN5/PB4PP/R4RK1 w q - - bm Rxf4; id "ERET 001 - Relief";
    pub fn parse(line: &str) -> Result<Self, String> {
        let bm_idx = line.find("bm").or_else(|| line.find("am")).ok_or_else(|| format!("missing `bm` in '{line}'"))?;
        let semi_idx = line.find(";").ok_or_else(|| format!("missing `;` in '{line}'"))?;
        let fen = &line[0..bm_idx];
        let solution_str = &line[bm_idx + 3..semi_idx];
        let id_str = &line[semi_idx + 6..line.len()-2];
        let board = Board::from_str(fen).map_err(|e| format!("{e}"))?;
        Ok(Self {
            board: HistoryBoard::new(board),
            solution: ChessMove::from_san(&board, solution_str).map_err(|e| format!("{e}"))?,
            id: String::from(id_str),
        })
    }
}

pub fn eigenmann() -> usize {
    let src = std::fs::read_to_string("eigenmann.txt").expect("eigenmann.txt missing");
    let test_suite = load_test_suite(&src);
    let mut score = 0;
    for case in &test_suite {
        println!("--- {} ---", case.id);
        let engine_move = chooser::best_move(&case.board, TimeControl::new(None, TCMode::MoveTime(15_000)), &[], std::io::stdout(), std::io::sink()).unwrap().best_move;
        println!("    solution: {}", case.solution);
        println!("    engine: {engine_move}");
        if case.solution == engine_move {
            score += 1;
        }
    }
    score
}

