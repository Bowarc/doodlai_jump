use neat::GenerateRandomCollection;

#[macro_use]
extern crate log;
mod agent;
mod utils;

const NB_GAMES: usize = 10;
const GAME_TIME_S: usize = 100; // Nb of secconds we let the ai play the game before registering their scrore
const GAME_DT: f64 = 0.0166;
const NB_GENERATIONS: usize = 15;
const NB_GENOME_PER_GEN: usize = 100;

static mut LOADED_NNT: Option<neat::NeuralNetworkTopology<{agent::IN}, {agent::OUT}>> = None;

fn fitness(dna: &agent::DNA) -> f32 {
    let agent = agent::Agent::from(dna);

    let mut fitness = 0.;

    let mut rng = rand::thread_rng();

    (0..NB_GAMES)
        .map(|_| play_game(&agent, &mut rng))
        .sum::<f32>()
        / NB_GAMES as f32
}

fn play_game(agent: &agent::Agent, rng: &mut rand::rngs::ThreadRng) -> f32 {
    let mut game = game::Game::new();

    // loop for the number of frames we want to play, should be enough frames to play 100s at 60fps
    for i in 0..((1. / GAME_DT) * GAME_TIME_S as f64) as usize {
        let mut inputs = Vec::new();

        let rect_to_vec = |rect: &maths::Rect| -> [f32; 4] {
            [
                rect.center().x as f32,
                rect.center().y as f32,
                rect.width() as f32,
                rect.height() as f32,
            ]
        };

        inputs.extend(rect_to_vec(&game.player.rect));

        // ordered by distance to player
        let closest_platforms = {
            let mut temp = game.platforms.clone();
            temp.sort_unstable_by_key(|platfrom| {
                maths::get_distance(platfrom.rect.center(), game.player.rect.center()) as i32
            });
            temp
        };

        for platform in closest_platforms {
            inputs.extend(rect_to_vec(&platform.rect));
        }

        assert_eq!(inputs.len(), 24);

        let action =
            neat::MaxIndex::max_index(agent.network.predict(inputs.try_into().unwrap()).iter());

        match action {
            0 => (), // No action
            1 => game.player_move_left(),
            2 => game.player_move_right(),
            _ => (),
        }

        game.update(GAME_DT)
    }

    game.score()
}

fn main() {
    let config = logger::LoggerConfig::default().set_level(log::LevelFilter::Debug);

    logger::init(config, Some("./log/ring.log"));

    let stopwatch = time::Stopwatch::start_new();

    let running = utils::set_up_ctrlc();

    debug!("Starting training server");

    let nnt  = serde_json::from_str::<neat::NNTSerde<{agent::IN}, {agent::OUT}>>(include_str!("./nnt.json")).unwrap();

    unsafe{
        LOADED_NNT = Some(nnt.into());
    };


    let mut sim = neat::GeneticSim::new(
        Vec::gen_random(NB_GENOME_PER_GEN),
        fitness,
        neat::crossover_pruning_nextgen,
    );


    for i in 0..NB_GENERATIONS {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            info!("Exit requested");
            break;
        }
        debug!("Generation {i}, {} since start",  time::format(stopwatch.read(), 2));
        sim.next_generation()
    }
    debug!("Generation done, parsing outputs");

    let mut fits = sim
        .genomes
        .iter()
        .map(|dna| (dna, fitness(dna)))
        .collect::<Vec<_>>();
    fits.sort_by(|(dnaa, fita), (dnab, fitb)| fita.partial_cmp(&fitb).unwrap());

    debug!("Max score: {}", &fits[0].1);

    {
        let intermediate = neat::NNTSerde::from(&fits[0].0.network);
        let serialized = serde_json::to_string(&intermediate).unwrap();
        println!("\n{}", serialized);
    }

    debug!(
        "Stopping loop. The training server ran {}",
        time::format(stopwatch.read(), 3)
    );
}
