pub mod chooser;
pub mod eval;
pub mod gamestate;
pub mod graphics;

use std::collections::HashMap;
use std::ops::Deref;
use chess::*;

#[derive(Clone, Debug)]
pub struct WrappedBoard {
    pub board: Board,
    pub history: HashMap<u64, u8>,
}

impl WrappedBoard {
    pub fn new(board: Board) -> Self {
        let mut history = HashMap::new();
        history.insert(board.get_hash(), 1);
        Self {
            board,
            history,
        }
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
        if self.history.get(&self.board.get_hash()).copied().unwrap_or_default() >= 3 {
            BoardStatus::Stalemate
        } else {
            self.board.status()
        }
    }
}

impl Deref for WrappedBoard {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}

pub fn board_to_fen(board: &Board) -> String {
    let mut fen = String::new();

    // 1. Convert the board's piece positions to the FEN piece placement part
    for rank in ALL_RANKS.into_iter().rev() {
        // Start from rank 8 to 1
        let mut empty_count = 0;
        for file in ALL_FILES {
            let square = Square::make_square(rank, file);
            if let Some((piece, color)) = board.piece_on(square).zip(board.color_on(square)) {
                // If there were empty squares before, push the empty count
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                // Append the piece character to the FEN string
                match piece {
                    Piece::Pawn => fen.push_str(if color == Color::White { "P" } else { "p" }),
                    Piece::Knight => fen.push_str(if color == Color::White { "N" } else { "n" }),
                    Piece::Bishop => fen.push_str(if color == Color::White { "B" } else { "b" }),
                    Piece::Rook => fen.push_str(if color == Color::White { "R" } else { "r" }),
                    Piece::Queen => fen.push_str(if color == Color::White { "Q" } else { "q" }),
                    Piece::King => fen.push_str(if color == Color::White { "K" } else { "k" }),
                }
            } else {
                empty_count += 1;
            }
        }
        // If there are empty squares, append the count to the FEN string
        if empty_count > 0 {
            fen.push_str(&empty_count.to_string());
        }

        // Add a separator between ranks, but not after the last one
        if rank.to_index() > 0 {
            fen.push('/');
        }
    }

    // 2. Add the active color (White or Black)
    let active_color = if board.side_to_move() == Color::White {
        "w"
    } else {
        "b"
    };
    fen.push_str(&format!(" {active_color} "));

    let mut any_castle = false;
    if board.castle_rights(Color::White).has_kingside() {
        any_castle = true;
        fen.push('K');
    }
    if board.castle_rights(Color::White).has_queenside() {
        any_castle = true;
        fen.push('Q');
    }
    if board.castle_rights(Color::Black).has_kingside() {
        any_castle = true;
        fen.push('k');
    }
    if board.castle_rights(Color::Black).has_queenside() {
        any_castle = true;
        fen.push('q');
    }

    if !any_castle {
        fen.push('-');
    }
    fen.push(' ');

    // 4. Add en passant target square
    if let Some(en_passant) = board.en_passant() {
        fen.push_str(&format!("{}", en_passant));
    } else {
        fen.push_str("-");
    }
    fen.push_str(" ");

    // 5. Add the halfmove clock and fullmove number
    // You can track the halfmove clock and fullmove number manually or from some game state
    fen.push_str("0 1"); // Placeholder for halfmove clock and fullmove number

    fen
}

fn main() {
    let board = Board::default();
    let fen = board_to_fen(&board);
    println!("{}", fen); // Prints the FEN of the default starting position
}
