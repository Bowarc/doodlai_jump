use neat::*;
use plotters::{
    drawing::IntoDrawingArea as _,
    style::{Color as _, IntoFont as _},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ring::{
    Brain, PerformanceStats, PlottingObserver, GAME_DELTA_TIME, GAME_FPS, GAME_TIME_S,
    MUTATION_PASSES, MUTATION_RATE, NB_GAMES, NB_GENERATIONS, NB_GENOME_PER_GEN,
};
use std::io::Write as _;

#[macro_use]
extern crate log;

mod utils;

fn fitness(brain: &Brain) -> f32 {
    (0..NB_GAMES).map(|_| play_game(&brain)).sum::<f32>() / NB_GAMES as f32
}

fn play_game(brain: &Brain) -> f32 {
    let mut game = game::Game::new();

    let mut saved_score = game.score();
    let mut save_timer = time::DTDelay::new(10.);

    // loop for the number of frames we want to play, should be enough frames to play 100s at 60fps
    // for _ in 0..(GAME_FPS * GAME_TIME_S) {
    while game.score() < 100_000. {
        let output = brain.predict(ring::generate_inputs(&game));

        match output.iter().max_index().unwrap() {
            0 => (), // No action
            1 => game.player_move_left(),
            2 => game.player_move_right(),
            _ => (),
        }

        game.update(GAME_DELTA_TIME);
        save_timer.update(GAME_DELTA_TIME);

        if game.lost {
            // println!("Lost: {}", game.score());
            break;
        }

        if save_timer.ended() {
            if game.score() == saved_score {
                // The player stagnated and needs to be shot (ingame)
                game.lost = true;
                break;
            }
            saved_score = game.score();
            save_timer.restart_custom_timeline(save_timer.time_since_ended());
        }
    }

    game.score()
}

fn sort_genomes(genomes: &[Brain]) -> Vec<(&Brain, f32)> {
    // Iter with rayon

    let mut genomes = genomes
        .par_iter()
        .map(|dna| (dna, fitness(dna)))
        .collect::<Vec<(&Brain, f32)>>();

    genomes.sort_unstable_by_key(|(_dna, fitness)| -fitness as i32);

    genomes
}

fn main() {
    let config = logger::LoggerConfig::default().set_level(log::LevelFilter::Debug);

    logger::init(config, Some("./log/ring.log"));

    let stopwatch = time::Stopwatch::start_new();

    let running = utils::set_up_ctrlc();

    debug!("Starting training server");

    // unsafe {
    //     agent::LOADED_NNT = Some(
    //         serde_json::from_str::<neat::NNTSerde<{ AGENT_IN }, { AGENT_OUT }>>(include_str!(
    //             "./nnt.json"
    //         ))
    //         .unwrap()
    //         .into(),
    //     );
    // };

    let performance_stats =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(NB_GENERATIONS)));
    let observer = PlottingObserver {
        performance_stats: performance_stats.clone(),
    };

    let mut rng = rand::rng();

    let mut sim = GeneticSim::new(
        Vec::gen_random(&mut rng, NB_GENOME_PER_GEN),
        FitnessEliminator::builder()
            .fitness_fn(fitness)
            .observer(observer)
            .build(),
        CrossoverRepopulator::new(
            MUTATION_RATE,
            ReproductionSettings {
                mutation: MutationSettings {
                    mutation_rate: MUTATION_RATE,
                    ..Default::default()
                },
                mutation_passes: MUTATION_PASSES,
            },
        ),
    );

    let pb = indicatif::ProgressBar::new(NB_GENERATIONS as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}], {eta})")
            .expect("Could not create the progress bar")
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Training"));

    let mut all_time_best = 0.;
    let mut actual_generations = NB_GENERATIONS;

    for i in 0..NB_GENERATIONS {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            actual_generations = i + 1;
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
            pb.set_message(format!("Sim {}/{}", i + 1, NB_GENERATIONS,))
        }
    }
    if running.load(std::sync::atomic::Ordering::SeqCst) {
        pb.finish();
    }
    debug!(
        "Stopping loop. The training server ran {}\nSaving data . . .\n",
        time::format(stopwatch.read(), 3)
    );

    let genomes = sort_genomes(&sim.genomes);

    {
        let serialized = serde_json::to_string(&genomes.first().unwrap().0).unwrap();
        std::fs::File::create("./sim/best.json")
            .unwrap()
            .write_all(serialized.as_bytes())
            .unwrap();
    }

    drop(sim);

    let data: Vec<_> = std::sync::Arc::into_inner(performance_stats)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .enumerate()
        .collect();

    let highs = data
        .iter()
        .map(|(i, PerformanceStats { high, .. })| (*i, *high));

    let medians = data
        .iter()
        .map(|(i, PerformanceStats { median, .. })| (*i, *median));

    let lows = data
        .iter()
        .map(|(i, PerformanceStats { low, .. })| (*i, *low));

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
        .build_cartesian_2d(
            0usize..actual_generations,
            0f32..(highs.clone().max_by_key(|(_i, p)| *p as i32).unwrap().1 as f32 * 1.2),
        )
        .unwrap();

    chart.configure_mesh().draw().unwrap();

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
