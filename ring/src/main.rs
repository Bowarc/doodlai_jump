use neat::{GenerateRandomCollection, MaxIndex};
use plotters::{
    drawing::IntoDrawingArea as _,
    style::{Color as _, IntoFont as _},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ring::{agent, GAME_DELTA_TIME, GAME_FPS, GAME_TIME_S, NB_GAMES, NB_GENERATIONS, NB_GENOME_PER_GEN};
use std::io::Write as _;

#[macro_use]
extern crate log;

mod utils;

// static mut LOADED_NNT: Option<neat::NeuralNetworkTopology<{ AGENT_IN }, { AGENT_OUT }>> = None;

fn fitness(dna: &agent::DNA) -> f32 {
    let agent = agent::Agent::from(dna);

    (0..NB_GAMES).map(|_| play_game(&agent)).sum::<f32>() / NB_GAMES as f32
}

fn play_game(agent: &agent::Agent) -> f32 {
    let mut game = game::Game::new();

    // loop for the number of frames we want to play, should be enough frames to play 100s at 60fps
    for _ in 0..(GAME_FPS * GAME_TIME_S) {
        let output = agent.network.predict(ring::generate_inputs(&game));

        match output.iter().max_index() {
            0 => (), // No action
            1 => game.player_move_left(),
            2 => game.player_move_right(),
            _ => (),
        }

        game.update(GAME_DELTA_TIME);
        if game.lost {
            break;
        }
    }

    game.score()
}

fn sort_genomes(
    sim: &neat::GeneticSim<
        impl Fn(&agent::DNA) -> f32 + Send + Sync,
        impl neat::NextgenFn<agent::DNA> + Send + Sync,
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

    let performance_stats =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(NB_GENERATIONS)));
    let ng = agent::PlottingNG {
        performance_stats: performance_stats.clone(),
        actual_ng: neat::crossover_pruning_nextgen,
    };

    let mut sim = neat::GeneticSim::new(Vec::gen_random(NB_GENOME_PER_GEN), fitness, ng);

    let pb = indicatif::ProgressBar::new(NB_GENERATIONS as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}], {eta})")
            .expect("Could not create the progress bar")
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Training"));

    let mut all_time_best = 0.;

    for i in 0..NB_GENERATIONS {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        // debug!("Generation {}/{}", i + 1, NB_GENERATIONS,);
        // let t = std::time::Instant::now();

        sim.next_generation();

        // let (sorted_genome, sort_duration) = time::timeit(|| sort_genomes(&sim));

        // let best = sorted_genome.first().unwrap();
        // let second = sorted_genome.get(1).unwrap();
        // let third = sorted_genome.get(2).unwrap();
        // let mid = sorted_genome.get(NB_GENOME_PER_GEN / 2).unwrap();
        // let worst = sorted_genome.last().unwrap();

        // if best.1 > all_time_best{
        //     all_time_best = best.1;
        // }

        // println!(
        //     "Gen {} done, took {}\nResults: {:.0}/{:.0}/{:.0}. sorted in {}.",
        //     i + 1,
        //     time::format(stopwatch.read(), 1),
        //     best.1,
        //     mid.1,
        //     worst.1,
        //     time::format(sort_duration, 1)
        // );

        // {
        //     let mut data = format!(
        //         "Generation: {}/{}\nScores: ({:.0}/{:.0}/{:.0})-{:.0}-{:.0}\n\n",
        //         i + 1,
        //         NB_GENERATIONS,
        //         best.1,
        //         second.1,
        //         third.1,
        //         mid.1,
        //         worst.1
        //     );
        //     data.push_str(
        //         &sim.genomes
        //             .iter()
        //             .map(|dna| format!("{dna:?}\n"))
        //             .collect::<String>(),
        //     );
        //     std::fs::File::create("./sim/DNAbackup.txt".to_string())
        //         .unwrap()
        //         .write_all(data.as_bytes())
        //         .unwrap();

        //     // std::fs::File::create(format!("./sim/{}.best.json", i + 1,))
        //     //     .unwrap()
        //     //     .write_all(
        //     //         serde_json::to_string(&neat::NNTSerde::from(&best.0.network))
        //     //             .unwrap()
        //     //             .as_bytes(),
        //     //     )
        //     //     .unwrap();
        // }

        // for (name, data) in [("best", best), ("mid", mid), ("worst", worst)] {
        //     std::fs::File::create(format!(
        //         "./sim/gen{}_score{:.0}-{}.json",
        //         i + 1,
        //         data.1,
        //         name
        //     ))
        //     .unwrap()
        //     .write_all(
        //         serde_json::to_string(&neat::NNTSerde::from(&data.0.network))
        //             .unwrap()
        //             .as_bytes(),
        //     )
        //     .unwrap();
        // }

        if running.load(std::sync::atomic::Ordering::SeqCst) {
            pb.inc(1);
            // pb.set_message(format!(
            //     "Sim {}/{} [{:.0}/{:.0}/{:.0}] {}/{}",
            //     i + 1,
            //     NB_GENERATIONS,
            //     best.1,
            //     mid.1,
            //     worst.1,
            //     time::format(t.elapsed(), 2),
            //     time::format(sort_duration, 2)
            // ))
            pb.set_message(format!(
                "Sim {}/{}",
                i + 1,
                NB_GENERATIONS,
            ))
        }
    }
    if running.load(std::sync::atomic::Ordering::SeqCst) {
        pb.finish();
    }
    debug!(
        "Stopping loop. The training server ran {}\nSaving data . . .\n",
        time::format(stopwatch.read(), 3)
    );

    let genomes = sort_genomes(&sim);


    {
        let intermediate = neat::NNTSerde::from(&genomes.first().unwrap().0.network);
        let serialized = serde_json::to_string(&intermediate).unwrap();
        std::fs::File::create("./sim/best.json")
            .unwrap()
            .write_all(serialized.as_bytes())
            .unwrap();
    }

    drop(sim);

    let root = plotters::prelude::SVGBackend::new("./sim/fitness-plot.svg", (640, 480))
        .into_drawing_area();
    root.fill(&plotters::prelude::WHITE).unwrap();

    let mut chart = plotters::prelude::ChartBuilder::on(&root)
        .caption(
            "agent fitness values per generation",
            ("sans-serif", 50).into_font(),
        )
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(30)
        // .build_cartesian_2d(0usize..NB_GENERATIONS, 0f32..(all_time_best*1.15))
        .build_cartesian_2d(0usize..NB_GENERATIONS, 0f32..(3000.))
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    let data: Vec<_> = std::sync::Arc::into_inner(performance_stats)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .enumerate()
        .collect();

    let highs = data
        .iter()
        .map(|(i, agent::PerformanceStats { high, .. })| (*i, *high));

    let medians = data
        .iter()
        .map(|(i, agent::PerformanceStats { median, .. })| (*i, *median));

    let lows = data
        .iter()
        .map(|(i, agent::PerformanceStats { low, .. })| (*i, *low));

    chart
        .draw_series(plotters::prelude::LineSeries::new(
            highs,
            &plotters::prelude::GREEN,
        ))
        .unwrap()
        .label("high");

    chart
        .draw_series(plotters::prelude::LineSeries::new(
            medians,
            &plotters::prelude::YELLOW,
        ))
        .unwrap()
        .label("median");

    chart
        .draw_series(plotters::prelude::LineSeries::new(
            lows,
            &plotters::prelude::RED,
        ))
        .unwrap()
        .label("low");

    chart
        .configure_series_labels()
        .background_style(&plotters::prelude::WHITE.mix(0.8))
        .border_style(&plotters::prelude::BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();
}
