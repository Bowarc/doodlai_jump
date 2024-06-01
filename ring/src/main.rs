use neat::{GenerateRandomCollection, MaxIndex};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ring::{
    agent, AGENT_IN, AGENT_OUT, GAME_DT, GAME_TIME_S, NB_GAMES, NB_GENERATIONS, NB_GENOME_PER_GEN,
};
use std::io::Write as _;

#[macro_use]
extern crate log;

mod utils;

static mut LOADED_NNT: Option<neat::NeuralNetworkTopology<{ AGENT_IN }, { AGENT_OUT }>> = None;

fn fitness(dna: &agent::DNA) -> f32 {
    let agent = agent::Agent::from(dna);

    (0..NB_GAMES).map(|_| play_game(&agent)).sum::<f32>() / NB_GAMES as f32
}

fn play_game(agent: &agent::Agent) -> f32 {
    let mut game = game::Game::new();

    // loop for the number of frames we want to play, should be enough frames to play 100s at 60fps
    for _ in 0..((1. / GAME_DT) * GAME_TIME_S as f64) as usize {
        if game.lost {
            break;
        }

        let output = agent.network.predict(ring::generate_inputs(&game));

        match output.iter().max_index() {
            0 => (), // No action
            1 => game.player_move_left(),
            2 => game.player_move_right(),
            _ => (),
        }

        game.update(GAME_DT);
    }

    game.score()
}

// F: FitnessFn<G> + Send + Sync,
// NG: NextgenFn<G> + Send + Sync,
// G: Sized + Send,

fn sort_genomes(
    sim: &neat::GeneticSim<
        impl Fn(&agent::DNA) -> f32 + Send + Sync,
        impl Fn(Vec<(agent::DNA, f32)>) -> Vec<agent::DNA> + Send + Sync,
        agent::DNA,
    >,
) -> Vec<(&agent::DNA, f32)> {
    // Iter with rayon

    let mut genomes = sim
        .genomes
        .par_iter()
        .map(|dna| (dna, fitness(dna)))
        .collect::<Vec<(&agent::DNA, f32)>>();

    genomes.sort_unstable_by_key(|(_dna, fitness)| -fitness as i32);

    genomes
}

fn main() {
    let config = logger::LoggerConfig::default().set_level(log::LevelFilter::Debug);

    logger::init(config, Some("./log/ring.log"));

    let stopwatch = time::Stopwatch::start_new();

    let running = utils::set_up_ctrlc();

    debug!("Starting training server");

    // let nnt = serde_json::from_str::<neat::NNTSerde<{ agent::IN }, { agent::OUT }>>(include_str!(
    //     "./nnt.json"
    // ))
    // .unwrap();

    // unsafe {
    //     LOADED_NNT = Some(nnt.into());
    // };

    let mut sim = neat::GeneticSim::new(
        Vec::gen_random(NB_GENOME_PER_GEN),
        fitness,
        neat::crossover_pruning_nextgen,
    );

    for i in 0..NB_GENERATIONS {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        debug!("Generation {}/{}", i + 1, NB_GENERATIONS,);
        let stopwatch = time::Stopwatch::start_new();

        {
            let data = sim
                .genomes
                .iter()
                .map(|dna| format!("{dna:?}\n"))
                .collect::<String>();
            std::fs::File::create("./sim.backup")
                .unwrap()
                .write_all(data.as_bytes())
                .unwrap();
        }

        sim.next_generation();

        let (sorted_genome, sort_duration) = time::timeit(|| sort_genomes(&sim));

        let best = sorted_genome.first().unwrap();

        println!(
            "Gen {} done, took {}\nResults: {:.0}/{:.0}/{:.0}. sorted in {}.",
            i + 1,
            time::format(stopwatch.read(), 1),
            best.1,
            sorted_genome.get(NB_GENOME_PER_GEN / 2).unwrap().1,
            sorted_genome.last().unwrap().1,
            time::format(sort_duration, 1)
        );

        std::fs::File::create(format!("./generations/gen{}_score{:.0}.json", i, best.1))
            .unwrap()
            .write_all(
                serde_json::to_string(&neat::NNTSerde::from(&best.0.network))
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
    }
    debug!("Generation done, parsing outputs");

    let genomes = sort_genomes(&sim);

    debug!(
        "Max score: {}\nMin score: {}",
        &genomes.first().unwrap().1,
        &genomes.last().unwrap().1
    );

    {
        let intermediate = neat::NNTSerde::from(&genomes.first().unwrap().0.network);
        let serialized = serde_json::to_string(&intermediate).unwrap();
        std::fs::File::create("./nnt.json")
            .unwrap()
            .write_all(serialized.as_bytes())
            .unwrap();
        println!("\n{}", serialized);
    }

    debug!(
        "Stopping loop. The training server ran {}",
        time::format(stopwatch.read(), 3)
    );
}
