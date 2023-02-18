use clap::Parser;
use log::debug;
use simple_logger;
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

fn init_logger(verbose: bool) {
    if verbose {
        let _ = simple_logger::init_with_level(log::Level::Debug)
            .expect("Logger initialization failed");
        debug!("Verbose logging is enabled.");
    } else {
        let _ = simple_logger::init().expect("Logger initialization failed.");
    }
}

fn f32_representation(s: &str) -> Result<f32, String> {
    let res: f32 = s
        .parse()
        .map_err(|_| format!("Cannot parse {s} as 32-bit float."))?;
    Ok(res)
}

fn strictly_positive_percent(s: &str) -> Result<f32, String> {
    let f = f32_representation(s)?;
    if f <= 0.0 || f > 100.0 {
        Err(format!(
            "'{f}' must be strictly positiive and no more than 100.0."
        ))
    } else {
        Ok(f)
    }
}

fn non_negative(s: &str) -> Result<f32, String> {
    let f = f32_representation(s)?;
    if f < 0.0 {
        Err(format!("'{f}' cannot be negative."))
    } else {
        Ok(f)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// CPU load (in percents) to generate. Must be strictly positiive and no more than 100.0.
    #[arg(value_parser=strictly_positive_percent)]
    load: f32,
    /// Enable debug logging.
    #[arg(short, long)]
    verbose: bool,
    /// Standard deviation (in percents) of the load. Cannot be negative.
    #[arg(short, long, default_value_t = 0.0, value_parser=non_negative)]
    deviation: f32,
}

fn main() {
    let cli = Cli::parse();
    init_logger(cli.verbose);
    println!("{}", cli.load);
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    sys.refresh_cpu();
    println!("{}%", sys.global_cpu_info().cpu_usage());
}
