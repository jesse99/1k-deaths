#[macro_use]
extern crate log;
extern crate simplelog;

mod bump;

use bump::*;
use ipmpsc::{Receiver, SharedRingBuffer};
use onek_types::*;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::{fs::File, str::FromStr};

fn handle_mesg(state: &StateIO, mesg: LogicMessages) {
    debug!("received {mesg:?}");
    match mesg {
        LogicMessages::Bump(oid, loc) => handle_bump(state, oid, loc),
    }
}

fn init_logging(config: &Config) {
    // See https://docs.rs/simplelog/0.12.1/simplelog/struct.ConfigBuilder.html
    let location = LevelFilter::from_str(&config.str_value("log_location", "off")).expect("bad log_location");
    let log_level = LevelFilter::from_str(&config.str_value("log_level", "info")).expect("bad log_level");
    let log_path = config.str_value("log_path", "logic.log");
    let config = ConfigBuilder::new()
        .set_location_level(location) // file names and line numbers
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .build();
    // Unwrapping File::create is a little lame but it actually returns a decent error message.
    let _ = WriteLogger::init(log_level, config, File::create(&log_path).unwrap()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("onek-logic");
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

    let map_file = "/tmp/logic-sink";
    let rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);

    let state = StateIO::new("/tmp/state-to-logic");
    loop {
        match rx.recv() {
            Ok(mesg) => handle_mesg(&state, mesg),
            Err(err) => {
                error!("rx error: {err}");
                return Result::Err(Box::new(err));
            }
        }
    }
}
