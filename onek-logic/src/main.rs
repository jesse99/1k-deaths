#[macro_use]
extern crate log;
extern crate simplelog;

use ipmpsc::{Receiver, SharedRingBuffer};
use onek_types::*;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::{fs::File, str::FromStr};

fn player_can_move(state: &StateIO, to: Point) -> Option<Note> {
    let cell = state.get_cell_at(to);
    match cell.terrain {
        Terrain::Dirt => None, // TODO: these should depend on character type (and maybe affects)
        Terrain::ShallowWater => Some(Note::new(
            NoteKind::Environmental,
            "You splash through the water.".to_owned(),
        )),
        Terrain::DeepWater => Some(Note::new(NoteKind::Error, "The water is too deep.".to_owned())),
        Terrain::Wall => Some(Note::new(NoteKind::Error, "You bounce off the wall.".to_owned())),
    }
}

// TODO: should we rule out bumps more than one square from oid?
// TODO: what about stuff like hopping? maybe those are restricted to moves?
fn handle_bump(state: &StateIO, oid: Oid, loc: Point) {
    if oid != PLAYER_ID {
        todo!("non-player movement isn't implemented yet");
    }

    // At some point may want a dispatch table, e.g.
    // (Actor, Action, Obj) -> Handler(actor, obj)

    // If the move resulted in a note then add it to state.
    let note = player_can_move(state, loc);
    match &note {
        None => (),
        Some(note) => state.send_mutate(StateMutators::AddNote(note.clone())),
    }

    // Do the move as long as the note isn't an error note.
    match note {
        Some(Note {
            kind: NoteKind::Error,
            text: _,
        }) => (),
        _ => state.send_mutate(StateMutators::MovePlayer(loc)),
    }
}

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
