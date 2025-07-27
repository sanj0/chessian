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
        Textures {
            white_pawn: pieces.sub_image(Rect::new(
                5. * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            )),
            white_knight: pieces.sub_image(Rect::new(
                3. * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            )),
            white_bishop: pieces.sub_image(Rect::new(
                2. * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            )),
            white_rook: pieces.sub_image(Rect::new(
                4. * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            )),
            white_queen: pieces.sub_image(Rect::new(
                1. * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            )),
            white_king: pieces.sub_image(Rect::new(
                0. * sprite_size,
                sprite_size,
                sprite_size,
                sprite_size,
            )),

            black_pawn: pieces.sub_image(Rect::new(5. * sprite_size, 0., sprite_size, sprite_size)),
            black_knight: pieces.sub_image(Rect::new(
                3. * sprite_size,
                0.,
                sprite_size,
                sprite_size,
            )),
            black_bishop: pieces.sub_image(Rect::new(
                2. * sprite_size,
                0.,
                sprite_size,
                sprite_size,
            )),
            black_rook: pieces.sub_image(Rect::new(4. * sprite_size, 0., sprite_size, sprite_size)),
            black_queen: pieces.sub_image(Rect::new(
                1. * sprite_size,
                0.,
                sprite_size,
                sprite_size,
            )),
            black_king: pieces.sub_image(Rect::new(0. * sprite_size, 0., sprite_size, sprite_size)),
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
