pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_FPS: usize = 20; // 60
pub const GAME_DELTA_TIME: f64 = 1. / GAME_FPS as f64;
pub const NB_GENERATIONS: usize = 1000;
pub const NB_GENOME_PER_GEN: usize = 500;
pub const MUTATION_RATE: f32 = 0.05;
pub const MUTATION_PASSES: usize = 3;

const NB_PLATFORM_IN: usize = 3;
const OBJECT_DATA_LEN: usize = 2;
// Player y velocity + processed dt scale + data for each platform we want to send
pub const AGENT_IN: usize = 1 + 1 + NB_PLATFORM_IN * OBJECT_DATA_LEN;
pub const AGENT_OUT: usize = 3; // None, Left, right

fn wrapped_dx(player_x: f64, target_x: f64, width: f64) -> f64 {
    (target_x - player_x + width / 2.).rem_euclid(width) - width / 2.
}

pub fn generate_inputs(game: &doodl_jump::Game, processed_dt: f64, reference_dt: f64) -> [f32; AGENT_IN] {
    let mut inputs = Vec::with_capacity(AGENT_IN);
    let player_center = game.player.rect.center();
    let dt_scale = if reference_dt > 0.0 {
        processed_dt / reference_dt
    } else {
        1.0
    };

    inputs.push((game.player.velocity.y / 1000.) as f32);
    inputs.push(dt_scale as f32);

    let mut platform_data = game
        .platforms
        .iter()
        .map(|platform| {
            let platform_center = platform.rect.center();
            let dx = wrapped_dx(player_center.x, platform_center.x, doodl_jump::GAME_WIDTH);
            let dy = platform_center.y - player_center.y;
            let distance_sq = dx * dx + dy * dy;

            (distance_sq, dx, dy)
        })
        .collect::<Vec<_>>();

    platform_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    for (_, dx, dy) in platform_data.into_iter().take(NB_PLATFORM_IN) {
        inputs.push((dx / (doodl_jump::GAME_WIDTH * 0.5)) as f32);
        inputs.push((dy / doodl_jump::GAME_HEIGHT) as f32);
    }

    while inputs.len() < AGENT_IN {
        inputs.push(0.0);
    }

    inputs.try_into().unwrap()
}

pub type Brain = neat::NeuralNetwork<AGENT_IN, AGENT_OUT>;
