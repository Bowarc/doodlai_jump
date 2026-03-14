use std::path::PathBuf;

use ::rand::prelude::*;
use ai_player::{Brain, GenerationDump};
use doodl_jump::{Game, player::MoveDirection};
use egui_macroquad as egui_mq;
use egui_mq::egui;
use macroquad::prelude::*;
use rayon::prelude::*;

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
    GenerationMenu {
        selected_dump: Option<PathBuf>,
        /// Set when an rfd dialog is in flight so we don't open multiple.
        picking: bool,
        /// Loaded dump cached after selection so the menu can show info.
        dump: Option<GenerationDump>,
        /// Last error message (e.g. deserialization failure).
        error: Option<String>,
        /// Stagnation timeout for playback (seconds).
        stagnation_timeout_s: f64,
        /// If true, playback uses macroquad frame dt instead of dump dt settings.
        use_real_dt: bool,
    },
    UserPlaying {
        game: Game,
    },
    AIPlaying {
        brain: Brain,
        game: Game,
    },
    GenerationPlayback {
        dump: GenerationDump,
        game: Game,
        brains: Vec<Brain>,
        /// Player/camera index to track.
        tracked_i: usize,
        paused: bool,

        /// If true, playback uses macroquad frame dt instead of dump dt settings.
        use_real_dt: bool,

        /// Stagnation timeout (seconds).
        stagnation_timeout_s: f64,
        /// Time since any tracked score improved.
        stagnation_elapsed_s: f64,
        /// Last seen scores used for stagnation detection.
        saved_scores: Vec<f32>,

        /// Fitness-ranked colors (0 best -> green, last -> red).
        agent_colors: Vec<Color>,
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

fn load_dump(path: &PathBuf) -> Result<GenerationDump, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read dump file: {e}"))?;

    let (dump, _): (GenerationDump, _) =
        bincode_next::serde::decode_from_slice(&bytes, bincode_next::config::standard())
            .map_err(|e| format!("Failed to deserialize generation dump: {e}"))?;

    Ok(dump)
}

fn rank_to_color(rank: usize, total: usize) -> Color {
    if total <= 1 {
        return Color::new(0.2, 1.0, 0.2, 1.0);
    }
    let t = (rank as f32) / ((total - 1) as f32);
    Color::new(t, 1.0 - t, 0.0, 1.0)
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
    let gen_rect = Rect::new(center_x - btn_w / 2.0, center_y + 120.0, btn_w, btn_h);

    if draw_button(play_rect, "Play") {
        return Transition::To(AppState::UserPlaying { game: Game::new(1) });
    }

    if draw_button(ai_rect, "AI Play") {
        return Transition::To(AppState::AIMenu {
            selected_model: None,
            picking: false,
        });
    }

    if draw_button(gen_rect, "Generation Playback") {
        return Transition::To(AppState::GenerationMenu {
            selected_dump: None,
            picking: false,
            dump: None,
            error: None,
            stagnation_timeout_s: 2.0,
            use_real_dt: false,
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
                        game: Game::new(1),
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

fn update_gen_menu(
    selected_dump: &mut Option<PathBuf>,
    picking: &mut bool,
    dump: &mut Option<GenerationDump>,
    error: &mut Option<String>,
    stagnation_timeout_s: &mut f64,
    use_real_dt: &mut bool,
) -> Transition {
    clear_background(DARKGRAY);

    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    // Title
    let title = "Generation Playback - select a dump";
    let title_size = 32;
    let title_dims = measure_text(title, None, title_size, 1.0);
    draw_text(
        title,
        center_x - title_dims.width / 2.0,
        center_y - 140.0,
        title_size as f32,
        WHITE,
    );

    // Current selection display
    let label = match selected_dump {
        Some(p) => format!("Dump: {}", p.display()),
        None => "No dump selected".to_string(),
    };
    let label_dims = measure_text(&label, None, 20, 1.0);
    draw_text(
        &label,
        center_x - label_dims.width / 2.0,
        center_y - 95.0,
        20.0,
        LIGHTGRAY,
    );

    // Error display
    if let Some(e) = error.as_ref() {
        let err_dims = measure_text(e, None, 18, 1.0);
        draw_text(
            e,
            center_x - err_dims.width / 2.0,
            center_y - 65.0,
            18.0,
            RED,
        );
    }

    // Stagnation timeout display (simple + hotkeys)
    let st_label = format!(
        "Stagnation timeout: {:.2}s  (Up/Down to adjust)",
        *stagnation_timeout_s
    );
    let st_dims = measure_text(&st_label, None, 20, 1.0);
    draw_text(
        &st_label,
        center_x - st_dims.width / 2.0,
        center_y - 35.0,
        20.0,
        WHITE,
    );

    // Hotkey adjustment
    if is_key_pressed(KeyCode::Up) {
        *stagnation_timeout_s = (*stagnation_timeout_s * 1.25).min(20.0);
    }
    if is_key_pressed(KeyCode::Down) {
        *stagnation_timeout_s = (*stagnation_timeout_s / 1.25).max(0.1);
    }

    // Toggle dt mode (macroquad checkbox + keyboard shortcut)
    let dt_label = format!(
        "Use real dt: {}  (press T to toggle)",
        if *use_real_dt { "ON" } else { "OFF" }
    );
    let dt_dims = measure_text(&dt_label, None, 20, 1.0);
    draw_text(
        &dt_label,
        center_x - dt_dims.width / 2.0,
        center_y - 5.0,
        20.0,
        WHITE,
    );
    if is_key_pressed(KeyCode::T) {
        *use_real_dt = !*use_real_dt;
    }

    let btn_w = 240.0;
    let btn_h = 50.0;

    // Browse button
    let browse_rect = Rect::new(center_x - btn_w / 2.0, center_y + 10.0, btn_w, btn_h);
    if draw_button(browse_rect, "Browse...") && !*picking {
        *picking = true;
        let file = rfd::FileDialog::new()
            .add_filter("Generation dump (.bin)", &["bin"])
            .pick_file();
        if let Some(path) = file {
            *selected_dump = Some(path);
            match load_dump(selected_dump.as_ref().unwrap()) {
                Ok(d) => {
                    *dump = Some(d);
                    *error = None;
                }
                Err(e) => {
                    *dump = None;
                    *error = Some(e);
                }
            }
        }
        *picking = false;
    }

    // Start button (enabled only when a dump is loaded)
    let start_rect = Rect::new(center_x - btn_w / 2.0, center_y + 80.0, btn_w, btn_h);
    if dump.is_some() {
        if draw_button(start_rect, "Start") {
            if let Some(d) = dump.take() {
                let n = d.genomes.len().max(1);

                // Must match training seed usage:
                // trainer seeds StdRng from dump.seed, then samples game_seed.
                let mut outer_rng = StdRng::seed_from_u64(d.seed);
                let game_seed: u64 = outer_rng.next_u64();

                let game = Game::new_with_seed(n, game_seed);

                let brains = d.genomes.clone();
                let agent_colors = (0..n).map(|i| rank_to_color(i, n)).collect::<Vec<_>>();

                let mut saved_scores = vec![0.0; n];
                for i in 0..n {
                    saved_scores[i] = game.score(i);
                }

                return Transition::To(AppState::GenerationPlayback {
                    dump: d,
                    game,
                    brains,
                    tracked_i: 0,
                    paused: false,
                    use_real_dt: *use_real_dt,
                    stagnation_timeout_s: *stagnation_timeout_s,
                    stagnation_elapsed_s: 0.0,
                    saved_scores,
                    agent_colors,
                });
            }
        }
    } else {
        draw_button_disabled(start_rect, "Start");
    }

    // Back button
    let back_rect = Rect::new(center_x - btn_w / 2.0, center_y + 150.0, btn_w, btn_h);
    if draw_button(back_rect, "Back") || is_key_pressed(KeyCode::Escape) {
        *picking = false;
        return Transition::To(AppState::MainMenu);
    }

    Transition::None
}

fn update_user_playing(game: &mut Game) -> Transition {
    clear_background(BLACK);

    // Input (player 0)
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        game.player_move(0, MoveDirection::Left);
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        game.player_move(0, MoveDirection::Right);
    }

    let dt = get_frame_time() as f64;
    game.update(dt);

    draw_game(game);

    if game.lost.get(0).copied().unwrap_or(false) {
        draw_game_over(game);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return Transition::To(AppState::UserPlaying { game: Game::new(1) });
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
    let inputs = ai_player::generate_inputs(game, 0, dt);
    let output = brain.predict(inputs);
    ai_player::apply_action(game, 0, &output);

    game.update(dt as f64);

    draw_game(game);

    if game.lost.get(0).copied().unwrap_or(false) {
        draw_game_over(game);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return Transition::To(AppState::AIPlaying {
                brain: brain.clone(),
                game: Game::new(1),
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

fn update_generation_playback(
    dump: &GenerationDump,
    game: &mut Game,
    brains: &[Brain],
    tracked_i: &mut usize,
    paused: &mut bool,
    use_real_dt: &mut bool,
    stagnation_timeout_s: &mut f64,
    stagnation_elapsed_s: &mut f64,
    saved_scores: &mut [f32],
    agent_colors: &[Color],
) -> Transition {
    clear_background(BLACK);

    // Playback dt: either real dt (macroquad) or the exact settings dumped by the trainer.
    let dt = if *use_real_dt {
        get_frame_time() as f64
    } else if let Some(jitter) = dump.dt_jitter {
        game.jittered_dt(dump.base_dt, jitter)
    } else {
        dump.base_dt
    };

    // UI
    egui_mq::ui(|ctx| {
        egui::Window::new("Generation Playback")
            .default_pos((10.0, 10.0))
            .show(ctx, |ui| {
                ui.label(format!("Seed: {}", dump.seed));
                let living = game
                    .lost
                    .iter()
                    .take(brains.len())
                    .filter(|&&lost| !lost)
                    .count();
                ui.label(format!("Agents: {} (living: {})", brains.len(), living));
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Track player:");
                    ui.add(
                        egui::DragValue::new(tracked_i).range(0..=brains.len().saturating_sub(1)),
                    );
                });

                ui.horizontal(|ui| {
                    let label = if *paused { "Resume" } else { "Pause" };
                    if ui.button(label).clicked() {
                        *paused = !*paused;
                    }
                    if ui.button("Restart").clicked() {
                        let n = brains.len().max(1);
                        let mut outer_rng = StdRng::seed_from_u64(dump.seed);
                        let game_seed: u64 = outer_rng.next_u64();
                        *game = Game::new_with_seed(n, game_seed);
                        *stagnation_elapsed_s = 0.0;
                        for (i, s) in saved_scores.iter_mut().enumerate() {
                            *s = game.score(i);
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Use real dt:");
                    ui.checkbox(use_real_dt, "");
                });

                ui.separator();
                ui.label("Stagnation timeout (seconds):");
                ui.add(
                    egui::Slider::new(stagnation_timeout_s, 0.1..=20.0)
                        .logarithmic(true)
                        .text("s"),
                );
                ui.label(format!("Stagnation elapsed: {:.2}s", *stagnation_elapsed_s));

                ui.separator();

                egui::CollapsingHeader::new("Legend (fitness rank)")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(240.0)
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                            )
                            .drag_to_scroll(true)
                            .show(ui, |ui| {
                                // Use a slightly larger row height to make wheel scrolling feel faster
                                // (fewer "rows per wheel tick" perceived).
                                ui.spacing_mut().item_spacing.y = 6.0;

                                ui.label("Click an entry to spectate (track) that agent.");

                                for i in 0..brains.len() {
                                    let c = agent_colors.get(i).copied().unwrap_or(WHITE);
                                    let lost = game.lost.get(i).copied().unwrap_or(false);

                                    let (r, g, b, a) = if lost {
                                        (c.r, c.g, c.b, 0.45)
                                    } else {
                                        (c.r, c.g, c.b, 1.0)
                                    };

                                    let color32 = egui::Color32::from_rgba_unmultiplied(
                                        (r * 255.0) as u8,
                                        (g * 255.0) as u8,
                                        (b * 255.0) as u8,
                                        (a * 255.0) as u8,
                                    );

                                    // Make the whole row clickable to switch spectated agent.
                                    let row_label = format!(
                                        "#{:03}  score: {:>7.0}{}",
                                        i,
                                        game.score(i),
                                        if lost { " (lost)" } else { "" }
                                    );

                                    ui.horizontal(|ui| {
                                        ui.colored_label(color32, "■");
                                        if ui
                                            .add(egui::SelectableLabel::new(
                                                *tracked_i == i,
                                                row_label,
                                            ))
                                            .clicked()
                                        {
                                            *tracked_i = i;
                                        }
                                    });
                                }
                            });
                    });

                ui.separator();
                ui.label("Esc: back to main menu");
            });
    });
    egui_mq::draw();

    if is_key_pressed(KeyCode::Escape) {
        return Transition::To(AppState::MainMenu);
    }

    if !*paused {
        let outputs: Vec<[f32; ai_player::AGENT_OUT]> = brains
            .par_iter()
            .enumerate()
            .map(|(i, brain)| brain.predict(ai_player::generate_inputs(game, i, dt as f32)))
            .collect();

        for (i, output) in outputs.iter().enumerate() {
            ai_player::apply_action(game, i, output);
        }

        game.update(dt);

        // Stagnation timeout: mirror trainer behaviour (kill if no progress for N seconds).
        let mut any_improved = false;
        for i in 0..brains.len() {
            let s = game.score(i);
            if s != saved_scores[i] {
                saved_scores[i] = s;
                any_improved = true;
            }
        }

        if any_improved {
            *stagnation_elapsed_s = 0.0;
        } else {
            *stagnation_elapsed_s += dt;
            if *stagnation_elapsed_s >= *stagnation_timeout_s {
                for i in 0..brains.len() {
                    if let Some(l) = game.lost.get_mut(i) {
                        *l = true;
                    }
                }
                *paused = true;
            }
        }
    }

    // Render world using tracked player's camera.
    draw_generation_game(game, *tracked_i, agent_colors);

    Transition::None
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

/// Map game coordinates to screen coordinates.
/// Game uses y-up with per-player scroll; screen uses y-down from top-left.
fn game_to_screen(game: &Game, player_index: usize, gx: f64, gy: f64) -> (f32, f32) {
    let scale_x = screen_width() / doodl_jump::GAME_WIDTH as f32;
    let scale_y = screen_height() / doodl_jump::GAME_HEIGHT as f32;
    let scale = scale_x.min(scale_y);

    let offset_x = (screen_width() - doodl_jump::GAME_WIDTH as f32 * scale) / 2.0;
    let offset_y = (screen_height() - doodl_jump::GAME_HEIGHT as f32 * scale) / 2.0;

    let sx = gx as f32 * scale + offset_x;
    let scroll = game.scrolls.get(player_index).copied().unwrap_or(0) as f64;
    let sy = (gy - scroll) as f32 * scale + offset_y;

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
        let (sx, sy) = game_to_screen(game, 0, tl.x, tl.y);
        let w = platform.rect.width() as f32 * scale;
        let h = platform.rect.height() as f32 * scale;
        draw_rectangle(sx, sy, w, h, GREEN);
    }

    // Player (player 0)
    let player0 = &game.players[0];
    let player_tl = player0.rect.aa_topleft();
    let (px, py) = game_to_screen(game, 0, player_tl.x, player_tl.y);
    let pw = player0.rect.width() as f32 * scale;
    let ph = player0.rect.height() as f32 * scale;
    draw_rectangle(px, py, pw, ph, YELLOW);

    // Score (player 0)
    draw_text(
        &format!("Score: {:.0}", game.score(0)),
        10.0,
        30.0,
        28.0,
        WHITE,
    );
}

fn draw_generation_game(game: &Game, tracked_i: usize, agent_colors: &[Color]) {
    let scale = game_scale();

    // Platforms
    for platform in &game.platforms {
        let tl = platform.rect.aa_topleft();
        let (sx, sy) = game_to_screen(game, tracked_i, tl.x, tl.y);
        let w = platform.rect.width() as f32 * scale;
        let h = platform.rect.height() as f32 * scale;
        draw_rectangle(sx, sy, w, h, GREEN);
    }

    // Agents
    for (i, player) in game.players.iter().enumerate() {
        let tl = player.rect.aa_topleft();
        let (px, py) = game_to_screen(game, tracked_i, tl.x, tl.y);
        let pw = player.rect.width() as f32 * scale;
        let ph = player.rect.height() as f32 * scale;

        let base = agent_colors.get(i).copied().unwrap_or(WHITE);
        let lost = game.lost.get(i).copied().unwrap_or(false);
        let c = if lost {
            Color::new(base.r, base.g, base.b, 0.45)
        } else {
            base
        };

        draw_rectangle(px, py, pw, ph, c);
    }

    // HUD
    let lost = game.lost.get(tracked_i).copied().unwrap_or(false);
    draw_text(
        &format!(
            "Tracking: #{tracked_i:03}  Score: {:.0}{}",
            game.score(tracked_i),
            if lost { " (lost)" } else { "" }
        ),
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

    let score_text = format!("Score: {:.0}", game.score(0));
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

            AppState::GenerationMenu {
                selected_dump,
                picking,
                dump,
                error,
                stagnation_timeout_s,
                use_real_dt,
            } => update_gen_menu(
                selected_dump,
                picking,
                dump,
                error,
                stagnation_timeout_s,
                use_real_dt,
            ),

            AppState::UserPlaying { game } => update_user_playing(game),

            AppState::AIPlaying { brain, game } => {
                let brain = &*brain;
                update_ai_playing(brain, game)
            }

            AppState::GenerationPlayback {
                dump,
                game,
                brains,
                tracked_i,
                paused,
                use_real_dt,
                stagnation_timeout_s,
                stagnation_elapsed_s,
                saved_scores,
                agent_colors,
            } => update_generation_playback(
                dump,
                game,
                brains,
                tracked_i,
                paused,
                use_real_dt,
                stagnation_timeout_s,
                stagnation_elapsed_s,
                saved_scores,
                agent_colors,
            ),
        };

        if let Transition::To(new_state) = transition {
            state = new_state;
        }

        next_frame().await;
    }
}
