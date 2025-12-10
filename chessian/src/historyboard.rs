use chess::*;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct HistoryBoard {
    pub board: Board,
    pub history: HashMap<u64, u8>,
}

impl HistoryBoard {
    pub fn new(board: Board) -> Self {
        let mut history = HashMap::new();
        history.insert(board.get_hash(), 1);
        Self { board, history }
    }

    pub fn make_move(&self, m: ChessMove) -> Self {
        let new_board = self.board.make_move_new(m);
        let mut history = self.history.clone();
        *(history.entry(new_board.get_hash()).or_insert(0)) += 1;
        Self {
            board: new_board,
            history,
        }
    }

    pub fn status(&self) -> BoardStatus {
        if self
            .history
            .get(&self.board.get_hash())
            .copied()
            .unwrap_or_default()
            >= 3
        {
            BoardStatus::Stalemate
        } else {
            self.board.status()
        }
    }
}

impl Deref for HistoryBoard {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}

