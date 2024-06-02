use enemy::Enemy;
use platform::Platform;
use player::Player;

pub mod enemy;
pub mod platform;
pub mod player;

const PLATFORM_LIMIT: u32 = 5;

pub const GAME_WIDTH: f64 = 540.;
pub const GAME_HEIGHT: f64 = 960.;

pub struct Game {
    // fk getters and setters
    pub enemies: Vec<Enemy>,
    pub platforms: Vec<Platform>,
    pub player: Player,
    pub scroll: i32,
    pub lost: bool,
}

impl Game {
    pub fn new() -> Self {
        let mut platforms = Vec::new();

        for i in 1..=PLATFORM_LIMIT {
            let size = maths::Vec2::new(
                platform::PLATFORM_BASE_WIDTH,
                platform::PLATFORM_BASE_HEIGHT,
            );
            let pos = maths::Point::new(
                random::get_inc(
                    platform::PLATFORM_BASE_WIDTH,
                    GAME_WIDTH - platform::PLATFORM_BASE_WIDTH,
                ),
                ((GAME_HEIGHT as u32 / PLATFORM_LIMIT) * i) as f64,
            );

            platforms.push(Platform::new(maths::Rect::new_from_center(pos, size, 0.)));
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
            println!("{}", self.score());
            return;
        }
        assert_eq!(self.platforms.len(), PLATFORM_LIMIT as usize);

        // remove platforms
        if self.player.rect.center().y - self.scroll as f64 - GAME_HEIGHT > 0. {
            // println!("Failled");
            self.lost = true;
        }

        self.platforms.retain(|platform| {
            // maths::get_distance(platform.rect.center(), self.player.rect.center()) < 1000.
            platform.rect.center().y - (self.scroll as f64) < GAME_HEIGHT
        });

        // create platforms (remove platfoms first to not iter over newly created platforms)

        while (self.platforms.len() as u32) < PLATFORM_LIMIT {
            let pos = maths::Point::new(
                random::get_inc(
                    platform::PLATFORM_BASE_WIDTH / 2.,
                    GAME_WIDTH - platform::PLATFORM_BASE_WIDTH / 2.,
                ),
                self.scroll as f64,
            );
            let size = maths::Vec2::new(
                platform::PLATFORM_BASE_WIDTH,
                platform::PLATFORM_BASE_HEIGHT,
            );

            let rect = maths::Rect::new_from_center(pos, size, 0.);

            self.platforms.push(Platform::new(rect));
        }

        // update player

        self.player.update(&self.platforms, dt);

        // update 'camera'
        // let t = 20.0 * dt;
        // self.scroll = (self.scroll as f64 * (1. - t) + self.player.rect.center().y - (GAME_HEIGHT / 2.) * t) as u32;
        let new_scroll = (self.player.rect.center().y - (GAME_HEIGHT / 2.)) as i32;
        if new_scroll < self.scroll {
            self.scroll = new_scroll;
        }

        // println!("{}", self.score());
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
