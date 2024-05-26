use enemy::Enemy;
use platform::Platform;
use player::Player;

pub mod player;
pub mod enemy;
pub mod platform;

pub struct Game{
    // fk getters and setters 
    pub enemies: Vec<Enemy>,
    pub platforms: Vec<Platform>,
    pub player: Player,
    pub scroll: u32,
}

impl Game{
    pub fn new() -> Self{

        Self{
            enemies: Vec::new(),
            platforms: Vec::new(),
            player: Player::new(),
            scroll: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {

    }
}