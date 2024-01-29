#[macro_use]
extern crate log;
extern crate simplelog;

mod help;
mod main_mode;
mod map_view;
mod messages_view;
mod mode;
mod terminal;
mod termion_utils;
mod text_mode;
mod text_view;
mod window;

use help::*;
use main_mode::*;
use map_view::*;
use messages_view::*;
use mode::*;
use onek_shared::*;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::{fs::File, str::FromStr};
use termion_utils::*;
use text_mode::*;
use text_view::*;
use window::*;

use crate::terminal::Terminal;

fn init_logging(config: &Config) {
    // See https://docs.rs/simplelog/0.12.1/simplelog/struct.ConfigBuilder.html
    let location = LevelFilter::from_str(&config.str_value("log_location", "off")).expect("bad log_location");
    let log_level = LevelFilter::from_str(&config.str_value("log_level", "info")).expect("bad log_level");
    let log_path = config.str_value("log_path", "terminal.log");
    let config = ConfigBuilder::new()
        .set_location_level(location) // file names and line numbers
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .build();
    // Unwrapping File::create is a little lame but it actually returns a decent error message.
    let _ = WriteLogger::init(log_level, config, File::create(&log_path).unwrap()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Could use command line to set options instead of a config file. Config file seems
    // a bit nicer when dealing with multiple processes but maybe it'd be better to switch
    // to something like clap later.
    let config = Config::load("onek-terminal");
    init_logging(&config);

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let err = config.error();
    if err.is_some() {
        error!("error loading config: {}", err.as_ref().unwrap());
    }

    let ipc = IPC::new("/tmp/to-terminal");
    ipc.send_mutate(StateMutators::NewLevel("start".to_owned()));
    let mut terminal = Terminal::new(ipc);

    if config.bool_value("benchmark", false) {
        terminal.benchmark();
    } else {
        terminal.run();
    }

    Result::Ok(())
}
