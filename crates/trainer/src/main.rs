use genetic_rs_extras::{pb::ProgressObserver, plot::FitnessPlotter};
use neat::*;
use plotters::{
    drawing::IntoDrawingArea as _, prelude::SVGBackend, style::{Color as _, IntoFont as _}
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use trainer::{
    Brain, GAME_DELTA_TIME, GAME_FPS, GAME_TIME_S,
    MUTATION_PASSES, MUTATION_RATE, NB_GAMES, NB_GENERATIONS, NB_GENOME_PER_GEN,
};
use std::{io::Write as _, path::PathBuf};

#[macro_use]
extern crate log;

mod utils;

const OUTPUT_DIR: &str = "./sim";

fn fitness(brain: &Brain) -> f32 {
    (0..NB_GAMES).map(|_| play_game(&brain)).sum::<f32>() / NB_GAMES as f32
}

struct BestAgentSaver {
    path: PathBuf,
}

impl FitnessObserver<Brain> for BestAgentSaver {
    fn observe(&mut self, fitnesses: &[(Brain, f32)]) {
        let best = &fitnesses[0].0;
        let serialized = ron::ser::to_string_pretty(best, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::write(&self.path, serialized).expect("failed to write best agent to file");
    }
}

fn play_game(brain: &Brain) -> f32 {
    let mut game = doodl_jump::Game::new();

    let mut saved_score = game.score();
    let mut save_timer = time::DTDelay::new(10.);

    // loop for the number of frames we want to play, should be enough frames to play 100s at 60fps
    // for _ in 0..(GAME_FPS * GAME_TIME_S) {
    while game.score() < 100_000. {
        let output = brain.predict(trainer::generate_inputs(&game));

        match output.iter().max_index().unwrap() {
            0 => (), // No action
            1 => game.player_move_left(),
            2 => game.player_move_right(),
            _ => (),
        }

        game.update(GAME_DELTA_TIME);
        save_timer.update(GAME_DELTA_TIME);

        if game.lost {
            // println!("Lost: {}", game.score());
            break;
        }

        if save_timer.ended() {
            if game.score() == saved_score {
                // The player stagnated and needs to be shot (ingame)
                game.lost = true;
                break;
            }
            saved_score = game.score();
            save_timer.restart_custom_timeline(save_timer.time_since_ended());
        }
    }

    game.score()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let config = logger::LoggerConfig::default().set_level(log::LevelFilter::Debug);

    // logger::init(config);

    let stopwatch = time::Stopwatch::start_new();

    let running = utils::set_up_ctrlc();

    debug!("Starting training server");

    let output = PathBuf::from(OUTPUT_DIR);

    let observer = BestAgentSaver { path: output.join("best.ron") }
        .layer(ProgressObserver::new_with_default_style(NB_GENERATIONS as u64))
        .layer(FitnessPlotter::new());

    let mut rng = rand::rng();

    let mut sim = GeneticSim::new(
        Vec::gen_random(&mut rng, NB_GENOME_PER_GEN),
        FitnessEliminator::builder()
            .fitness_fn(fitness)
            .observer(observer)
            .build_or_panic(),
        CrossoverRepopulator::new(
            MUTATION_RATE,
            ReproductionSettings {
                mutation_passes: MUTATION_PASSES,
                ..Default::default()
            },
        ),
    );

    sim.perform_generations(NB_GENERATIONS);

    sim.eliminator.observer.0.1.finish();

    debug!(
        "Stopping loop. The training server ran {}\nSaving data . . .\n",
        time::format(&stopwatch.read(), 3)
    );

    // TODO still build this with ctrlc
    let plot_path = output.join("fitness.svg");
    let root = SVGBackend::new(&plot_path, (640, 480))
        .into_drawing_area();

    sim.eliminator.observer.1.plot(&root)?;

    Ok(())
}
