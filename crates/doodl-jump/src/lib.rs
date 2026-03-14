use enemy::Enemy;
use platform::Platform;
use player::Player;
use rand::prelude::*;
use std::collections::VecDeque;

use crate::player::MoveDirection;

pub mod enemy;
pub mod platform;
pub mod player;

/// Number of platforms that should be on-screen per player.
pub const PLATFORMS_PER_PLAYER: usize = 5;

pub const GAME_WIDTH: f64 = 540.;
pub const GAME_HEIGHT: f64 = 960.;

pub struct Game {
    // fk getters and setters
    pub enemies: Vec<Enemy>,
    /// Platforms sorted top (front) -> bottom (back).
    pub platforms: VecDeque<Platform>,
    /// All active players.
    pub players: Vec<Player>,
    /// Per-player camera scroll (most-negative == highest camera).
    pub scrolls: Vec<i32>,
    /// Per-player death flags.
    pub lost: Vec<bool>,

    /// Cached highest (most-negative) scroll reached by any player so far.
    ///
    /// We only need to spawn new platforms when this value decreases (i.e. some player
    /// progressed upward beyond the previous max).
    pub highest_scroll: i32,

    /// Game RNG (seeded) used for deterministic platform generation.
    pub rng: StdRng,
}

impl Game {
    /// Create a new game with `num_players` players using a nondeterministic seed.
    ///
    /// If you want deterministic games (reproducible platform layouts), use
    /// [`Game::new_with_seed`].
    pub fn new(num_players: usize) -> Self {
        // Use a nondeterministic seed by sampling a u64 from the thread RNG.
        let seed: u64 = rand::rng().random();
        Self::new_with_seed(num_players, seed)
    }

    /// Create a new seeded game with `num_players` players.
    ///
    /// Using the same `seed` will generate the same platform sequence (given the same
    /// simulation stepping / dt usage).
    pub fn new_with_seed(num_players: usize, seed: u64) -> Self {
        Self::new_with_rng(num_players, StdRng::seed_from_u64(seed))
    }

    fn new_with_rng(num_players: usize, mut rng: StdRng) -> Self {
        let num_players = num_players.max(1);

        let mut platforms = VecDeque::new();
        let spacing = GAME_HEIGHT / PLATFORMS_PER_PLAYER as f64;

        for i in 0..PLATFORMS_PER_PLAYER {
            let size = math::Vec2::new(
                platform::PLATFORM_BASE_WIDTH,
                platform::PLATFORM_BASE_HEIGHT,
            );

            let min_x = platform::PLATFORM_BASE_WIDTH / 2.;
            let max_x = GAME_WIDTH - platform::PLATFORM_BASE_WIDTH / 2.;
            let x = rng.random_range(min_x..=max_x);

            let pos = math::Point::new(x, spacing * (i as f64 + 0.5));

            platforms.push_back(Platform::new(math::Rect::new_from_center(pos, size, 0.)));
        }

        let mut players = Vec::with_capacity(num_players);
        for _ in 0..num_players {
            players.push(Player::new());
        }

        Self {
            enemies: Vec::new(),
            platforms,
            players,
            scrolls: vec![0; num_players],
            lost: vec![false; num_players],
            highest_scroll: 0,
            rng,
        }
    }

    pub fn update(&mut self, dt: f64) {
        let n = self.players.len();
        if n == 0 {
            return;
        }

        // track whether the global max_scroll improved (decreased).
        let mut max_scroll_improved = false;

        // update per-player lost state and physics.
        for i in 0..n {
            if self.lost[i] {
                continue;
            }

            if self.players[i].rect.center().y - self.scrolls[i] as f64 - GAME_HEIGHT > 0. {
                self.lost[i] = true;
                continue;
            }

            self.players[i].update(&self.platforms, dt);

            let new_scroll = (self.players[i].rect.center().y - (GAME_HEIGHT / 2.)) as i32;
            if new_scroll < self.scrolls[i] {
                self.scrolls[i] = new_scroll;
            }

            if self.scrolls[i] < self.highest_scroll {
                self.highest_scroll = self.scrolls[i];
                max_scroll_improved = true;
            }
        }

        self.recycle_platforms(max_scroll_improved);
    }

    fn recycle_platforms(&mut self, max_scroll_improved: bool) {
        let spacing = GAME_HEIGHT / PLATFORMS_PER_PLAYER as f64;
        let bottom_offscreen_threshold = GAME_HEIGHT + platform::PLATFORM_BASE_HEIGHT;

        // Determine the lowest (worst) camera: the maximum scroll value.
        // Platforms should not be removed until they are off the lowest agent's screen.
        let lowest_scroll = self.scrolls.iter().copied().max().unwrap_or(0) as f64;

        // Despawn platforms only when they are off the bottom of the lowest player's screen.
        while self
            .platforms
            .back()
            .map(|p| p.rect.center().y - lowest_scroll > bottom_offscreen_threshold)
            .unwrap_or(false)
        {
            self.platforms.pop_back();
        }

        // spawn platforms only when the highest player makes progress (i.e. max_scroll improves).
        if max_scroll_improved {
            let required_top_y =
                self.highest_scroll as f64 - spacing * (PLATFORMS_PER_PLAYER as f64 - 0.5);

            while self
                .platforms
                .front()
                .map(|p| p.rect.center().y > required_top_y)
                .unwrap_or(true)
            {
                let new_y = self
                    .platforms
                    .front()
                    .map(|p| p.rect.center().y - spacing)
                    .unwrap_or(self.highest_scroll as f64 - spacing);

                let min_x = platform::PLATFORM_BASE_WIDTH / 2.;
                let max_x = GAME_WIDTH - platform::PLATFORM_BASE_WIDTH / 2.;
                let x = self.rng.random_range(min_x..=max_x);

                let pos = math::Point::new(x, new_y);

                let size = math::Vec2::new(
                    platform::PLATFORM_BASE_WIDTH,
                    platform::PLATFORM_BASE_HEIGHT,
                );
                self.platforms
                    .push_front(Platform::new(math::Rect::new_from_center(pos, size, 0.)));
            }
        }
    }

    pub fn player_move(&mut self, player_index: usize, direction: MoveDirection) {
        if let Some(p) = self.players.get_mut(player_index) {
            p.current_direction = direction;
        }
    }

    pub fn player_shoot(&mut self, _player_index: usize) {}

    pub fn score(&self, player_index: usize) -> f32 {
        self.scrolls
            .get(player_index)
            .map(|s| -(*s as f32))
            .unwrap_or(0.0)
    }
}
