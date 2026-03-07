//! Shared trainer configuration and utilities.

/// Number of game attempts used to average a genome's fitness.
/// The trainer binary exposes its own CLI defaults; these constants are
/// here for library users that prefer compile-time defaults.
pub const NB_GAMES: usize = 3;

/// Number of seconds we let the AI play before registering the score.
pub const GAME_TIME_S: usize = 20;

/// Simulation frames per second used by the game update loop.
pub const GAME_FPS: usize = 20;

/// Fixed time step derived from `GAME_FPS`.
pub const GAME_DELTA_TIME: f64 = 1.0 / GAME_FPS as f64;

/// Number of generations to run during training.
pub const NB_GENERATIONS: usize = 1000;

/// Number of genomes per generation.
pub const NB_GENOME_PER_GEN: usize = 500;

/// Default mutation rate applied during crossover/repopulation.
pub const MUTATION_RATE: f32 = 0.05;

/// Number of mutation passes during reproduction.
pub const MUTATION_PASSES: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_constants() {
        // Simple sanity checks so these values are exercised in CI
        assert!(NB_GAMES > 0);
        assert!(GAME_FPS > 0);
        assert!(NB_GENERATIONS > 0);
        assert!(NB_GENOME_PER_GEN > 0);
        assert!((0.0..=1.0).contains(&MUTATION_RATE));
    }
}
