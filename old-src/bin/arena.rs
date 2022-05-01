#[macro_use]
extern crate log;
extern crate simplelog;

use chrono::Utc;
use clap::{ArgEnum, Parser};
use one_thousand_deaths::{self};
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;
use std::io::{stdout, Error};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum LoggingLevel {
    // can't use simplelog::Level because it doesn't derive ArgEnum
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = "Simulate combat between various opponents and print results."
)]
struct Args {
    // TODO: might also want invariants
    /// Only run opponents that match the substr, eg "Guard"
    #[clap(long, value_name = "SUBSTR")]
    filter: Option<String>,

    /// Logging verbosity
    #[clap(long, arg_enum, value_name = "NAME", default_value_t = LoggingLevel::Info)]
    log_level: LoggingLevel,

    /// Number of times to simulate each pair of opponents
    #[clap(long, value_name = "N", default_value_t = 100)]
    num_runs: i32,

    /// Base RNG seed
    #[clap(long, value_name = "N", default_value_t = 1)]
    seed: u64,

    /// Print runtime
    #[clap(long)]
    time: bool,
}

fn to_filter(level: LoggingLevel) -> LevelFilter {
    match level {
        LoggingLevel::Error => LevelFilter::Error,
        LoggingLevel::Warn => LevelFilter::Warn,
        LoggingLevel::Info => LevelFilter::Info,
        LoggingLevel::Debug => LevelFilter::Debug,
        LoggingLevel::Trace => LevelFilter::Trace,
    }
}

fn configure_logging(level: LevelFilter) {
    let logging = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();
    let file = File::create("1k-deaths.log").unwrap();
    CombinedLogger::init(vec![WriteLogger::new(level, logging, file)]).unwrap();

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> Result<(), Box<Error>> {
    let options = Args::parse();
    configure_logging(to_filter(options.log_level));

    let writer = &mut stdout();
    let start_time = Utc::now().timestamp_millis();
    one_thousand_deaths::run_arena_matches(writer, options.num_runs, options.seed, options.filter)?;
    let elapsed = (Utc::now().timestamp_millis() - start_time) as f64;
    if options.time {
        println!("took {:.1}s", elapsed / 1000.0);
    }
    Ok(())
}
