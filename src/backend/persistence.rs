// There are a couple different goals we have for persistence:
// 1) In so far as possible, old games should be forward compatible with new code.
// 2) We need to be able to append new events onto an existing file as the user plays the
// game.
// 3) We're storing the full history of the game so these files could get large. Therefore
// an efficient binary format seems like a good idea.
//
// There are different backends for serde but none of them are exactly what we want:
// 1) The primary issue here is that we will want to add new events without breaking saved
// games, ideally by adding events anywhere in the enum. bincode supports adding new variants
// but only at the end. ciborium and MessagePack (aka rmp) allows them to be added anywhere.
// 2) The serde way to append seems to be to use serialize_seq with a None argument for
// size and then call serialize_element for each new element. bincode doesn't support the
// None argument. ciborium doesn't even have an implementation for this. rmp does support
// it however we'd have to keep the sequence serializer open for the entire game and I was
// not able to figure out a good way to save it off into a field (sticking this into a worker
// thread is another option but communicating errors back and forth seems like a pita).
// 3) All of these binary formats should be fine for performance.
//
// So the main sticking point here is appending events. Currently we simply write a u32
// (u8 might be better) to indicate that events follow and then write out a Vec<Event>.
// Bit lame but simple and should work fine in practice.
use super::Event;
use rmp_serde::decode::Error::{InvalidDataRead, InvalidMarkerRead};
use rmp_serde::Serializer as RmpSerializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fmt::{self};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;

#[cfg(test)]
use super::{Message, Point, Topic};
#[cfg(test)]
use std::fs;

const MAJOR_VERSION: u8 = 1;
const MINOR_VERSION: u8 = 0;

#[derive(Debug, Clone)]
pub struct BadVersionError {
    major: u8,
}

impl fmt::Display for BadVersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Expected file version {} but found version {}",
            MAJOR_VERSION, self.major
        )
    }
}

impl std::error::Error for BadVersionError {}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
struct Header {
    major_version: u8,
    minor_version: u8,
    date: String,
    os: String,
}

impl Header {
    fn new() -> Header {
        let local = chrono::Local::now();
        Header {
            major_version: MAJOR_VERSION,
            minor_version: MINOR_VERSION,
            date: local.to_rfc2822(),
            os: env::consts::OS.to_string(),
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "version: {}.{} date: {} os: {}",
            self.major_version, self.minor_version, self.date, self.os
        )
    }
}

// TODO: We might also want to save the entire game state (maybe in a separate file).
// Loading that could be quite a bit faster than loading and replaying events.
fn new_with_header(path: &str, header: Header) -> Result<RmpSerializer<File>, Box<dyn Error>> {
    let path = Path::new(path);
    let file = File::create(&path)?;

    let mut serializer = rmp_serde::encode::Serializer::new(file);
    header.serialize(&mut serializer)?;

    Ok(serializer)
}

/// Create a brand new saved game at path (overwriting any existing game).
pub fn new_game(path: &str) -> Result<RmpSerializer<File>, Box<dyn Error>> {
    let header = Header::new();
    new_with_header(path, header)
}

/// Append onto an existing game (which must exist).
pub fn open_game(path: &str) -> Result<RmpSerializer<File>, Box<dyn Error>> {
    let file = OpenOptions::new().append(true).open(path)?;
    let serializer = rmp_serde::encode::Serializer::new(file);
    Ok(serializer)
}

pub fn append_game(
    serializer: &mut RmpSerializer<File>,
    events: &Vec<Event>,
) -> Result<(), Box<dyn Error>> {
    // The count indicates that events follow. This is kind of nice because it's difficult
    // to distinguish between eof and a truncated file. It also allows us to do a basic
    // sanity check on load.
    let count = events.len() as u32;
    serializer.serialize_u32(count)?;
    events.serialize(serializer)?;
    Ok(())
}

pub fn load_game(path: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let path = Path::new(path);
    let file = File::open(&path)?;
    let mut de = rmp_serde::decode::Deserializer::new(file);

    let header = Header::deserialize(&mut de)?;
    if header.major_version != MAJOR_VERSION {
        return Err(Box::new(BadVersionError {
            major: header.major_version,
        }));
    }
    info!("loaded file, {header}");

    let mut events = Vec::new();
    loop {
        let len = match u32::deserialize(&mut de) {
            Ok(l) => l,
            // Because we write out the count marker we should never get eof errors.
            Err(InvalidMarkerRead(err)) if matches!(err.kind(), ErrorKind::UnexpectedEof) => break,
            Err(InvalidDataRead(err)) if matches!(err.kind(), ErrorKind::UnexpectedEof) => break,
            Err(err) => return Err(Box::new(err)),
        };

        let mut chunk = Vec::<Event>::deserialize(&mut de)?;
        assert_eq!(len as usize, chunk.len()); // TODO: this is either a corrupted file or a logic error so arguably we should define a new Error for this
        events.append(&mut chunk);
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load() {
        // Can we open a game we've saved and read what we wrote?
        let path = format!("/tmp/saved-{}.game", line!()); // tests are run concurrently so we need to ensure paths are unique
        let _ = fs::remove_file(&path);

        let events1 = vec![Event::NewGame, Event::NewLevel];
        let events2 = vec![
            Event::PlayerMoved(Point::new(1, 2)),
            Event::AddObject(Point::new(2, 3), super::super::make::stone_wall()),
        ];

        {
            // save, close
            let mut serializer = new_game(&path).unwrap();
            append_game(&mut serializer, &events1).unwrap();
            append_game(&mut serializer, &events2).unwrap();
        }

        // load
        let events = load_game(&path).unwrap();

        assert_eq!(events.len(), 4);
        assert_eq!(events[0], events1[0]);
        assert_eq!(events[1], events1[1]);
        assert_eq!(events[2], events2[0]);
        assert_eq!(events[3], events2[1]);
    }

    #[test]
    fn test_append_old() {
        // Can we append onto a previously saved game?
        let path = format!("/tmp/saved-{}.game", line!());
        let _ = fs::remove_file(&path);

        let events1 = vec![Event::NewGame, Event::NewLevel];
        let events2 = vec![
            Event::PlayerMoved(Point::new(1, 2)),
            Event::AddObject(Point::new(2, 3), super::super::make::stone_wall()),
        ];
        let events3 = vec![Event::AddMessage(Message::new(Topic::Normal, "hello"))];

        {
            // save, close
            let mut serializer = new_game(&path).unwrap();
            append_game(&mut serializer, &events1).unwrap();
            append_game(&mut serializer, &events2).unwrap();
        }

        {
            // load 1
            let events = load_game(&path).unwrap();

            assert_eq!(events.len(), 4);
            assert_eq!(events[0], events1[0]);
            assert_eq!(events[1], events1[1]);
            assert_eq!(events[2], events2[0]);
            assert_eq!(events[3], events2[1]);
        }

        {
            // open, close
            let mut serializer = open_game(&path).unwrap();
            append_game(&mut serializer, &events3).unwrap();
        }

        // load 2
        let events = load_game(&path).unwrap();

        assert_eq!(events.len(), 5);
        assert_eq!(events[0], events1[0]);
        assert_eq!(events[1], events1[1]);
        assert_eq!(events[2], events2[0]);
        assert_eq!(events[3], events2[1]);
        assert_eq!(events[4], events3[0]);
    }

    #[test]
    fn test_bad_paths() {
        // File in a non-existent directory.
        let path = "/nothing/there/x.y";
        assert!(new_game(path).is_err());
        assert!(open_game(path).is_err());
        assert!(load_game(path).is_err());

        // Write to read-only directory.
        let path = "/user/bad.game";
        assert!(new_game(path).is_err());
        assert!(open_game(path).is_err());

        // Load of missing file.
        let path = "/tmp/no-file-here";
        assert!(load_game(path).is_err());
    }

    #[test]
    fn test_old_file() {
        // Do we get the proper error trying to read a file that's too old?
        let path = format!("/tmp/saved-{}.game", line!());
        let _ = fs::remove_file(&path);

        {
            let mut header = Header::new();
            header.major_version = MAJOR_VERSION - 1;

            let mut serializer = new_with_header(&path, header).unwrap();
            let events1 = vec![Event::NewGame, Event::NewLevel];
            append_game(&mut serializer, &events1).unwrap();
        }

        let err = load_game(&path).unwrap_err();
        let desc = format!("{err}");
        assert!(desc.contains("Expected file version"));
    }
}
