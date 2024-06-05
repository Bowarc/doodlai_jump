pub mod agent;

pub const NB_GAMES: usize = 3;
pub const GAME_TIME_S: usize = 20; // Nb of secconds we let the ai play the game before registering their scrore
pub const GAME_FPS: usize = 20; // 60
pub const GAME_DELTA_TIME: f64 = 1. / GAME_FPS as f64;
pub const NB_GENERATIONS: usize = 20_000;
pub const NB_GENOME_PER_GEN: usize = 5_000;
pub const MUTATION_RATE: f32 = 0.01;
pub const MUTATION_PASSES: usize = 3;

const NB_PLATFORM_IN: usize = 2;
const OBJECT_DATA_LEN: usize = 2;
// Player x + player y velocity + data for each platform we want to send
pub const AGENT_IN: usize = 1 + 1 + NB_PLATFORM_IN * OBJECT_DATA_LEN;
pub const AGENT_OUT: usize = 3; // None, Left, right

pub fn generate_inputs(game: &game::Game) -> [f32; AGENT_IN] {
    let mut inputs = Vec::new();

    let build_single_input = |obj_rect: &maths::Rect, player: &maths::Rect| -> [f32; 2] {
        let a = (
            obj_rect.center().x,
            maths::get_distance(obj_rect.center(), player.center()) as i32,
        );

        let b = (
            obj_rect.center().x,
            maths::get_distance(
                obj_rect.center(),
                maths::Point::new(player.center().x + game::GAME_WIDTH, player.center().y),
            ) as i32,
        );

        let c = (
            obj_rect.center().x,
            maths::get_distance(
                obj_rect.center(),
                maths::Point::new(player.center().x - game::GAME_WIDTH, player.center().y),
            ) as i32,
        );

        let min = [a, b, c].iter().cloned().min_by_key(|(_x, d)| *d).unwrap();

        [min.0 as f32, min.1 as f32]
    };

    // inputs.extend(rect_to_vec(&game.player.rect));
    inputs.extend([
        game.player.rect.center().x as f32,
        game.player.velocity.y as f32,
    ]);

    // ordered by distance to player
    inputs.extend({
        let mut platform_data = game
            .platforms
            .iter()
            .map(|platform| {
                build_single_input(&platform.rect, &game.player.rect)
            })
            // .map(rect_to_vec)
            .collect::<Vec<_>>();

        platform_data.sort_unstable_by_key(|xd| {
            // By default is low to high
            xd[1] as i32
        });
        let _end = platform_data.split_off(NB_PLATFORM_IN);

        platform_data
            .iter()
            .cloned()
            .flatten()
            .collect::<Vec<f32>>()
    });

    inputs.try_into().unwrap()
}


// pub fn generate_inputs(game: &game::Game) -> [f32; AGENT_IN] {
//     let mut inputs = Vec::new();

//     let build_single_input = |obj_rect: &maths::Rect, player: &maths::Rect| -> [f32; 2] {
//         let a = (
//             obj_rect.center().x,
//             maths::get_distance(obj_rect.center(), player.center()) as i32,
//         );

//         let b = (
//             obj_rect.center().x,
//             maths::get_distance(
//                 obj_rect.center(),
//                 maths::Point::new(player.center().x + game::GAME_WIDTH, player.center().y),
//             ) as i32,
//         );

//         let c = (
//             obj_rect.center().x,
//             maths::get_distance(
//                 obj_rect.center(),
//                 maths::Point::new(player.center().x - game::GAME_WIDTH, player.center().y),
//             ) as i32,
//         );

//         let min = [a, b, c].iter().cloned().min_by_key(|(_x, d)| *d).unwrap();

//         [min.0 as f32, min.1 as f32]
//     };

//     // inputs.extend(rect_to_vec(&game.player.rect));
//     inputs.extend([
//         game.player.rect.center().x as f32,
//         game.player.velocity.y as f32,
//     ]);

//     // ordered by distance to player
//     inputs.extend({
//         let mut platform_data = game
//             .platforms
//             .iter()
//             .map(|platform| {
//                 [
//                     platform.rect.center().x ,
//                     maths::get_distance(platform.rect.center(), game.player.rect.center()),
//                 ]
//             })
//             // .map(rect_to_vec)
//             .collect::<Vec<_>>();

//         platform_data.sort_unstable_by_key(|xd| {
//             // By default is low to high
//             xd[1] as i32
//         });
//         let _end = platform_data.split_off(NB_PLATFORM_IN);

//         platform_data
//             .iter()
//             .cloned()
//             .flatten()
//             .map(|v| v as f32)
//             .collect::<Vec<f32>>()
//     });

//     inputs.try_into().unwrap()
// }
