use enemy::Enemy;
use platform::Platform;
use player::Player;

pub mod player;
pub mod enemy;
pub mod platform;


const PLATFORM_LIMIT: u32 = 10;

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

    pub fn update(&mut self, dt: f64) {

        // remove platforms

        self.platforms.retain(|platform|{
            maths::get_distance(platform.rect.center(), self.player.rect.center()) < 1000.
        });


        // create platforms (remove platfoms first to not iter over newly created platforms)

        while (self.platforms.len() as u32) < PLATFORM_LIMIT {
            let x = random::get_inc(150., 540. - 150.);
            let y = self.scroll as f64 + 960.;

            self.platforms.push(Platform::new((x, y)));
        }

        // update player

        self.player.update(dt, &self.platforms);
    }

    pub fn player_move_left(&mut self){
        self.player.current_direction  = Some(false)
    }

    pub fn player_move_right(&mut self){
        self.player.current_direction  = Some(true)
    }

    pub fn player_shoot(&mut self){

    }
}