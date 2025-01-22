#[macro_use]
extern crate log;
extern crate simplelog;

mod console;
mod game;
mod shared;

use clap::{Parser, ValueEnum};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;
use std::io;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LoggingLevel {
    // can't use simplelog::Level because it doesn't derive ValueEnum
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

// TODO: add seed
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)] // TODO: could do better here but terminal support wil go away at some point
struct Args {
    /// Enable slow debug checks
    #[cfg(debug_assertions)]
    #[clap(long)]
    invariants: bool,

    /// Logging verbosity
    #[clap(long, value_enum, value_name = "NAME", default_value_t = LoggingLevel::Info)]
    log_level: LoggingLevel,

    /// Path to saved file
    #[clap(long, value_name = "PATH", default_value_t = String::from("1k-deaths.log"))]
    log_path: String,
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

fn init_logging(options: &Args) {
    // See https://docs.rs/simplelog/0.12.1/simplelog/struct.ConfigBuilder.html
    // TODO: may want to support allow and ignore lists. Note that the functions (eg
    // add_filter_allow_str) append onto an internal list.
    let location = LevelFilter::Off; // disable logging file and line number
    let log_level = to_filter(options.log_level);
    let config = ConfigBuilder::new()
        .set_location_level(location) // file names and line numbers
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .build();
    // Unwrapping File::create is a little lame but it actually returns a decent error message.
    let _ = WriteLogger::init(log_level, config, File::create(&options.log_path).unwrap()).unwrap();
}

fn main() -> io::Result<()> {
    let options = Args::parse();
    init_logging(&options);

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let game = game::new();
    let mut console = console::new(game);
    console.run()
}
