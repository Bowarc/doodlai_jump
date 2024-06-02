pub mod agent;

pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_FPS: usize = 20; // 60
pub const GAME_DELTA_TIME: f64 = 1./GAME_FPS as f64;
pub const NB_GENERATIONS: usize = 10_000;
pub const NB_GENOME_PER_GEN: usize = 5_000;
pub const MUTATION_RATE: f32 = 0.01;
pub const MUTATION_PASSES: usize = 3;

const NB_PLATFORM_IN: usize = 2;
const OBJECT_DATA_LEN: usize = 2;
// Player + player y velocity + data for each platform we want to send
pub const AGENT_IN: usize = OBJECT_DATA_LEN + 1 + NB_PLATFORM_IN * OBJECT_DATA_LEN;
pub const AGENT_OUT: usize = 3; // None, Left, right

pub fn generate_inputs(game: &game::Game) -> [f32; AGENT_IN] {
    let mut inputs = Vec::new();

    let rect_to_vec = |rect: &maths::Rect| -> [f32; OBJECT_DATA_LEN] {
        [
            rect.center().x as f32 / game::GAME_WIDTH as f32,
            (rect.center().y as f32 + rect.height() as f32 /2.) / game::GAME_HEIGHT as f32,
            // rect.width() as f32,
            // rect.height() as f32,
        ]
    };

    inputs.extend(rect_to_vec(&game.player.rect));
    inputs.extend([game.player.velocity.y as f32]);

    // ordered by distance to player
    inputs.extend({
        let mut platform_data = game
            .platforms
            .iter()
            .map(|platform| &platform.rect)
            .map(rect_to_vec)
            .collect::<Vec<_>>();

        platform_data.sort_unstable_by_key(|pd| {
            // By default is low to high
            let a = maths::get_distance(
                game.player.rect.center(),
                maths::Point::new(*pd.get(0).unwrap() as f64, *pd.get(1).unwrap() as f64),
            ) as i32;

            let b = maths::get_distance(
                game.player.rect.center(),
                maths::Point::new(
                    *pd.get(0).unwrap() as f64 + game::GAME_WIDTH,
                    *pd.get(1).unwrap() as f64,
                ),
            ) as i32;

            let c = maths::get_distance(
                game.player.rect.center(),
                maths::Point::new(
                    *pd.get(0).unwrap() as f64 - game::GAME_WIDTH,
                    *pd.get(1).unwrap() as f64,
                ),
            ) as i32;

            *[a, b, c].iter().min().unwrap()
        });
        let _end = platform_data.split_off(NB_PLATFORM_IN);

        platform_data.iter().cloned().flatten().collect::<Vec<_>>()
    });

    inputs.try_into().unwrap()
}
