#[macro_use]
extern crate log;
extern crate simplelog;

mod backend;
mod terminal;

use clap::Parser;
use simplelog::*;
use std::{fs::File, str::FromStr};

// See https://docs.rs/clap/latest/clap/_derive/index.html
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

    /// Enable special developer commands
    #[arg(long)]
    wizard: bool,
}

fn main() {
    let args = Args::parse();
    let log_level = LevelFilter::from_str(&args.log_level).expect("bad log-level");
    let location = LevelFilter::from_str(&args.log_location).expect("bad log-location");

    // See https://docs.rs/simplelog/0.12.0/simplelog/struct.ConfigBuilder.html
    // TODO: may want to support allow and ignore lists. Note that the functions (eg
    // add_filter_allow_str) append onto an internal list.
    let config = ConfigBuilder::new()
        .set_location_level(location) // file names and line numbers
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .build();
    // Unwrapping File::create is a little lame but it actually returns a decent error message.
    let _ = WriteLogger::init(log_level, config, File::create(&args.log_path).unwrap()).unwrap();

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    if args.wizard {
        terminal::WIZARD_MODE.with(|w| {
            *w.borrow_mut() = true;
        })
    }

    let game = backend::Game::new();
    let mut terminal = terminal::Terminal::new(game);
    terminal.run();
}
