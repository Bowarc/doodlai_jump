use enemy::Enemy;
use platform::Platform;
use player::Player;
use std::collections::VecDeque;

pub mod enemy;
pub mod platform;
pub mod player;

pub const NUM_PLATFORMS: usize = 5;

pub const GAME_WIDTH: f64 = 540.;
pub const GAME_HEIGHT: f64 = 960.;

pub struct Game {
    // fk getters and setters
    pub enemies: Vec<Enemy>,
    pub platforms: VecDeque<Platform>,
    pub player: Player,
    pub scroll: i32,
    pub lost: bool,
}

impl Game {
    pub fn new() -> Self {
        let mut platforms = VecDeque::new();
        let spacing = GAME_HEIGHT / NUM_PLATFORMS as f64;

        for i in 0..NUM_PLATFORMS {
            let size = math::Vec2::new(
                platform::PLATFORM_BASE_WIDTH,
                platform::PLATFORM_BASE_HEIGHT,
            );
            let pos = math::Point::new(
                random::get_inc(
                    platform::PLATFORM_BASE_WIDTH / 2.,
                    GAME_WIDTH - platform::PLATFORM_BASE_WIDTH / 2.,
                ),
                spacing * (i as f64 + 0.5),
            );

            platforms.push_back(Platform::new(math::Rect::new_from_center(pos, size, 0.)));
        }

        Self {
            enemies: Vec::new(),
            platforms,
            player: Player::new(),
            scroll: 0,
            lost: false,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if self.lost {
            return;
        }

        if self.player.rect.center().y - self.scroll as f64 - GAME_HEIGHT > 0. {
            self.lost = true;
            return;
        }

        self.player.update(&self.platforms, dt);
        let new_scroll = (self.player.rect.center().y - (GAME_HEIGHT / 2.)) as i32;
        if new_scroll < self.scroll {
            self.scroll = new_scroll;
        }

        self.recycle_platforms();
    }

    fn recycle_platforms(&mut self) {
        let spacing = GAME_HEIGHT / NUM_PLATFORMS as f64;
        let bottom_offscreen_threshold = GAME_HEIGHT + platform::PLATFORM_BASE_HEIGHT;

        while self
            .platforms
            .back()
            .map(|p| p.rect.center().y - self.scroll as f64 > bottom_offscreen_threshold)
            .unwrap_or(false)
        {
            self.platforms.pop_back();

            let new_y = self
                .platforms
                .front()
                .map(|p| p.rect.center().y - spacing)
                .unwrap_or(self.scroll as f64 - spacing);

            let pos = math::Point::new(
                random::get_inc(
                    platform::PLATFORM_BASE_WIDTH / 2.,
                    GAME_WIDTH - platform::PLATFORM_BASE_WIDTH / 2.,
                ),
                new_y,
            );

            let size = math::Vec2::new(
                platform::PLATFORM_BASE_WIDTH,
                platform::PLATFORM_BASE_HEIGHT,
            );
            self.platforms
                .push_front(Platform::new(math::Rect::new_from_center(pos, size, 0.)));
        }

        while self.platforms.len() < NUM_PLATFORMS {
            let new_y = self
                .platforms
                .front()
                .map(|p| p.rect.center().y - spacing)
                .unwrap_or(self.scroll as f64);

            let pos = math::Point::new(
                random::get_inc(
                    platform::PLATFORM_BASE_WIDTH / 2.,
                    GAME_WIDTH - platform::PLATFORM_BASE_WIDTH / 2.,
                ),
                new_y,
            );

            let size = math::Vec2::new(
                platform::PLATFORM_BASE_WIDTH,
                platform::PLATFORM_BASE_HEIGHT,
            );
            self.platforms
                .push_front(Platform::new(math::Rect::new_from_center(pos, size, 0.)));
        }

        while self.platforms.len() > NUM_PLATFORMS {
            self.platforms.pop_back();
        }
    }

    pub fn player_move_left(&mut self) {
        self.player.current_direction = Some(false)
    }

    pub fn player_move_right(&mut self) {
        self.player.current_direction = Some(true)
    }

    pub fn player_shoot(&mut self) {}

    pub fn score(&self) -> f32 {
        return -self.scroll as f32;
    }
}
