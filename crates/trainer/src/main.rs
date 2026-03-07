use ai_player::Brain;
use clap::Parser;
use genetic_rs_extras::{pb::ProgressObserver, plot::FitnessPlotter};
use neat::*;
use plotters::{drawing::IntoDrawingArea as _, prelude::SVGBackend};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::io::Write as _;

#[macro_use]
extern crate log;

mod cli;
mod utils;

use crate::cli::TrainerCli;

struct TrainerFitnessFn<'a> {
    cfg: &'a TrainerCli,
    seed: u64,
}

impl<'a> FitnessFn<Brain> for TrainerFitnessFn<'a> {
    fn fitness(&self, brain: &Brain) -> f32 {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let mut total_score = 0.0;

        for _ in 0..self.cfg.nb_games {
            total_score += play_game(brain, self.cfg, &mut rng);
        }

        total_score / self.cfg.nb_games as f32
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

fn play_game(brain: &Brain, cfg: &TrainerCli, rng: &mut impl Rng) -> f32 {
    let mut game = doodl_jump::Game::new();
    let mut elapsed_s = 0.0;

    let mut saved_score = game.score();
    let mut save_timer = time::DTDelay::new(cfg.stagnation_timeout_s);

    while game.score() < 100_000. {
        let frame_dt = cfg.frame_delta_time(rng);
        let output = brain.predict(ai_player::generate_inputs(&game, frame_dt as f32));

        ai_player::apply_action(&mut game, &output);

        game.update(frame_dt);
        elapsed_s += frame_dt;
        save_timer.update(frame_dt);

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

    let penalty_time_s = (elapsed_s - cfg.fitness_time_grace_s).max(0.0) as f32;
    let time_penalty = penalty_time_s * cfg.fitness_time_penalty_per_s;

    game.score() - time_penalty
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

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
    let fitness_fn = TrainerFitnessFn {
        cfg: &cfg,
        seed: rng.random(),
    };

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
        sim.eliminator.fitness_fn.seed = rng.random();
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
