extern crate derive_more;
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate serde_derive;

mod backend;
mod terminal;

use backend::Game;
use clap::{ArgEnum, Parser};
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;
use std::path::{Path, PathBuf};

#[repr(usize)]
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
#[clap(author, version, about, long_about = None)] // TODO: could do better here but terminal support wil go away at some point
struct Args {
    /// ignore any saved files
    #[clap(long)]
    new_game: bool,

    /// path to saved file
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    load: Option<PathBuf>,

    /// fixed random number seed (defaults to random)
    #[clap(long, value_name = "N")]
    seed: Option<u32>,

    /// logging verbosity
    #[clap(long, arg_enum, value_name = "NAME", default_value_t = LoggingLevel::Info)]
    log_level: LoggingLevel,

    /// enable special developer commands
    #[clap(long)]
    wizard: bool,
}

fn main() {
    let options = Args::parse();

    let logging = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        logging,
        File::create("1k-deaths.log").unwrap(),
    )])
    .unwrap();
    let local = chrono::Local::now();
    info!(
        "started up on {} with version {}",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let path = "saved.game";
    let (game, events) = if Path::new(path).is_file() && !options.new_game {
        Game::old_game(path)
    } else {
        (Game::new_game(path), Vec::new())
    };

    let mut terminal = terminal::Terminal::new(game, events);
    terminal.run();
}
