use std::str::FromStr;

use chess::*;

use chessian::historyboard::HistoryBoard;
use chessian::chooser::*;

pub struct GameState {
    board: HistoryBoard,
    legal_moves: Vec<ChessMove>,
    exclude_moves: Vec<ChessMove>,
    last_engine_move: Option<ChessMove>,
    undo_queue: Vec<(HistoryBoard, ChessMove)>,
    redo_queue: Vec<(HistoryBoard, ChessMove)>,
    last_move: Option<ChessMove>,
}

impl GameState {
    pub fn from_board(board: Board) -> Self {
        Self {
            board: HistoryBoard::new(board),
            legal_moves: MoveGen::new_legal(&board).collect(),
            exclude_moves: Vec::new(),
            last_engine_move: None,
            undo_queue: Vec::new(),
            redo_queue: Vec::new(),
            last_move: None,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        Board::from_str(fen)
            .map(Self::from_board)
            .map_err(|e| format!("{e}"))
    }

    pub fn board(&self) -> &HistoryBoard {
        &self.board
    }

    pub fn legal_moves_from(&self, square: Square) -> Vec<ChessMove> {
        self.legal_moves
            .iter()
            .filter(|m| m.get_source() == square)
            .cloned()
            .collect()
    }

    pub fn make_move(&mut self, m: ChessMove) {
        self.undo_queue.push((self.board.clone(), m));
        self.redo_queue.clear();
        self.board = self.board.make_move(m);
        self.get_legal_moves();
        self.last_move = Some(m);
    }

    pub fn engine_move(&mut self, time_control: TimeControl) -> Option<ChooserResult> {
        if let Some(result) = best_move(
            &self.board,
            time_control,
            &self.exclude_moves,
            std::io::stdout(),
            std::io::sink(),
        ) {
            self.make_move(result.best_move);
            self.last_engine_move = Some(result.best_move);
            if let Some(r) = result.response {
                println!("I'm predicting {r}");
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn excluded_moves(&mut self) -> &mut Vec<ChessMove> {
        &mut self.exclude_moves
    }

    pub fn last_engine_move(&mut self) -> Option<ChessMove> {
        self.last_engine_move
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some((b, m)) = self.undo_queue.pop() {
            self.redo_queue
                .push((self.board.clone(), self.last_move.unwrap()));
            self.board = b;
            self.last_move = Some(m);
            self.get_legal_moves();
            true
        } else {
            false
        }
    }

    pub fn redo_move(&mut self) -> bool {
        if let Some((b, m)) = self.redo_queue.pop() {
            self.undo_queue
                .push((self.board.clone(), self.last_move.unwrap()));
            self.board = b;
            self.last_move = Some(m);
            self.get_legal_moves();
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_queue.is_empty()
    }

    pub fn history(&self) -> &Vec<(HistoryBoard, ChessMove)> {
        &self.undo_queue
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_queue.is_empty()
    }

    pub fn get_legal_moves(&mut self) {
        self.legal_moves = MoveGen::new_legal(&self.board.board).collect();
    }

    pub fn last_move(&self) -> Option<ChessMove> {
        self.last_move
    }
}

impl std::default::Default for GameState {
    fn default() -> Self {
        Self::from_board(Board::default())
    }
}
