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
use std::path::Path;

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
    #[clap(long, value_name = "PATH")]
    load: Option<String>,

    /// fixed random number seed (defaults to random)
    #[clap(long, value_name = "N")]
    seed: Option<u64>,

    /// logging verbosity
    #[clap(long, arg_enum, value_name = "NAME", default_value_t = LoggingLevel::Info)]
    log_level: LoggingLevel,

    /// enable special developer commands
    #[clap(long)]
    wizard: bool,

    /// enable slow debug checks
    #[cfg(debug_assertions)]
    #[clap(long)]
    invariants: bool,
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
        "started up on {} with version {}",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {
    let options = Args::parse();
    configure_logging(to_filter(options.log_level));

    if options.wizard {
        terminal::WIZARD_MODE.with(|w| {
            *w.borrow_mut() = true;
        })
    }

    let mut warnings = Vec::new();
    if options.seed.is_some() && (options.load.is_some() || Path::new("saved.game").is_file()) && !options.new_game {
        // --new-game --load is a bit odd but means start a new game saved to the specified
        // path. But --seed --load without the --new-game is wrong because we need to replay
        // saved games using the original seed (we could reset the seed once we're finished
        // replaying but that's kind of a pain).
        warnings.push("Ignoring --seed (game is beiing replayed so the original seed is being used.)".to_string());
    }

    // TODO: probably need to make --seed and old_game into a warning
    // (can't just set the seed because we'd have to do it after replay finishes)

    // Timestamps are a poor seed but should be fine for our purposes.
    let seed = options.seed.unwrap_or(chrono::Utc::now().timestamp_millis() as u64);
    let (mut game, actions) = match options.load {
        Some(ref path) if options.new_game => (Game::new_game(path, seed), Vec::new()),
        Some(ref path) => Game::old_game(path, warnings),
        None if Path::new("saved.game").is_file() && !options.new_game => Game::old_game("saved.game", warnings),
        None => (Game::new_game("saved.game", seed), Vec::new()),
    };
    {
        #[cfg(debug_assertions)]
        if options.invariants {
            game.set_invariants(true);
        }
    }

    let mut terminal = terminal::Terminal::new(game, actions);
    terminal.run();
}
