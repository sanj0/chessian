use std::env::args;
use std::io::{self, Write};
use std::str::FromStr;

use chess::Color as ChessColor;
use chess::*;
use macroquad::color::Color;
use macroquad::input::KeyCode;
use macroquad::prelude::*;
use macroquad::ui::*;

use gamestate::GameState;
use graphics::Textures;

use chessian::*;
use chessian::eval::*;

pub const FIELD_SIZE: f32 = 100.0;
pub const COLOR_WHITE: Color = Color::from_hex(0xFFFFF2);
pub const COLOR_BLACK: Color = Color::from_hex(0xFFC0CB);
pub const COLOR_BLUE: Color = Color::from_hex(0xB3EBF2);
pub const COLOR_RED: Color = Color::from_hex(0xFF746C);
pub const MOVE_INDICATOR_SIZE: f32 = 15.0;
pub const COLOR_MOVES: Color = Color::new(1., 0.1, 0.1, 0.5);

pub const UI_WIDTH: f32 = 200.0;
pub const UI_ID_CHECKBOX: Id = 0;
pub const UI_ID_CHECKBOX_DSN: Id = 2;
pub const UI_ID_CHECKBOX_DP: Id = 3;
pub const UI_ID_SLIDER: Id = 4;
pub const UI_ID_UNDO_REDO_GROUP: Id = 1;

#[macroquad::main(conf)]
async fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let mut game_state = if let Some(fen) = args.nth(1) {
        GameState::from_fen(&fen)?
    } else {
        GameState::default()
    };

    if matches![args.next(), Some(s) if s == "go"] {
        let Some(Ok(millis)) = args.next().map(|s| s.parse()) else {
            return Err(String::from("Expected <fen> go <millis>"));
        };
        match game_state.board().status() {
            BoardStatus::Ongoing => {
                let Some(result) = chessian::chooser::best_move(game_state.board(), 1, millis) else {
                    return Err(String::from("error"));
                };
                println!("{}", board_to_fen(&game_state.board().make_move_new(result.best_move)));
            }
            BoardStatus::Stalemate => println!("stalemate"),
            BoardStatus::Checkmate => println!("lost"),
        }
        return Ok(());
    }

    let piece_sprites = Textures::load("pieces.png", 16.0).await;
    let mut highlight_moves: Vec<ChessMove> = Vec::new();
    let mut auto_respond = true;
    let mut engine_move_next_frame = false;
    let mut last_alpha = None;
    let mut last_depth = None;
    let mut last_millis = None;
    let mut draw_square_names = true;
    let mut draw_pieces = true;
    let mut thinking_millis = 5_000;
    let mut invert = false;
    let mut total_time = 0;

    loop {
        clear_background(BLACK);

        let hovered_square = hovered_square(invert);

        for mut y in 0..=7 {
            for mut x in 0..=7 {
                let square = Square::make_square(
                    Rank::from_index(if invert { y } else { 7 - y }),
                    File::from_index(if invert { 7 - x } else { x }),
                );
                let x_pos = x as f32 * FIELD_SIZE;
                let y_pos = y as f32 * FIELD_SIZE;
                let (color, opp_color) = if (x + y) % 2 == 0 {
                    (COLOR_WHITE, COLOR_BLACK)
                } else {
                    (COLOR_BLACK, COLOR_WHITE)
                };
                // Draw field
                draw_rectangle(x_pos, y_pos, FIELD_SIZE, FIELD_SIZE, color);
                if square == hovered_square {
                    draw_rectangle_lines(x_pos, y_pos, FIELD_SIZE, FIELD_SIZE, 7.5, COLOR_BLUE);
                }
                // Draw piece?
                if draw_pieces {
                    if let Some((piece, color)) = game_state
                        .board()
                        .piece_on(square)
                        .zip(game_state.board().color_on(square))
                    {
                        draw_piece(piece, color, x_pos, y_pos, &piece_sprites);
                    }
                }

                if draw_square_names {
                    draw_text(
                        &square.to_string(),
                        x_pos,
                        y_pos + FIELD_SIZE,
                        20.0,
                        opp_color,
                    );
                }

                if let Some(m) = game_state.last_move() {
                    if m.get_source() == square || m.get_dest() == square {
                        draw_rectangle_lines(x_pos, y_pos, FIELD_SIZE, FIELD_SIZE, 7.5, COLOR_RED);
                    }
                }
            }
        }

        if engine_move_next_frame {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 0.75),
            );
            draw_text_centered("Engine calculates ...", 35.0, COLOR_BLUE);
            next_frame().await;
            if let Some(result) = game_state.engine_move(thinking_millis) {
                last_alpha = Some(result.deep_eval);
                last_depth = Some(result.reached_depth);
                last_millis = Some(result.millis);
                total_time += result.millis;
                println!("{:.2}s total time thinking", total_time as f64 / 1000.0);
            }
            engine_move_next_frame = false;
            highlight_moves.clear();
            continue;
        }

        // Draw highlighted moves
        for m in &highlight_moves {
            let dest = m.get_dest();
            let (x, y) = square_to_xy(if invert { invert_square(dest) } else { dest });
            draw_circle(
                x + FIELD_SIZE / 2.,
                y + FIELD_SIZE / 2.,
                MOVE_INDICATOR_SIZE,
                COLOR_MOVES,
            );
        }

        // Process input
        if is_mouse_button_down(MouseButton::Left) {
            if matches![
                hovered_piece(game_state.board(), invert),
                Some((_, color)) if color == game_state.board().side_to_move()]
            {
                highlight_moves = game_state.legal_moves_from(hovered_square);
            } else {
                if let Some(m) = highlight_moves
                    .iter()
                    .find(|m| m.get_dest() == hovered_square)
                {
                    let mut mov = *m;
                    if mov.get_promotion().is_some() {
                        mov = ChessMove::new(
                            mov.get_source(),
                            mov.get_dest(),
                            Some(choose_promotion()),
                        );
                    }
                    game_state.make_move(mov);
                    engine_move_next_frame = auto_respond;
                }
                highlight_moves.clear();
            }
        }

        if let Some(c) = get_char_pressed() {
            let control_down = if cfg!(target_os = "macos") {
                is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper)
            } else {
                is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
            };
            match c {
                'a' => auto_respond = !auto_respond,
                'f' => println!("{}", chessian::board_to_fen(game_state.board())),
                'm' => {
                    engine_move_next_frame = true;
                    highlight_moves.clear();
                }
                'z' if control_down => {
                    if game_state.undo_move() {
                        highlight_moves.clear();
                    }
                }
                'y' if control_down => {
                    if game_state.redo_move() {
                        highlight_moves.clear();
                    }
                }
                's' => draw_square_names = !draw_square_names,
                'p' => draw_pieces = !draw_pieces,
                'i' => invert = !invert,
                'r' => {
                    game_state = GameState::default();
                    total_time = 0;
                }
                otherwise => (),
            }
        }

        root_ui().window(
            hash!(),
            Vec2::new(FIELD_SIZE * 8.0, 0.0),
            Vec2::new(UI_WIDTH, FIELD_SIZE * 8.0),
            |ui| {
                ui.separator();
                ui.label(None, &format!("Eval: {}", eval(game_state.board())));
                if let Some(alpha) = last_alpha {
                    ui.label(None, &format!("Last alpha: {}", alpha));
                } else {
                    ui.label(None, &format!("Last alpha: None"));
                }
                if let Some(depth) = last_depth {
                    ui.label(None, &format!("Last depth: {}", depth));
                } else {
                    ui.label(None, &format!("Last depth: None"));
                }
                if let Some(millis) = last_millis {
                    ui.label(
                        None,
                        &format!("Last search: {:.3}s", millis as f64 / 1_000.0),
                    );
                } else {
                    ui.label(None, &format!("Last search: None"));
                }
                ui.separator();
                ui.checkbox(UI_ID_CHECKBOX, "Auto respond", &mut auto_respond);
                ui.checkbox(UI_ID_CHECKBOX_DSN, "Square names", &mut draw_square_names);
                ui.checkbox(UI_ID_CHECKBOX_DP, "Draw pieces", &mut draw_pieces);
                ui.label(None, &format!("Game: {:?}", game_state.board().status()));
                let mut seconds = thinking_millis as f32 / 1000.0;
                ui.slider(UI_ID_SLIDER, "Search time", 0.5..120.0, &mut seconds);
                if ui.button(None, "1s") {
                    seconds = 1.0;
                }
                if ui.button(None, "3s") {
                    seconds = 3.0;
                }
                if ui.button(None, "5s") {
                    seconds = 5.0;
                }
                if ui.button(None, "10s") {
                    seconds = 10.0;
                }
                thinking_millis = (seconds * 1000.0) as u128;
                if ui.button(None, "< undo") {
                    game_state.undo_move();
                }
                ui.same_line(50.0);
                if ui.button(None, "redo >") {
                    game_state.redo_move();
                }
            },
        );

        next_frame().await
    }

    Ok(())
}

fn draw_text_centered(text: &str, font_size: f32, color: Color) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    let dims = measure_text(text, None, font_size as u16, 1.0);
    let x = (screen_w - dims.width) / 2.0;
    let y = (screen_h + dims.height) / 2.0;

    draw_text(text, x, y, font_size, color);
}

fn invert_square(sq: Square) -> Square {
    Square::make_square(
        Rank::from_index(7 - sq.get_rank().to_index()),
        File::from_index(7 - sq.get_file().to_index()),
    )
}

fn square_to_xy(square: Square) -> (f32, f32) {
    (
        square.get_file().to_index() as f32 * FIELD_SIZE,
        (7 - square.get_rank().to_index()) as f32 * FIELD_SIZE,
    )
}

fn hovered_piece(board: &Board, invert: bool) -> Option<(Piece, ChessColor)> {
    let square = hovered_square(invert);
    board.piece_on(square).zip(board.color_on(square))
}

fn hovered_square(invert: bool) -> Square {
    let (x, y) = mouse_position();
    let sq = square_under(x, y);
    if invert { invert_square(sq) } else { sq }
}

fn square_under(x: f32, y: f32) -> Square {
    Square::make_square(
        Rank::from_index(7 - (y / FIELD_SIZE) as usize),
        File::from_index((x / FIELD_SIZE) as usize),
    )
}

fn draw_piece(piece: Piece, color: ChessColor, x: f32, y: f32, piece_sprites: &Textures) {
    let texture = &Texture2D::from_image(piece_sprites.get_piece((piece, color)));
    texture.set_filter(FilterMode::Nearest);
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(FIELD_SIZE, FIELD_SIZE)),
            ..Default::default()
        },
    );
}

fn choose_promotion() -> Piece {
    loop {
        print!("Promote to (n = Knight, b = Bishop, r = Rook, q = Queen): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "n" => return Piece::Knight,
            "b" => return Piece::Bishop,
            "r" => return Piece::Rook,
            "q" => return Piece::Queen,
            _ => println!("Invalid input!"),
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Chessian".to_owned(),
        window_width: 8 * FIELD_SIZE as i32 + UI_WIDTH as i32,
        window_height: 8 * FIELD_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}
