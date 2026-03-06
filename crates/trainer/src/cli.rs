use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(author = "Boward, HyerCodec", about = "Train a Doodl Jump agent")]
pub struct TrainerCli {
    /// Number of game attempts used to average a genome's fitness.
    #[arg(long, default_value_t = 3)]
    pub nb_games: usize,

    /// Simulation FPS used by the game update loop.
    #[arg(long, default_value_t = 20)]
    pub game_fps: usize,

    /// Number of generations to train.
    #[arg(long, default_value_t = 1000)]
    pub nb_generations: usize,

    /// Number of genomes per generation.
    #[arg(long, default_value_t = 500)]
    pub nb_genome_per_gen: usize,

    /// Mutation rate applied during crossover/repopulation.
    #[arg(long, default_value_t = 0.05)]
    pub mutation_rate: f32,

    /// Number of mutation passes during reproduction.
    #[arg(long, default_value_t = 3)]
    pub mutation_passes: usize,

    /// Seconds without score increase before marking the run as stagnant.
    #[arg(long, default_value_t = 10.0)]
    pub stagnation_timeout_s: f64,

    /// Directory where training outputs are saved.
    #[arg(long, default_value = "./sim")]
    pub output_dir: PathBuf,
}

impl TrainerCli {
    pub fn game_delta_time(&self) -> f64 {
        1.0 / self.game_fps as f64
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.nb_games == 0 {
            return Err("--nb-games must be greater than 0".to_string());
        }

        if self.game_fps == 0 {
            return Err("--game-fps must be greater than 0".to_string());
        }

        if self.nb_generations == 0 {
            return Err("--nb-generations must be greater than 0".to_string());
        }

        if self.nb_genome_per_gen == 0 {
            return Err("--nb-genome-per-gen must be greater than 0".to_string());
        }

        if !(0.0..=1.0).contains(&self.mutation_rate) {
            return Err("--mutation-rate must be between 0.0 and 1.0".to_string());
        }

        if self.stagnation_timeout_s <= 0.0 {
            return Err("--stagnation-timeout-s must be greater than 0".to_string());
        }

        Ok(())
    }
}
