use clap::Parser;
use rand::RngExt as _;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "Train a Doodl Jump agent")]
pub struct TrainerCli {
    /// Number of game attempts used to average a genome's fitness.
    #[arg(long, default_value_t = 3)]
    pub nb_games: usize,

    /// Simulation FPS used by the game update loop.
    #[arg(long, default_value_t = 20)]
    pub game_fps: usize,

    /// Use variable frame time instead of fixed dt (dt is randomly jittered around 1 / game_fps).
    #[arg(long, default_value_t = false)]
    pub variable_dt: bool,

    /// Max relative jitter used with --variable-dt. 0.25 means dt in [0.75x, 1.25x].
    #[arg(long, default_value_t = 0.25)]
    pub variable_dt_jitter: f64,

    /// Number of generations to train.
    #[arg(long, default_value_t = 150)]
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

    /// Grace period (seconds) before time penalty starts affecting fitness.
    #[arg(long, default_value_t = 3.0)]
    pub fitness_time_grace_s: f64,

    /// Fitness penalty applied per second after the grace period.
    #[arg(long, default_value_t = 0.5)]
    pub fitness_time_penalty_per_s: f32,

    /// Directory where training outputs are saved.
    #[arg(long, default_value = "./sim")]
    pub output_dir: PathBuf,
}

impl TrainerCli {
    pub fn game_delta_time(&self) -> f64 {
        1.0 / self.game_fps as f64
    }

    pub fn frame_delta_time(&self, rng: &mut impl rand::Rng) -> f64 {
        let base_dt = self.game_delta_time();
        if !self.variable_dt {
            return base_dt;
        }

        let min_scale = 1.0 - self.variable_dt_jitter;
        let max_scale = 1.0 + self.variable_dt_jitter;

        base_dt * rng.random_range(min_scale..=max_scale)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.nb_games == 0 {
            return Err("--nb-games must be greater than 0".to_string());
        }

        if self.game_fps == 0 {
            return Err("--game-fps must be greater than 0".to_string());
        }

        if !(0.0..1.0).contains(&self.variable_dt_jitter) {
            return Err("--variable-dt-jitter must be in [0.0, 1.0)".to_string());
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

        if self.fitness_time_grace_s < 0.0 {
            return Err("--fitness-time-grace-s must be >= 0".to_string());
        }

        if self.fitness_time_penalty_per_s < 0.0 {
            return Err("--fitness-time-penalty-per-s must be >= 0".to_string());
        }

        Ok(())
    }
}
