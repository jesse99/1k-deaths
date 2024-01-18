#[macro_use]
extern crate log;
extern crate simplelog;

use ipmpsc::{Receiver, Sender, SharedRingBuffer};
use onek_types::*;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::{fs::File, str::FromStr};

mod fov;
mod game;
mod mutators;
mod objects;
mod old_pov;
mod pov;
mod queries;
mod vec2d;

use fov::*;
use game::*;
use mutators::*;
// use objects::*;
use old_pov::*;
use pov::*;
use queries::*;

fn create_sender(name: &ChannelName) -> ipmpsc::Sender {
    match SharedRingBuffer::open(name.as_str()) {
        Ok(buffer) => Sender::new(buffer),
        Err(err) => panic!("error opening sender {name}: {err:?}"),
    }
}

fn handle_mesg(game: &mut Game, mesg: StateMessages) {
    debug!("received {mesg}");
    match mesg {
        StateMessages::Mutate(mesg) => handle_mutate(game, mesg),
        StateMessages::Query(channel_name, mesg) => handle_query(channel_name, game, mesg),
        StateMessages::RegisterForQuery(channel_name) => {
            info!("registering {channel_name} reply sender");
            let sender = create_sender(&channel_name);
            game.reply_senders.insert(channel_name, sender);
        }
        StateMessages::RegisterForUpdate(_channel_name) => println!("RegisterForUpdate isn't implemented yet"),
    }
}

fn init_logging(config: &Config) {
    // See https://docs.rs/simplelog/0.12.1/simplelog/struct.ConfigBuilder.html
    // TODO: may want to support allow and ignore lists. Note that the functions (eg
    // add_filter_allow_str) append onto an internal list.
    let location = LevelFilter::from_str(&config.str_value("log_location", "off")).expect("bad log_location");
    let log_level = LevelFilter::from_str(&config.str_value("log_level", "info")).expect("bad log_level");
    let log_path = config.str_value("log_path", "state.log");
    let config = ConfigBuilder::new()
        .set_location_level(location) // file names and line numbers
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .build();
    // Unwrapping File::create is a little lame but it actually returns a decent error message.
    let _ = WriteLogger::init(log_level, config, File::create(&log_path).unwrap()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("onek-state");
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

    let map_file = "/tmp/state-sink";
    let rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);

    let mut game = Game::new();

    loop {
        match rx.recv() {
            // TODO: do we want zero-copy?
            Ok(mesg) => handle_mesg(&mut game, mesg),
            Err(err) => {
                error!("rx error: {err}");
                return Result::Err(Box::new(err));
            }
        }
        // TODO: panic if a transaction lingers for too long
        // will probably need to add a time snapshot to transaction elements
    }
}
