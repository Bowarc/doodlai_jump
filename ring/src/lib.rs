pub mod agent;

pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_DT: f64 = 0.05; // 0.0166
pub const NB_GENERATIONS: usize = 1000;
pub const NB_GENOME_PER_GEN: usize = 100;

pub const AGENT_IN: usize = 25;
pub const AGENT_OUT: usize = 3; // None, Left, right

pub fn generate_inputs(game: &game::Game) -> [f32; AGENT_IN] {
    let mut inputs = Vec::new();

    let rect_to_vec = |rect: &maths::Rect| -> [f32; 4] {
        [
            rect.center().x as f32 / game::GAME_WIDTH as f32,
            rect.center().y as f32 / game::GAME_HEIGHT as f32,
            rect.width() as f32,
            rect.height() as f32,
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
            maths::get_distance(
                game.player.rect.center(),
                maths::Point::new(pd[0] as f64, pd[1] as f64),
            ) as i32
        });

        let platform_data = platform_data.iter().cloned().flatten().collect::<Vec<_>>();

        platform_data
    });

    inputs.try_into().unwrap()
}
