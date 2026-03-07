use std::path::PathBuf;

use ai_player::Brain;
use doodl_jump::Game;
use macroquad::prelude::*;

#[derive(Default)]
enum AppState {
    #[default]
    MainMenu,
    AIMenu {
        selected_model: PathBuf,
    },
    UserPlaying {
        game: Game,
        alive: bool,
    },
    AIPlaying {
        model: Brain,
        game: Game,
        alive: bool,
    },
}

#[macroquad::main("Doodlai Jump")]
async fn main() {
    println!("Hello, world!");
}
