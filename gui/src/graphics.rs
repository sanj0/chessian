use chess::{Color as ChessColor, Piece};
use macroquad::prelude::*;

pub struct Textures {
    white_pawn: Image,
    white_knight: Image,
    white_bishop: Image,
    white_rook: Image,
    white_queen: Image,
    white_king: Image,
    black_pawn: Image,
    black_knight: Image,
    black_bishop: Image,
    black_rook: Image,
    black_queen: Image,
    black_king: Image,
}

impl Textures {
    pub async fn load(path: &str, sprite_size: f32) -> Self {
        let pieces = load_image(path).await.unwrap();
        let piece_order = [5., 3., 2., 4., 1., 0.];
        let mut white_pieces = piece_order.into_iter().map(|x| {
            pieces.sub_image(Rect::new(
                x * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            ))
        });
        let mut black_pieces = piece_order
            .into_iter()
            .map(|x| pieces.sub_image(Rect::new(x * sprite_size, 0., sprite_size, sprite_size)));
        Textures {
            white_pawn: white_pieces.next().unwrap(),
            white_knight: white_pieces.next().unwrap(),
            white_bishop: white_pieces.next().unwrap(),
            white_rook: white_pieces.next().unwrap(),
            white_queen: white_pieces.next().unwrap(),
            white_king: white_pieces.next().unwrap(),
            black_pawn: black_pieces.next().unwrap(),
            black_knight: black_pieces.next().unwrap(),
            black_bishop: black_pieces.next().unwrap(),
            black_rook: black_pieces.next().unwrap(),
            black_queen: black_pieces.next().unwrap(),
            black_king: black_pieces.next().unwrap(),
        }
    }

    pub fn get_piece(&self, (piece, color): (Piece, ChessColor)) -> &Image {
        match (piece, color) {
            (Piece::Pawn, ChessColor::White) => &self.white_pawn,
            (Piece::Knight, ChessColor::White) => &self.white_knight,
            (Piece::Bishop, ChessColor::White) => &self.white_bishop,
            (Piece::Rook, ChessColor::White) => &self.white_rook,
            (Piece::Queen, ChessColor::White) => &self.white_queen,
            (Piece::King, ChessColor::White) => &self.white_king,
            (Piece::Pawn, ChessColor::Black) => &self.black_pawn,
            (Piece::Knight, ChessColor::Black) => &self.black_knight,
            (Piece::Bishop, ChessColor::Black) => &self.black_bishop,
            (Piece::Rook, ChessColor::Black) => &self.black_rook,
            (Piece::Queen, ChessColor::Black) => &self.black_queen,
            (Piece::King, ChessColor::Black) => &self.black_king,
        }
    }
}
