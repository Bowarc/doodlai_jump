use enemy::Enemy;
use platform::Platform;
use player::Player;

pub mod enemy;
pub mod platform;
pub mod player;

const PLATFORM_LIMIT: u32 = 5;

pub struct Game {
    // fk getters and setters
    pub enemies: Vec<Enemy>,
    pub platforms: Vec<Platform>,
    pub player: Player,
    pub scroll: i32,
}

impl Game {
    pub fn new() -> Self {
        let mut platforms = Vec::new();

        for i in 1..PLATFORM_LIMIT {
            let x = random::get_inc(150., 540. - 150.);
            let y = (960 / PLATFORM_LIMIT) * i;
            platforms.push(Platform::new((x, y as f64)));
        }

        Self {
            enemies: Vec::new(),
            platforms,
            player: Player::new(),
            scroll: 0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        // remove platforms

        self.platforms.retain(|platform| {
            // maths::get_distance(platform.rect.center(), self.player.rect.center()) < 1000.
            platform.rect.center().y - (self.scroll as f64) < 960.
        });

        // create platforms (remove platfoms first to not iter over newly created platforms)

        while (self.platforms.len() as u32) < PLATFORM_LIMIT {
            let x = random::get_inc(150., 540. - 150.);
            let y = self.scroll as f64 + 0.;

            self.platforms.push(Platform::new((x, y)));
        }

        // update player

        self.player.update(dt, &self.platforms);

        // update 'camera'
        // let t = 20.0 * dt;
        // self.scroll = (self.scroll as f64 * (1. - t) + self.player.rect.center().y - (960. / 2.) * t) as u32;
        let new_scroll = (self.player.rect.center().y - (960. / 2.)) as i32;
        if new_scroll < self.scroll {
            self.scroll = new_scroll;
        }
    }

    pub fn player_move_left(&mut self) {
        self.player.current_direction = Some(false)
    }

    pub fn player_move_right(&mut self) {
        self.player.current_direction = Some(true)
    }

    pub fn player_shoot(&mut self) {}
}
