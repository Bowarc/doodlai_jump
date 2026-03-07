//! Shared AI‑player utilities used by both the trainer and the display crate.
//!
//! Contains the `Brain` type alias, network I/O constants, input generation
//! (perception) and action application (behavior).

use neat::MaxIndex;

/// How many nearest platforms we feed into the network.
const NB_PLATFORM_IN: usize = 5;
/// Data values per platform (dx, dy).
const OBJECT_DATA_LEN: usize = 2;

/// Network input size.
///
/// Layout:
/// - player vertical velocity (scaled)
/// - raw frame dt in seconds (scaled down to keep values small)
/// - for each of the [`NB_PLATFORM_IN`] nearest platforms:
///   - dx (wrapped, normalised to roughly \[-1, 1\])
///   - dy (normalised by game height)
pub const AGENT_IN: usize = 1 + 1 + NB_PLATFORM_IN * OBJECT_DATA_LEN;

/// Network output size: None, Left, Right.
pub const AGENT_OUT: usize = 3;

/// Neural network type used as the agent brain.
pub type Brain = neat::NeuralNetwork<AGENT_IN, AGENT_OUT>;

/// Shortest signed horizontal distance on a wrapping axis of the given `width`.
fn wrapped_dx(player_x: f64, target_x: f64, width: f64) -> f64 {
    (target_x - player_x + width / 2.0).rem_euclid(width) - width / 2.0
}

/// Build the neural network input array from the current game state.
///
/// * `game` - current game state.
/// * `dt`   - raw frame delta-time in seconds. This is passed straight to the
///   network (scaled down by 10x to keep values in a reasonable range) so the
///   agent can learn to compensate for varying frame durations regardless of
///   what framerate it was trained at.
pub fn generate_inputs(game: &doodl_jump::Game, dt: f32) -> [f32; AGENT_IN] {
    let mut inputs = Vec::with_capacity(AGENT_IN);
    let player_center = game.player.rect.center();

    // 0: Scaled vertical velocity
    inputs.push((game.player.velocity.y / 1000.0) as f32);

    // 1: Raw dt scaled down (e.g. 0.033s at 30fps → ~0.0033)
    inputs.push(dt * 0.1);

    // 2-7: Platforms
    //
    // Platforms are ordered front (top / smallest y) -> back (bottom / largest y).
    for platform in &game.platforms {
        if inputs.len() == AGENT_IN {
            break;
        }

        let pc = platform.rect.center();
        let dx = wrapped_dx(player_center.x, pc.x, doodl_jump::GAME_WIDTH);
        let dy = pc.y - player_center.y;

        inputs.push((dx / (doodl_jump::GAME_WIDTH * 0.5)) as f32);
        inputs.push((dy / doodl_jump::GAME_HEIGHT) as f32);
    }

    // pad with zeros when fewer platforms exist than NB_PLATFORM_IN
    while inputs.len() < AGENT_IN {
        inputs.push(0.0);
    }

    inputs
        .try_into()
        .expect("generate_inputs: AGENT_IN size mismatch")
}

/// Execute the action chosen by the network on the game.
///
/// Picks the output neuron with the highest activation:
/// - 0 -> no action
/// - 1 -> move left
/// - 2 -> move right
pub fn apply_action(game: &mut doodl_jump::Game, output: &[f32; AGENT_OUT]) {
    game.player_move(output.iter().max_index().unwrap().into());
}
