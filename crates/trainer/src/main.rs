use clap::Parser;
use genetic_rs_extras::{pb::ProgressObserver, plot::FitnessPlotter};
use neat::*;
use plotters::{drawing::IntoDrawingArea as _, prelude::SVGBackend};
use std::io::Write as _;
use trainer::Brain;

#[macro_use]
extern crate log;

mod cli;
mod utils;

use crate::cli::TrainerCli;

struct TrainerFitnessFn<'a> {
    cfg: &'a TrainerCli,
}

impl<'a> FitnessFn<Brain> for TrainerFitnessFn<'a> {
    fn fitness(&self, brain: &Brain) -> f32 {
        (0..self.cfg.nb_games)
            .map(|_| play_game(brain, self.cfg))
            .sum::<f32>()
            / self.cfg.nb_games as f32
    }
}

struct BestAgentSaver {
    path: std::path::PathBuf,
}

impl FitnessObserver<Brain> for BestAgentSaver {
    fn observe(&mut self, fitnesses: &[(Brain, f32)]) {
        let best = &fitnesses[0].0;
        let mut file = std::fs::File::create(&self.path).expect("Failed to create best agent file");
        let bytes = bincode_next::serde::encode_to_vec(&best, bincode_next::config::standard())
            .expect("Failed to serialize agent");
        file.write_all(&bytes)
            .expect("Failed to write best agent to file");
    }
}

fn play_game(brain: &Brain, cfg: &TrainerCli) -> f32 {
    let mut game = doodl_jump::Game::new();

    let mut saved_score = game.score();
    let mut save_timer = time::DTDelay::new(cfg.stagnation_timeout_s);

    while game.score() < 100_000. {
        let output = brain.predict(trainer::generate_inputs(&game));

        match output.iter().max_index().unwrap() {
            0 => (), // No action
            1 => game.player_move_left(),
            2 => game.player_move_right(),
            _ => (),
        }

        game.update(cfg.game_delta_time());
        save_timer.update(cfg.game_delta_time());

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

    let cfg = TrainerCli::parse();
    cfg.validate().map_err(std::io::Error::other)?;

    let stopwatch = time::Stopwatch::start_new();

    let running = utils::set_up_ctrlc();

    debug!("Starting training server");

    std::fs::create_dir_all(&cfg.output_dir)?;
    let output = cfg.output_dir.clone();

    let observer = BestAgentSaver {
        path: output.join("best.nn"),
    }
    .layer(ProgressObserver::new_with_default_style(
        cfg.nb_generations as u64,
    ))
    .layer(FitnessPlotter::new());

    let mut rng = rand::rng();
    let fitness_fn = TrainerFitnessFn { cfg: &cfg };

    let mut sim = GeneticSim::new(
        Vec::gen_random(&mut rng, cfg.nb_genome_per_gen),
        FitnessEliminator::builder()
            .fitness_fn(fitness_fn)
            .observer(observer)
            .build_or_panic(),
        CrossoverRepopulator::new(
            cfg.mutation_rate,
            ReproductionSettings {
                mutation_passes: cfg.mutation_passes,
                ..Default::default()
            },
        ),
    );

    for _ in 0..cfg.nb_generations {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        sim.next_generation();
    }

    if running.load(std::sync::atomic::Ordering::SeqCst) {
        sim.eliminator.observer.0 .1.finish();
        debug!("Finished all generations");
    } else {
        sim.eliminator.observer.0 .1.finish_and_clear();
        debug!("Received stop signal, stopping training...");
    }

    debug!(
        "Stopping loop. The training server ran {}\nSaving data . . .\n",
        time::format(&stopwatch.read(), 3)
    );

    let plot_path = output.join("fitness.svg");
    let root = SVGBackend::new(&plot_path, (640, 480)).into_drawing_area();

    sim.eliminator.observer.1.plot(&root)?;

    Ok(())
}
