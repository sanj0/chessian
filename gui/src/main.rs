mod graphics;

use std::io::{self, Write, stdout};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;

use chess::Color as ChessColor;
use chess::*;
use macroquad::color::Color;
use macroquad::input::KeyCode;
use macroquad::prelude::*;
use macroquad::ui::*;

use gamestate::GameState;
use graphics::Textures;

use chessian::chooser::*;
use chessian::*;

pub const FIELD_SIZE: f32 = 100.0;
pub const COLOR_WHITE: Color = Color::from_hex(0xFFFFF2);
pub const COLOR_BLACK: Color = Color::from_hex(0xFFC0CB);
pub const COLOR_BLUE: Color = Color::from_hex(0xB3EBF2);
pub const COLOR_RED: Color = Color::from_hex(0xFF746C);
pub const MOVE_INDICATOR_SIZE: f32 = 15.0;
pub const COLOR_MOVES: Color = Color::new(1., 0.1, 0.1, 0.5);

pub const EVAL_BAR_W: f32 = 35.0;

pub const UI_WIDTH: f32 = 200.0;
pub const UI_ID_CHECKBOX: Id = 0;
pub const UI_ID_CHECKBOX_DSN: Id = 2;
pub const UI_ID_CHECKBOX_DP: Id = 3;
pub const UI_ID_SLIDER: Id = 4;
pub const UI_ID_UNDO_REDO_GROUP: Id = 1;
pub const UI_ID_EVAL: Id = 666;

#[macroquad::main(conf)]
async fn main() -> Result<(), String> {
    //testsuite::eigenmann();
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
                let Some(result) = chessian::chooser::best_move(
                    game_state.board(),
                    TimeControl::new(None, TCMode::MoveTime(millis)),
                    &[],
                    stdout(),
                    stdout(),
                ) else {
                    return Err(String::from("error"));
                };
                println!(
                    "{}",
                    board_to_fen(&game_state.board().make_move_new(result.best_move))
                );
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
    let mut message = "-";
    let mut pending_promotion_move: Option<ChessMove> = None;
    let mut eval = true;
    let mut eval_move: Option<ChessMove> = None;
    let mut eval_depth = 1;
    let mut eval_stop_flag = Arc::new(AtomicBool::new(false));
    let mut eval_handle = spawn_eval_thread(
        game_state.board().clone(),
        eval_depth,
        eval_stop_flag.clone(),
    );

    loop {
        root_ui().window(
            hash!(),
            Vec2::new(FIELD_SIZE * 8.0 + EVAL_BAR_W, 0.0),
            Vec2::new(UI_WIDTH, FIELD_SIZE * 8.0),
            |ui| {
                ui.separator();
                if let Some(alpha) = last_alpha {
                    ui.label(None, &format!("Eval: {}", alpha));
                } else {
                    ui.label(None, &"Eval: None".to_string());
                }
                if eval {
                    ui.label(None, &format!("Eval depth: {}", eval_depth));
                } else {
                    ui.label(None, "No eval");
                }
                let prev_eval = eval;
                ui.checkbox(UI_ID_EVAL, "Eval", &mut eval);
                if !eval {
                    eval_stop_flag.store(true, Ordering::Relaxed);
                } else if !prev_eval {
                    eval_depth = 1;
                    spawn_new_eval_thread(
                        game_state.board().clone(),
                        &mut eval_stop_flag,
                        eval_depth,
                        &mut eval_handle,
                    );
                }
                ui.label(None, &format!("Your move: {message}"));
                if let Some(depth) = last_depth {
                    ui.label(None, &format!("Last depth: {}", depth));
                } else {
                    ui.label(None, &"Last depth: None".to_string());
                }
                if let Some(millis) = last_millis {
                    ui.label(
                        None,
                        &format!("Last search: {:.3}s", millis as f64 / 1_000.0),
                    );
                } else {
                    ui.label(None, &"Last search: None".to_string());
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
                if ui.button(None, "GO, GO, GO!") {
                    engine_move_next_frame = true;
                }
                ui.label(
                    None,
                    &format!(
                        "Don't play: {}",
                        game_state
                            .excluded_moves()
                            .iter()
                            .map(|m| format!("{},", m))
                            .collect::<String>()
                    ),
                );
                if ui.button(None, "< undo") {
                    game_state.undo_move();
                    if eval {
                        eval_depth = 1;
                        spawn_new_eval_thread(
                            game_state.board().clone(),
                            &mut eval_stop_flag,
                            eval_depth,
                            &mut eval_handle,
                        );
                    }
                }
                ui.same_line(50.0);
                if ui.button(None, "redo >") {
                    game_state.redo_move();
                    if eval {
                        eval_depth = 1;
                        spawn_new_eval_thread(
                            game_state.board().clone(),
                            &mut eval_stop_flag,
                            eval_depth,
                            &mut eval_handle,
                        );
                    }
                }
            },
        );

        if let Ok(Some(result)) = eval_handle.try_recv() {
            last_alpha = Some(if game_state.board().side_to_move() == ChessColor::Black {
                -result.deep_eval
            } else {
                result.deep_eval
            });
            eval_move = Some(result.best_move);
            if eval {
                eval_depth += 1;
                spawn_new_eval_thread(
                    game_state.board().clone(),
                    &mut eval_stop_flag,
                    eval_depth,
                    &mut eval_handle,
                );
            }
        }

        if let Some(score) = last_alpha {
            let pawn_score = score as f32 / 100.0;
            let bar_y = FIELD_SIZE * 4.0 + pawn_score * 25.0;
            draw_rectangle(FIELD_SIZE * 8.0, bar_y, EVAL_BAR_W, FIELD_SIZE * 8.0, BLACK);
            draw_rectangle(FIELD_SIZE * 8.0, 0.0, EVAL_BAR_W, bar_y, COLOR_WHITE);
            draw_text(
                &format!("{pawn_score:.1}"),
                FIELD_SIZE * 8.0,
                FIELD_SIZE * 4.0,
                15.0,
                COLOR_RED,
            );
        } else {
            draw_rectangle(FIELD_SIZE * 8.0, 0.0, EVAL_BAR_W, FIELD_SIZE * 8.0, GRAY);
        }

        let hovered_square = hovered_square(invert);
        let mouse_in_board = mouse_position().0 <= FIELD_SIZE * 8.0;

        for y in 0..=7 {
            for x in 0..=7 {
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
                if square == hovered_square && mouse_in_board {
                    draw_rectangle_lines(x_pos, y_pos, FIELD_SIZE, FIELD_SIZE, 7.5, COLOR_BLUE);
                }
                // Draw piece?
                if draw_pieces
                    && let Some((piece, color)) = game_state
                        .board()
                        .piece_on(square)
                        .zip(game_state.board().color_on(square))
                {
                    draw_piece(piece, color, x_pos, y_pos, &piece_sprites);
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

                if let Some(m) = game_state.last_move()
                    && (m.get_source() == square || m.get_dest() == square)
                {
                    draw_rectangle_lines(x_pos, y_pos, FIELD_SIZE, FIELD_SIZE, 7.5, COLOR_RED);
                }
            }
        }

        if let Some(r) = eval_move
            && eval
        {
            let (x0, y0) = square_to_xy(if invert {
                invert_square(r.get_source())
            } else {
                r.get_source()
            });
            let (x1, y1) = square_to_xy(if invert {
                invert_square(r.get_dest())
            } else {
                r.get_dest()
            });
            draw_line(
                x0 + FIELD_SIZE / 2.0,
                y0 + FIELD_SIZE / 2.0,
                x1 + FIELD_SIZE / 2.0,
                y1 + FIELD_SIZE / 2.0,
                5.0,
                COLOR_RED,
            );
        }

        if let Some(pending_promotion) = pending_promotion_move {
            let dest = pending_promotion.get_dest();
            let to_inner_board = if dest.get_rank() == Rank::First {
                Square::up
            } else {
                Square::down
            };
            let queen_sq = to_inner_board(&dest).unwrap();
            let rook_sq = to_inner_board(&queen_sq).unwrap();
            let bishop_sq = to_inner_board(&rook_sq).unwrap();
            let knight_sq = to_inner_board(&bishop_sq).unwrap();
            for (square, piece) in [queen_sq, rook_sq, bishop_sq, knight_sq]
                .iter()
                .zip([Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight].iter())
            {
                let (x, y) = square_to_xy(if invert {
                    invert_square(*square)
                } else {
                    *square
                });
                draw_piece(
                    *piece,
                    game_state.board().side_to_move(),
                    x,
                    y,
                    &piece_sprites,
                );
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                let clicked_promotion = if hovered_square == queen_sq {
                    Some(Piece::Queen)
                } else if hovered_square == rook_sq {
                    Some(Piece::Rook)
                } else if hovered_square == bishop_sq {
                    Some(Piece::Bishop)
                } else if hovered_square == knight_sq {
                    Some(Piece::Knight)
                } else {
                    None
                };
                if let Some(promotion) = clicked_promotion {
                    game_state.make_move(ChessMove::new(
                        pending_promotion.get_source(),
                        dest,
                        Some(promotion),
                    ));
                    if eval {
                        eval_depth = 1;
                        spawn_new_eval_thread(
                            game_state.board().clone(),
                            &mut eval_stop_flag,
                            eval_depth,
                            &mut eval_handle,
                        );
                    }
                    game_state.excluded_moves().clear();
                }
                pending_promotion_move = None;
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
            if let Some(result) =
                game_state.engine_move(TimeControl::new(None, TCMode::MoveTime(thinking_millis)))
            {
                if let Some(last_alpha) = last_alpha {
                    let diff = result.deep_eval - last_alpha;
                    if diff > 500 {
                        message = "BLUNDER!";
                    } else if diff > 200 {
                        message = "Blunder!";
                    } else if diff > 100 {
                        message = "Mistake!";
                    } else if diff > 0 {
                        message = "Inaccuracy!";
                    } else if diff > -50 {
                        message = "Good!";
                    } else {
                        message = "Perfect!";
                    }
                }
                last_alpha = Some(result.deep_eval);
                last_depth = Some(result.reached_depth);
                last_millis = Some(result.millis);
                total_time += result.millis;
                println!("{:.2}s total time thinking", total_time as f64 / 1000.0);
            }
            engine_move_next_frame = false;
            if eval {
                eval_depth = 1;
                spawn_new_eval_thread(
                    game_state.board().clone(),
                    &mut eval_stop_flag,
                    eval_depth,
                    &mut eval_handle,
                );
            }
            highlight_moves.clear();
            continue;
        }

        if !mouse_in_board {
            next_frame().await;
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
        if is_mouse_button_pressed(MouseButton::Left) {
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
                    let mov = *m;
                    if mov.get_promotion().is_some() {
                        pending_promotion_move = Some(mov);
                    } else {
                        game_state.make_move(mov);
                        if eval {
                            eval_depth = 1;
                            spawn_new_eval_thread(
                                game_state.board().clone(),
                                &mut eval_stop_flag,
                                eval_depth,
                                &mut eval_handle,
                            );
                        }
                        game_state.excluded_moves().clear();
                        engine_move_next_frame = auto_respond;
                    }
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
                    game_state.excluded_moves().clear();
                    highlight_moves.clear();
                }
                'd' => {
                    if let Some(m) = game_state.last_engine_move() {
                        game_state.excluded_moves().push(m);
                        game_state.undo_move();
                        engine_move_next_frame = true;
                    }
                }
                'z' if control_down => {
                    if game_state.undo_move() {
                        highlight_moves.clear();
                        if eval {
                            eval_depth = 1;
                            spawn_new_eval_thread(
                                game_state.board().clone(),
                                &mut eval_stop_flag,
                                eval_depth,
                                &mut eval_handle,
                            );
                        }
                    }
                }
                'y' if control_down => {
                    if game_state.redo_move() {
                        highlight_moves.clear();
                        if eval {
                            eval_depth = 1;
                            spawn_new_eval_thread(
                                game_state.board().clone(),
                                &mut eval_stop_flag,
                                eval_depth,
                                &mut eval_handle,
                            );
                        }
                    }
                }
                's' => draw_square_names = !draw_square_names,
                'p' => draw_pieces = !draw_pieces,
                'i' => invert = !invert,
                'r' => {
                    game_state = GameState::default();
                    total_time = 0;
                }
                't' => {
                    let history = game_state.history();
                    println!("Analyzing game. Will take {} seconds", history.len() * 3);
                    for (b, m) in history {
                        let result = chessian::chooser::best_move(
                            b,
                            TimeControl::new(None, TCMode::MoveTime(3000)),
                            &[],
                            std::io::sink(),
                            std::io::sink(),
                        )
                        .unwrap();
                        print!("{}", result.deep_eval);
                        let _ = std::io::stdout().flush();
                    }
                }
                otherwise => (),
            }
        }

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

fn spawn_new_eval_thread(
    board: HistoryBoard,
    stop_flag: &mut Arc<AtomicBool>,
    eval_depth: usize,
    rec: &mut mpsc::Receiver<Option<ChooserResult>>,
) {
    stop_flag.store(true, Ordering::Relaxed);
    // wait for old eval thread to stop
    rec.recv();
    *stop_flag = Arc::new(AtomicBool::new(false));
    *rec = spawn_eval_thread(board, eval_depth, stop_flag.clone());
}

fn spawn_eval_thread(
    board: HistoryBoard,
    depth: usize,
    stop_flag: Arc<AtomicBool>,
) -> mpsc::Receiver<Option<ChooserResult>> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let eval = best_move(
            &board,
            TimeControl::new(Some(stop_flag), TCMode::Depth(depth)),
            &[],
            std::io::sink(),
            std::io::sink(),
        );
        tx.send(eval)
    });

    rx
}

fn conf() -> Conf {
    Conf {
        window_title: "Chessian".to_owned(),
        window_width: 8 * FIELD_SIZE as i32 + EVAL_BAR_W as i32 + UI_WIDTH as i32,
        window_height: 8 * FIELD_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}
