#[macro_use]
extern crate log;
mod utils;

fn main() {
    let config = logger::LoggerConfig::default().set_level(log::LevelFilter::Debug);

    logger::init(config, Some("./log/ring.log"));

    let stopwatch = time::Stopwatch::start_new();


    let running = utils::set_up_ctrlc();

    debug!("Starting training server");

    while running.load(std::sync::atomic::Ordering::SeqCst) {

    }

    debug!(
        "Stopping loop. The training server ran {}",
        time::format(stopwatch.read(), 3)
    );
}
