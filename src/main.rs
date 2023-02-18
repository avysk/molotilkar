use std;

use clap::Parser;
use log::debug;
use rand;
use rand_distr::{Distribution, Normal};
use simple_logger;
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Percent(f32);
impl std::ops::Add for Percent {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
impl std::ops::Sub for Percent {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}
impl std::fmt::Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}
impl Percent {
    fn clamp(self) -> Self {
        if self < Percent(0.0) {
            Percent(0.0)
        } else if self > Percent(100.0) {
            Percent(100.0)
        } else {
            self.clone()
        }
    }
}

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

fn f32_representation(s: &str) -> Result<f32, String> {
    let fs = s.trim_end_matches('%');
    let res: f32 = fs
        .parse()
        .map_err(|_| format!("Cannot parse {s} as 32-bit float."))?;
    Ok(res)
}

fn strictly_positive_percent(s: &str) -> Result<Percent, String> {
    let f = f32_representation(s)?;
    if f <= 0.0 || f > 100.0 {
        Err(format!(
            "'{f}' must be strictly positiive and no more than 100.0."
        ))
    } else {
        Ok(Percent(f))
    }
}

fn non_negative_percent(s: &str) -> Result<Percent, String> {
    let f = f32_representation(s)?;
    if f < 0.0 {
        Err(format!("'{f}' cannot be negative."))
    } else {
        Ok(Percent(f))
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// CPU load (in percents) to generate. Must be strictly positiive and no more than 100.0.
    #[arg(value_parser=strictly_positive_percent)]
    load: Percent,
    /// Enable debug logging.
    #[arg(short, long)]
    verbose: bool,
    /// Standard deviation (in percents) of the load. Cannot be negative.
    #[arg(short, long, default_value_t=Percent(0.0), value_parser=non_negative_percent)]
    deviation: Percent,
    /// Adjust system loads by steps, given by this number. Must be strictly positive and no more
    /// than 100.0.
    #[arg(short, long, default_value_t=Percent(4.5), value_parser=strictly_positive_percent)]
    step: Percent,
    /// If system load is withing this number (in percents) from this number, do not adqust. Must
    /// be strictly positive and no more than 100.0.
    #[arg(short, long, default_value_t=Percent(1.0), value_parser=strictly_positive_percent)]
    epsilon: Percent,
    /// Do not change the system load during this time (in milliseconds). Must be strictly
    /// positive. Values below 500 will negatively affect precision and are not recommended.
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
    let normal = Normal::new(cli.load.0, cli.deviation.0)
        .expect("Failed to create normal distribution source");
    let normal_percent = || Percent(normal.sample(&mut rand::thread_rng())).clamp();
    loop {
        let willing = normal_percent();
        debug!("Willing to get {willing} load.");
        loop {
            debug!("Sleeping for {} milliseconds.", cli.time_period);
            std::thread::sleep(std::time::Duration::from_millis(cli.time_period));
            sys.refresh_cpu();
            let current_load = Percent(sys.global_cpu_info().cpu_usage());
            debug!("Current system load is {current_load}.");
            if willing - cli.epsilon <= current_load && current_load <= willing + cli.epsilon {
                debug!("Desired load reached.");
                break;
            }
        }
    }
}
