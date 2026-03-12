use ai_player::Brain;
use clap::Parser;
use genetic_rs_extras::{pb::ProgressObserver, plot::FitnessPlotter};
use neat::*;
use plotters::{drawing::IntoDrawingArea as _, prelude::SVGBackend};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{
    io::Write as _,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[macro_use]
extern crate log;

mod cli;
mod utils;

use crate::cli::TrainerCli;

struct TrainerFitnessFn<'a> {
    cfg: &'a TrainerCli,
    seed: Arc<AtomicU64>,
}

impl<'a> FitnessFn<Brain> for TrainerFitnessFn<'a> {
    fn fitness(&self, brain: &Brain) -> f32 {
        let mut rng = StdRng::seed_from_u64(self.seed.load(Ordering::SeqCst));
        let mut total_score = 0.0;

        for _ in 0..self.cfg.nb_games {
            total_score += play_game(brain, self.cfg, &mut rng);
        }

        total_score / self.cfg.nb_games as f32
    }
}

struct BestAgentSaver {
    path: PathBuf,
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

#[derive(Serialize, Deserialize)]
struct GenerationDump {
    seed: u64,
    genomes: Vec<Brain>,
}

impl GenerationDump {
    pub fn from_observed(seed: u64, fitnesses: &[(Brain, f32)]) -> Self {
        Self {
            seed,
            genomes: fitnesses.iter().map(|(brain, _)| brain.clone()).collect(),
        }
    }
}

struct GenerationDumper {
    output: PathBuf,
    generation: usize,
    seed: Arc<AtomicU64>,
}

impl FitnessObserver<Brain> for GenerationDumper {
    fn observe(&mut self, fitnesses: &[(Brain, f32)]) {
        let dump = GenerationDump::from_observed(self.seed.load(Ordering::SeqCst), fitnesses);
        let bytes = bincode_next::serde::encode_to_vec(&dump, bincode_next::config::standard())
            .expect("Failed to serialize generation dump");
        let mut file = std::fs::File::create(
            self.output
                .join(format!("gen_{:04}.bin", self.generation)),
        )
        .expect("Failed to create generation dump file");
        file.write_all(&bytes)
            .expect("Failed to write generation dump to file");
        self.generation += 1;
    }
}

enum TrainingOutputObserver {
    BestAgent(BestAgentSaver),
    FullGenerations(GenerationDumper),
}

impl TrainingOutputObserver {
    fn from_config(cfg: &TrainerCli, output_dir: &Path, seed: Arc<AtomicU64>) -> std::io::Result<Self> {
        if cfg.dump_full_gens {
            let gens_dir = output_dir.join("gens");
            std::fs::create_dir_all(&gens_dir)?;

            Ok(Self::FullGenerations(GenerationDumper {
                output: gens_dir,
                generation: 0,
                seed,
            }))
        } else {
            Ok(Self::BestAgent(BestAgentSaver {
                path: output_dir.join("best.nn"),
            }))
        }
    }
}

impl FitnessObserver<Brain> for TrainingOutputObserver {
    fn observe(&mut self, fitnesses: &[(Brain, f32)]) {
        match self {
            Self::BestAgent(observer) => observer.observe(fitnesses),
            Self::FullGenerations(observer) => observer.observe(fitnesses),
        }
    }
}

struct TrainerObserver {
    progress: ProgressObserver,
    plotter: FitnessPlotter,
    output: TrainingOutputObserver,
}

impl TrainerObserver {
    fn from_config(cfg: &TrainerCli, output_dir: &Path, seed: Arc<AtomicU64>) -> std::io::Result<Self> {
        Ok(Self {
            progress: ProgressObserver::new_with_default_style(cfg.nb_generations as u64),
            plotter: FitnessPlotter::new(),
            output: TrainingOutputObserver::from_config(cfg, output_dir, seed)?,
        })
    }

    fn finish(&self) {
        self.progress.finish();
    }

    fn finish_and_clear(&self) {
        self.progress.finish_and_clear();
    }
}

impl FitnessObserver<Brain> for TrainerObserver {
    fn observe(&mut self, fitnesses: &[(Brain, f32)]) {
        self.progress.observe(fitnesses);
        self.plotter.observe(fitnesses);
        self.output.observe(fitnesses);
    }
}

fn play_game(brain: &Brain, cfg: &TrainerCli, rng: &mut impl Rng) -> f32 {
    let mut game = doodl_jump::Game::new();
    let mut elapsed_s = 0.0;

    let mut saved_score = game.score();
    let mut save_timer = time::DTDelay::new(cfg.stagnation_timeout_s);

    while game.score() < cfg.max_game_score {
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

    let mut rng = rand::rng();
    let seed = Arc::new(AtomicU64::new(rng.random()));
    let fitness_fn = TrainerFitnessFn {
        cfg: &cfg,
        seed: Arc::clone(&seed),
    };
    let observer = TrainerObserver::from_config(&cfg, &output, Arc::clone(&seed))?;

    let fitness_elim = FitnessEliminator::builder()
        .fitness_fn(fitness_fn)
        .observer(observer)
        .build_or_panic();

    let crossover = CrossoverRepopulator::new(
        cfg.mutation_rate,
        ReproductionSettings {
            mutation_passes: cfg.mutation_passes,
            ..Default::default()
        },
    );

    let diverg = DivergenceWeights::default();

    let mut sim = GeneticSim::new(
        Vec::gen_random(&mut rng, cfg.population_size),
        SpeciatedFitnessEliminator::from_fitness_eliminator(fitness_elim, cfg.speciation_threshold, diverg.clone()),
        SpeciatedCrossoverRepopulator::from_crossover(crossover, cfg.speciation_threshold, ActionIfIsolated::CrossoverSimilarSpecies, diverg),
    );

    for _ in 0..cfg.nb_generations {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        sim.next_generation();
        seed.store(rng.random(), Ordering::SeqCst);
    }

    if running.load(std::sync::atomic::Ordering::SeqCst) {
        sim.eliminator.inner.observer.finish();
        debug!("Finished all generations");
    } else {
        sim.eliminator.inner.observer.finish_and_clear();
        debug!("Received stop signal, stopping training...");
    }

    debug!(
        "Stopping loop. The training server ran {}\nSaving data . . .\n",
        time::format(&stopwatch.read(), 3)
    );

    let plot_path = output.join("fitness.svg");
    let root = SVGBackend::new(&plot_path, (640, 480)).into_drawing_area();

    sim.eliminator.inner.observer.plotter.plot(&root)?;

    Ok(())
}
