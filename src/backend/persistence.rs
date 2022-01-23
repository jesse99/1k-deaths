// There are a bunch of different serde backends and I have looked at quite a few but none
// of them are perfect for us. In order of importance we have these goals:
// 1) In so far as possible, old games should be forward compatible with new code. In
// particular we're going to be constantly adding new Enum variants so that cannot break
// old saved games.
// 2) We want to be able to append events onto existing saved games. There are ways around
// this but they don't seem like good ideas, e.g. we could simply not save until the player
// quits (but there can be a *lot* of events) or we could save snapshots to individual
// files (but then we'd have to manage all those lame files).
// 3) Because we're storing the full history of the game we want an efficient format.
// https://github.com/djkoloski/rust_serialization_benchmark has some good info on the
// relative file sizes for different backends, but none of them seem to do really well here.
//
// Here's a breakdown on what I have found with the different backends:
//          size  enums      notes
// postcard 211K  at end     only works with vectors and slices which is annoying and potentially a bit slow
// rmp      942K  anywhere   nice to work with but file sizes are huge (tho they would compress very well)
// bincode        at end
// ciborium       anywhere   Serializer isn't public and into_writer takes the writer so seems really awkward to use
// speedy                    errors trying to use derive macros: could not find `private` in `speedy
//
// rmp does support serialize_seq with a None size which, in theory, would allow us to
// nicely handle append. However we'd have to keep the sequence serializer instance alive
// for the entire game and I was not able to figure out a good way to save it off into a
// field (sticking this into a worker thread is another option but communicating errors
// back and forth seems like a pita). bincode, ciborium, and postcard don't support
// serialize_seq (either they don't support it at all or not with the None option).
//
// borsh, nachricht, prost, and maybe rkyv are also options but, based on the benchmark
// link above they are unlikely to be better than postcard.
use super::Event;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use postcard::from_bytes;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fmt::{self};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Write;
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

fn write_len(file: &mut File, len: usize) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::new();
    bytes.write_u32::<LittleEndian>(len as u32)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn read_len(file: &mut File) -> Result<usize, Box<dyn Error>> {
    let mut bytes = vec![0u8; 4];
    file.read_exact(&mut bytes)?;
    let mut cursor = std::io::Cursor::new(bytes);
    let len = cursor.read_u32::<LittleEndian>()?;
    Ok(len as usize)
}

// TODO: We might also want to save the entire game state (maybe in a separate file).
// Loading that could be quite a bit faster than loading and replaying events. That would
// also isolate us from logic changes that could hose replay.
fn new_with_header(path: &str, header: Header) -> Result<File, Box<dyn Error>> {
    let path = Path::new(path);
    let mut file = File::create(&path)?;

    let bytes: Vec<u8> = postcard::to_stdvec(&header)?;
    write_len(&mut file, bytes.len())?;
    file.write_all(&bytes)?;

    Ok(file)
}

/// Create a brand new saved game at path (overwriting any existing game).
pub fn new_game(path: &str) -> Result<File, Box<dyn Error>> {
    let header = Header::new();
    new_with_header(path, header)
}

/// Append onto an existing game (which must exist).
pub fn open_game(path: &str) -> Result<File, Box<dyn Error>> {
    let file = OpenOptions::new().append(true).open(path)?;
    Ok(file)
}

pub fn append_game(file: &mut File, events: &[Event]) -> Result<(), Box<dyn Error>> {
    let bytes: Vec<u8> = postcard::to_stdvec(events)?; // TODO: compress events?
    write_len(file, bytes.len())?;
    file.write_all(&bytes)?;
    Ok(())
}

pub fn load_game(path: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let path = Path::new(path);
    let mut file = File::open(&path)?;

    let len = read_len(&mut file)?;
    let mut bytes = vec![0u8; len];
    file.read_exact(&mut bytes)?;
    let header: Header = from_bytes(&bytes)?;
    if header.major_version != MAJOR_VERSION {
        return Err(Box::new(BadVersionError {
            major: header.major_version,
        }));
    }
    info!("loaded file, {header}");

    let mut events = Vec::new();
    while let Ok(len) = read_len(&mut file) {
        let mut bytes = vec![0u8; len];
        file.read_exact(&mut bytes)?;
        let mut chunk: Vec<Event> = from_bytes(&bytes)?;
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

        let events1 = vec![Event::NewGame, Event::BeginConstructLevel];
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

        let events1 = vec![Event::NewGame, Event::BeginConstructLevel];
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
            // open, append, close
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
            let events1 = vec![Event::NewGame, Event::BeginConstructLevel];
            append_game(&mut serializer, &events1).unwrap();
        }

        let err = load_game(&path).unwrap_err();
        let desc = format!("{err}");
        assert!(desc.contains("Expected file version"));
    }
}
