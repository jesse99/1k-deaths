#[macro_use]
extern crate log;
extern crate simplelog;

use clap::Parser;
use simplelog::*;
use std::{fs::File, str::FromStr};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Can be trace, debug, info, warn, error, or off
    #[arg(long, default_value = "info", value_name = "LEVEL")]
    log_level: String,

    /// Log file and line number at LEVEL and above
    #[arg(long, default_value = "off", value_name = "LEVEL")]
    log_location: String,

    /// Relative path to log file
    #[arg(long, default_value = "1k-deaths.log", value_name = "PATH")]
    log_path: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

// TODO:
// add options to control the logger
//  support allow modules
//  support ignore modules
fn main() {
    let args = Args::parse();
    let log_level = LevelFilter::from_str(&args.log_level).expect("oops"); // TODO: don't use oops
    let location = LevelFilter::from_str(&args.log_location).expect("oops");

    let config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_location_level(location) // include file and line?
        .build();
    let _ = WriteLogger::init(log_level, config, File::create(&args.log_path).unwrap()).unwrap();

    // TODO: at info log timestamp and version, maybe args too
    error!("error level");
    info!("logging to {}", args.log_path);
    debug!("debug level");
}
