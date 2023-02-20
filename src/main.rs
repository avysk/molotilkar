mod message;
mod molotilka;
mod percent;

use std::cmp::min;
use std::sync::mpsc::sync_channel;

use clap::Parser;
use log::debug;
use rand;
use rand_distr::{Distribution, Normal};
use simple_logger;
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

use percent::Percent;

use crate::message::Message;

fn init_logger(verbose: bool) {
    if verbose {
        let _ = simple_logger::init_with_level(log::Level::Debug)
            .expect("Logger initialization failed");
        debug!("Verbose logging is enabled.");
    } else {
        let _ = simple_logger::init_with_level(log::Level::Info)
            .expect("Logger initialization failed.");
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// CPU load (in percents) to generate. Must be strictly positive.
    #[arg(value_parser=Percent::strictly_positive)]
    load: Percent,
    /// Enable debug logging.
    #[arg(short, long)]
    verbose: bool,
    /// Standard deviation (in percents) of the load.
    #[arg(short, long, default_value_t=Percent(0), value_parser=Percent::valid)]
    deviation: Percent,
    /// Adjust system loads by steps, given by this number (in percents). Must be strictly
    /// positive.
    #[arg(short, long, default_value_t=Percent(4), value_parser=Percent::strictly_positive)]
    step: Percent,
    /// Amount of time (in milliseconds) to sleep after issuing one step adjustment command.
    #[arg(short, long, default_value_t = 10000, value_parser=clap::value_parser!(u64).range(1..))]
    adjust_period: u64,
    /// If system load is withing this number (in percents) from target load, do not adqust.
    #[arg(short, long, default_value_t=Percent(1), value_parser=Percent::valid)]
    epsilon: Percent,
    /// Do not change the system load during this time (in milliseconds). The sum of this value and
    /// adjust-period should be at least 500, or the precision of measurng of system load will be
    /// negatively afected.
    #[arg(short, long, default_value_t=3700, value_parser=clap::value_parser!(u64).range(1..))]
    time_period: u64,
}

fn main() {
    let cli = Cli::parse();
    init_logger(cli.verbose);

    if cli.time_period < 500 {
        log::warn!("Values for time-period less than 500 are not recommended.");
    }

    debug!(
        "Will bring the system load to {0} with standard deviation {1}.",
        cli.load, cli.deviation
    );
    debug!(
        "Changing the load in steps {0}, not adqusting anything during {1}ms periods.",
        cli.step, cli.time_period
    );
    debug!(
        "System load is considered good enough if it is within {} of the target.",
        cli.epsilon
    );
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    let cpus = sys.cpus().len();
    debug!("Found {cpus} CPUs in the system.");
    let mut senders = Vec::new();
    for worker in 1..=cpus {
        let (tx, rx) = sync_channel(1);
        senders.push(tx);
        std::thread::spawn(move || molotilka::start(worker, rx));
    }

    for sender in senders.iter() {
        if sender.send(Message::Start).is_err() {
            panic!("Failed to send Start message to one of the workers");
        } else {
            debug!("Sent Start message to worker.");
        }
    }

    let mut workers_load = Percent(0);

    let normal = Normal::new(f32::from(cli.load.0), f32::from(cli.deviation.0))
        .expect("Failed to create normal distribution source");
    let normal_percent = || Percent::from(normal.sample(&mut rand::thread_rng()));
    loop {
        let willing = normal_percent();
        debug!("Willing to get {willing} load.");
        debug!("sleeping for {} milliseconds.", cli.time_period);
        std::thread::sleep(std::time::Duration::from_millis(cli.time_period));
        loop {
            sys.refresh_cpu();
            let current_load = Percent::from(sys.global_cpu_info().cpu_usage());
            debug!("Current system load is {current_load}.");
            if willing - cli.epsilon <= current_load && current_load <= willing + cli.epsilon {
                debug!(
                    "Desired load reached, we wanted {0}, we got {1}.",
                    willing, current_load
                );
                break;
            }

            if current_load < willing {
                debug!("Load is to small, workers load must be increased.");
                // We do not ask workers to generate more load than we want to get.
                workers_load = min(willing, workers_load + cli.step);
            } else {
                debug!("Load is too high, workers load must be decreased.");
                workers_load = workers_load - cli.step;
            }
            if workers_load == willing {
                debug!("Asking workers to do exactly {} load.", willing);
            }
            let load_msg = Message::Load(workers_load);
            senders.iter().enumerate().for_each(|(num, tx)| {
                debug!("Asking worker {0} to generate load {1}.", num, workers_load);
                if tx.send(load_msg).is_err() {
                    panic!("Worker {} is disconnected.", num);
                }
                debug!("Done.");
            });
            debug!("Sleeping for {} milliseconds.", cli.adjust_period);
            std::thread::sleep(std::time::Duration::from_millis(cli.adjust_period));
        }
    }
}
