#[macro_use]
extern crate log;
extern crate simplelog;

mod commands;
mod help;
mod main_mode;
mod map_view;
mod messages_view;
mod mode;
mod persistence;
mod replay_mode;
mod terminal;
mod termion_utils;
mod text_mode;
mod text_view;
mod window;

use clap::{Parser, ValueEnum};
use commands::*;
use help::*;
use main_mode::*;
use map_view::*;
use messages_view::*;
use mode::*;
use onek_shared::*;
use replay_mode::*;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;
use std::path::Path;
use termion_utils::*;
use text_mode::*;
use text_view::*;
use window::*;

use crate::terminal::Terminal;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LoggingLevel {
    // can't use simplelog::Level because it doesn't derive ValueEnum
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

// TODO: add wizard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)] // TODO: could do better here but terminal support wil go away at some point
struct Args {
    /// Replay a set of player actions for profiling.
    #[clap(long)]
    benchmark: bool,

    /// Path to saved file
    #[clap(long, value_name = "PATH")]
    load: Option<String>,

    /// Logging verbosity
    #[clap(long, value_enum, value_name = "NAME", default_value_t = LoggingLevel::Info)]
    log_level: LoggingLevel,

    /// Path to saved file
    #[clap(long, value_name = "PATH", default_value_t = String::from("terminal.log"))]
    log_path: String,

    /// Ignore any saved files
    #[clap(long)]
    new_game: bool,
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

// Start a brand new game and save it to path.
fn new_game(ipc: &IPC, path: &str, seed: u64) -> Option<File> {
    let mut notes = Vec::new();

    info!("new {path}");
    let file = match persistence::new_game(path, seed) {
        Ok(se) => Some(se),
        Err(err) => {
            notes.push(Note::new(
                NoteKind::Error,
                format!("Couldn't open {path} for writing: {err}"),
            ));
            None
        }
    };

    ipc.send_mutate(StateMutators::NewGame(notes));
    file
}

// Load a saved game and return the actions so that they can be replayed.
fn old_game(ipc: &IPC, path: &str, warnings: Vec<String>) -> (Option<File>, Vec<Command>) {
    let mut seed = 1;
    let mut commands = Vec::new();
    let mut notes = Vec::new();

    let mut file = None;
    info!("loading {path}");
    match persistence::load_game(path) {
        Ok((s, a)) => {
            seed = s;
            commands = a;
        }
        Err(err) => {
            info!("loading file had err: {err}");
            notes.push(Note::new(
                NoteKind::Error,
                format!("Couldn't open {path} for reading: {err}"),
            ));
        }
    };

    if !commands.is_empty() {
        info!("opening {path}");
        file = match persistence::open_game(path) {
            Ok(se) => Some(se),
            Err(err) => {
                notes.push(Note::new(
                    NoteKind::Error,
                    format!("Couldn't open {path} for appending: {err}"),
                ));
                None
            }
        };
    }

    notes.extend(warnings.iter().map(|w| Note::new(NoteKind::Warning, w.clone())));
    ipc.send_mutate(StateMutators::NewGame(notes));

    (file, commands)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Args::parse();
    init_logging(&options);

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let ipc = IPC::new("/tmp/to-terminal");
    if options.benchmark {
        let note = Note::new(NoteKind::Important, "Benchmarking".to_owned());
        ipc.send_mutate(StateMutators::NewGame(vec![note]));

        let mut terminal = Terminal::new(ipc, None, Vec::new());
        terminal.benchmark();
    } else {
        let mut warnings = Vec::new();
        // if options.seed.is_some() && (options.load.is_some() || Path::new("saved.game").is_file()) && !options.new_game
        // {
        //     // --new-game --load is a bit odd but means start a new game saved to the specified
        //     // path. But --seed --load without the --new-game is wrong because we need to replay
        //     // saved games using the original seed (we could reset the seed once we're finished
        //     // replaying but that's kind of a pain).
        //     warnings.push("Ignoring --seed (game is being replayed so the original seed is being used.)".to_string());
        // }

        // TODO: probably need to make --seed and old_game into a warning
        // (can't just set the seed because we'd have to do it after replay finishes)

        // Timestamps are a poor seed but should be fine for our purposes.
        let seed = 1;
        // let seed = options.seed.unwrap_or(chrono::Utc::now().timestamp_millis() as u64);
        let (file, commands) = match options.load {
            Some(ref path) if options.new_game => (new_game(&ipc, path, seed), Vec::new()),
            Some(ref path) => old_game(&ipc, path, warnings),
            None if Path::new("saved.game").is_file() && !options.new_game => old_game(&ipc, "saved.game", warnings),
            None => (new_game(&ipc, "saved.game", seed), Vec::new()),
        };

        // TODO: should we allow the game to be played if there is no file?
        let mut terminal = Terminal::new(ipc, file, commands);
        terminal.run();
    }

    Result::Ok(())
}
