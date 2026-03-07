use std::path::PathBuf;

use ai_player::Brain;
use doodl_jump::{Game, player::MoveDirection};
use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

enum AppState {
    MainMenu,
    AIMenu {
        selected_model: Option<PathBuf>,
        /// Set when an rfd dialog is in flight so we don't open multiple.
        picking: bool,
    },
    UserPlaying {
        game: Game,
    },
    AIPlaying {
        brain: Brain,
        game: Game,
    },
}

impl Default for AppState {
    fn default() -> Self {
        Self::MainMenu
    }
}

// ---------------------------------------------------------------------------
// Transition type returned by each screen's update function.
// `None` means "stay in this state".
// ---------------------------------------------------------------------------

enum Transition {
    None,
    To(AppState),
}

// ---------------------------------------------------------------------------
// Model loading
// ---------------------------------------------------------------------------

fn load_brain(path: &PathBuf) -> Result<Brain, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read model file: {e}"))?;

    let (brain, _): (Brain, _) =
        bincode_next::serde::decode_from_slice(&bytes, bincode_next::config::standard())
            .map_err(|e| format!("Failed to deserialize model: {e}"))?;

    Ok(brain)
}

// ---------------------------------------------------------------------------
// Per-state update + draw
// ---------------------------------------------------------------------------

fn update_main_menu() -> Transition {
    clear_background(DARKGRAY);

    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    // Title
    let title = "Doodlai Jump";
    let title_size = 48;
    let title_dims = measure_text(title, None, title_size, 1.0);
    draw_text(
        title,
        center_x - title_dims.width / 2.0,
        center_y - 80.0,
        title_size as f32,
        WHITE,
    );

    // --- Play button ---
    let btn_w = 200.0;
    let btn_h = 50.0;
    let play_rect = Rect::new(center_x - btn_w / 2.0, center_y - 10.0, btn_w, btn_h);
    let ai_rect = Rect::new(center_x - btn_w / 2.0, center_y + 60.0, btn_w, btn_h);

    if draw_button(play_rect, "Play") {
        return Transition::To(AppState::UserPlaying { game: Game::new() });
    }

    if draw_button(ai_rect, "AI Play") {
        return Transition::To(AppState::AIMenu {
            selected_model: None,
            picking: false,
        });
    }

    Transition::None
}

fn update_ai_menu(selected_model: &mut Option<PathBuf>, picking: &mut bool) -> Transition {
    clear_background(DARKGRAY);

    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    // Title
    let title = "AI Play - select a model";
    let title_size = 32;
    let title_dims = measure_text(title, None, title_size, 1.0);
    draw_text(
        title,
        center_x - title_dims.width / 2.0,
        center_y - 100.0,
        title_size as f32,
        WHITE,
    );

    // Current selection display
    let label = match selected_model {
        Some(p) => format!("Model: {}", p.display()),
        None => "No model selected".to_string(),
    };
    let label_dims = measure_text(&label, None, 20, 1.0);
    draw_text(
        &label,
        center_x - label_dims.width / 2.0,
        center_y - 50.0,
        20.0,
        LIGHTGRAY,
    );

    let btn_w = 200.0;
    let btn_h = 50.0;

    // Browse button
    let browse_rect = Rect::new(center_x - btn_w / 2.0, center_y, btn_w, btn_h);
    if draw_button(browse_rect, "Browse...") && !*picking {
        *picking = true;
        // rfd's sync dialog is fine here; macroquad runs on the main thread.
        let file = rfd::FileDialog::new()
            .add_filter("Neural Network (.nn)", &["nn"])
            .pick_file();
        if let Some(path) = file {
            *selected_model = Some(path);
        }
        *picking = false;
    }

    // Start button (only active when a model is selected)
    let start_rect = Rect::new(center_x - btn_w / 2.0, center_y + 70.0, btn_w, btn_h);
    if let Some(path) = selected_model {
        if draw_button(start_rect, "Start") {
            match load_brain(path) {
                Ok(brain) => {
                    return Transition::To(AppState::AIPlaying {
                        brain,
                        game: Game::new(),
                    });
                }
                Err(e) => {
                    eprintln!("{e}");
                    // Stay on this screen so the user can pick a different file.
                }
            }
        }
    } else {
        draw_button_disabled(start_rect, "Start");
    }

    // Back button
    let back_rect = Rect::new(center_x - btn_w / 2.0, center_y + 140.0, btn_w, btn_h);
    if draw_button(back_rect, "Back") {
        return Transition::To(AppState::MainMenu);
    }

    Transition::None
}

fn update_user_playing(game: &mut Game) -> Transition {
    clear_background(BLACK);

    // Input
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        game.player_move(MoveDirection::Left);
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        game.player_move(MoveDirection::Right);
    }

    let dt = get_frame_time() as f64;
    game.update(dt);

    draw_game(game);

    if game.lost {
        draw_game_over(game);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return Transition::To(AppState::UserPlaying { game: Game::new() });
        }
        if is_key_pressed(KeyCode::Escape) {
            return Transition::To(AppState::MainMenu);
        }
    } else if is_key_pressed(KeyCode::Escape) {
        return Transition::To(AppState::MainMenu);
    }

    Transition::None
}

fn update_ai_playing(brain: &Brain, game: &mut Game) -> Transition {
    clear_background(BLACK);

    let dt = get_frame_time();
    let inputs = ai_player::generate_inputs(game, dt);
    let output = brain.predict(inputs);
    ai_player::apply_action(game, &output);

    game.update(dt as f64);

    draw_game(game);

    if game.lost {
        draw_game_over(game);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return Transition::To(AppState::AIPlaying {
                brain: brain.clone(),
                game: Game::new(),
            });
        }
        if is_key_pressed(KeyCode::Escape) {
            return Transition::To(AppState::MainMenu);
        }
    } else if is_key_pressed(KeyCode::Escape) {
        return Transition::To(AppState::MainMenu);
    }

    Transition::None
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

/// Map game coordinates to screen coordinates.
/// Game uses y-up with scroll; screen uses y-down from top-left.
fn game_to_screen(game: &Game, gx: f64, gy: f64) -> (f32, f32) {
    let scale_x = screen_width() / doodl_jump::GAME_WIDTH as f32;
    let scale_y = screen_height() / doodl_jump::GAME_HEIGHT as f32;
    let scale = scale_x.min(scale_y);

    let offset_x = (screen_width() - doodl_jump::GAME_WIDTH as f32 * scale) / 2.0;
    let offset_y = (screen_height() - doodl_jump::GAME_HEIGHT as f32 * scale) / 2.0;

    let sx = gx as f32 * scale + offset_x;
    // Shift by scroll so the camera follows the player.
    let sy = (gy - game.scroll as f64) as f32 * scale + offset_y;

    (sx, sy)
}

fn game_scale() -> f32 {
    let scale_x = screen_width() / doodl_jump::GAME_WIDTH as f32;
    let scale_y = screen_height() / doodl_jump::GAME_HEIGHT as f32;
    scale_x.min(scale_y)
}

fn draw_game(game: &Game) {
    let scale = game_scale();

    // Platforms
    for platform in &game.platforms {
        let tl = platform.rect.aa_topleft();
        let (sx, sy) = game_to_screen(game, tl.x, tl.y);
        let w = platform.rect.width() as f32 * scale;
        let h = platform.rect.height() as f32 * scale;
        draw_rectangle(sx, sy, w, h, GREEN);
    }

    // Player
    let player_tl = game.player.rect.aa_topleft();
    let (px, py) = game_to_screen(game, player_tl.x, player_tl.y);
    let pw = game.player.rect.width() as f32 * scale;
    let ph = game.player.rect.height() as f32 * scale;
    draw_rectangle(px, py, pw, ph, YELLOW);

    // Score
    draw_text(
        &format!("Score: {:.0}", game.score()),
        10.0,
        30.0,
        28.0,
        WHITE,
    );
}

fn draw_game_over(game: &Game) {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    let text = "Game Over";
    let size = 48;
    let dims = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        center_x - dims.width / 2.0,
        center_y - 20.0,
        size as f32,
        RED,
    );

    let score_text = format!("Score: {:.0}", game.score());
    let score_dims = measure_text(&score_text, None, 28, 1.0);
    draw_text(
        &score_text,
        center_x - score_dims.width / 2.0,
        center_y + 20.0,
        28.0,
        WHITE,
    );

    let hint = "Enter/Space to retry - Escape for menu";
    let hint_dims = measure_text(hint, None, 20, 1.0);
    draw_text(
        hint,
        center_x - hint_dims.width / 2.0,
        center_y + 55.0,
        20.0,
        LIGHTGRAY,
    );
}

// ---------------------------------------------------------------------------
// Simple immediate-mode button helpers
// ---------------------------------------------------------------------------

fn draw_button(rect: Rect, label: &str) -> bool {
    let mouse = mouse_position();
    let hovered = rect.contains(Vec2::new(mouse.0, mouse.1));
    let color = if hovered { DARKBLUE } else { BLUE };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);

    let dims = measure_text(label, None, 24, 1.0);
    draw_text(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + (rect.h + dims.height) / 2.0,
        24.0,
        WHITE,
    );

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_button_disabled(rect: Rect, label: &str) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, DARKGRAY);

    let dims = measure_text(label, None, 24, 1.0);
    draw_text(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + (rect.h + dims.height) / 2.0,
        24.0,
        GRAY,
    );
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[macroquad::main("Doodlai Jump")]
async fn main() {
    let mut state = AppState::default();

    loop {
        let transition = match &mut state {
            AppState::MainMenu => update_main_menu(),

            AppState::AIMenu {
                selected_model,
                picking,
            } => update_ai_menu(selected_model, picking),

            AppState::UserPlaying { game } => update_user_playing(game),

            AppState::AIPlaying { brain, game } => {
                // Borrow brain immutably and game mutably by reborrowing.
                let brain = &*brain;
                update_ai_playing(brain, game)
            }
        };

        if let Transition::To(new_state) = transition {
            state = new_state;
        }

        next_frame().await;
    }
}
