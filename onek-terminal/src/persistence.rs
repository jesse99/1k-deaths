// ---- Overview -------------------------------------------------------------------------
// There are a couple ways to handle persistence:
//
// 1) The obvious way is to persist the backend state. This would be simple and efficient
// but we'd lose nifty features like replay which would impact debugging (though this
// could be mitigated by saving the backend state plus the next mutate message to execute).
//
// 2) We could persist mutate messages. That would support replay but would not execute
// the query backend logic or much code in the UI (though I'd expect that code to be less
// buggy and what bugs there are would often be visual and hard to spot via a normal
// replay).
//
// 3) We could persist player actions. That way replay would fully exercise the game and
// would allow users to replay their own or potentially other user's games.
//
// For now we're using option three: key strokes are converted to Command's which are
// persisted and then executed. Game loading is just replaying Command's. That might be
// too slow down the road but maybe we can do something like persist the final backend
// state so that replay can be skipped altogether if desired.
//
// Here's an ordered list of the goals we want:
// 1) If the game panics or does something else wrong we should be able to replay it and
// trigger the issue again. In the future we'll want to allow users to submit problematic
// saved games (or auto-cache them once we move to a web model).
// 2) Saved games would be awfully nice for performance measurements.
// 3) It'd be easier to mine player games for interesting statistics, e.g. how often a
// particular item or spell was used.
// 4) We could use saved games for regression testing. Possibly combined with a checksum
// to verify that the resulting state is the same as it was. Not sure how practical this
// actually is...
// 5) Players could replay notable games, e.g. from players who won with a tough character
// or had a really high score.
// TODO: need to implement some of the above
//
// ---- Crate selection ------------------------------------------------------------------
// There are a bunch of different serde backends and I have looked at quite a few but none
// of them are perfect for us. In order of importance we have these goals:
// 1) In so far as possible, old games should be forward compatible with new code. In
// particular we're going to be constantly adding new Enum variants so that cannot break
// old saved games.
// 2) We want to be able to append actions onto existing saved games. There are ways around
// this but they don't seem like good ideas, e.g. we could simply not save until the player
// quits (but there can be a *lot* of actions) or we could save snapshots to individual
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
use super::*;
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
use super::Point;
#[cfg(test)]
use std::fs;

const MAJOR_VERSION: u8 = 2;
const MINOR_VERSION: u8 = 0;

#[derive(Debug, Clone)]
pub struct BadVersionError {
    major: u8,
}

/// Create a brand new saved game at path (overwriting any existing game).
pub fn new_game(path: &str, seed: u64) -> Result<File, Box<dyn Error>> {
    let header = Header::new(seed);
    new_with_header(path, header)
}

/// Append onto an existing game (which must exist).
pub fn open_game(path: &str) -> Result<File, Box<dyn Error>> {
    let file = OpenOptions::new().append(true).open(path)?;
    Ok(file)
}

pub fn append_game(file: &mut File, commands: &[Command]) -> Result<(), Box<dyn Error>> {
    let bytes: Vec<u8> = postcard::to_allocvec(commands)?; // TODO: compress commands?
    write_len(file, bytes.len())?;
    file.write_all(&bytes)?;
    Ok(())
}

// TODO: Would be a lot better to return these a chunk at a time.
pub fn load_game(path: &str) -> Result<(u64, Vec<Command>), Box<dyn Error>> {
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

    let mut commands = Vec::new();
    while let Ok(len) = read_len(&mut file) {
        let mut bytes = vec![0u8; len];
        file.read_exact(&mut bytes)?;
        let mut chunk: Vec<Command> = from_bytes(&bytes)?;
        commands.append(&mut chunk);
    }

    Ok((header.seed, commands))
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
    app_version: String, // from Cargo.toml
    major_version: u8,   // save game version (will change less often than app_version)
    minor_version: u8,
    date: String,
    os: String,
    seed: u64,
}

impl Header {
    fn new(seed: u64) -> Header {
        let local = chrono::Local::now();
        info!("version: {}", env!("CARGO_PKG_VERSION"));
        Header {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            major_version: MAJOR_VERSION,
            minor_version: MINOR_VERSION,
            date: local.to_rfc2822(),
            os: env::consts::OS.to_string(),
            seed,
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "version: {} date: {} os: {}", self.app_version, self.date, self.os)
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
// Loading that could be quite a bit faster than loading and replaying commands. That would
// also isolate us from logic changes that could hose replay.
fn new_with_header(path: &str, header: Header) -> Result<File, Box<dyn Error>> {
    let path = Path::new(path);
    let mut file = File::create(&path)?;

    let bytes: Vec<u8> = postcard::to_allocvec(&header)?;
    write_len(&mut file, bytes.len())?;
    file.write_all(&bytes)?;

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load() {
        // Can we open a game we've saved and read what we wrote?
        let path = format!("/tmp/saved-{}.game", line!()); // tests are run concurrently so we need to ensure paths are unique
        let _ = fs::remove_file(&path);

        let commands1 = vec![Command::Scroll(1), Command::Scroll(2)];
        let commands2 = vec![Command::Bump(1, 2), Command::Bump(2, 3)];

        {
            // save, close
            let mut serializer = new_game(&path, 1).unwrap();
            append_game(&mut serializer, &commands1).unwrap();
            append_game(&mut serializer, &commands2).unwrap();
        }

        // load
        let commands = load_game(&path).unwrap().1;

        assert_eq!(commands.len(), 4);
        assert_eq!(commands[0], commands1[0]);
        assert_eq!(commands[1], commands1[1]);
        assert_eq!(commands[2], commands2[0]);
        assert_eq!(commands[3], commands2[1]);
    }

    #[test]
    fn test_append_old() {
        // Can we append onto a previously saved game?
        let path = format!("/tmp/saved-{}.game", line!());
        let _ = fs::remove_file(&path);

        let commands1 = vec![Command::Scroll(1), Command::Scroll(-1)];
        let commands2 = vec![Command::Bump(1, 2), Command::Bump(2, 3)];
        let commands3 = vec![Command::Bump(20, 30)];

        {
            // save, close
            let mut serializer = new_game(&path, 1).unwrap();
            append_game(&mut serializer, &commands1).unwrap();
            append_game(&mut serializer, &commands2).unwrap();
        }

        {
            // load 1
            let commands = load_game(&path).unwrap().1;

            assert_eq!(commands.len(), 4);
            assert_eq!(commands[0], commands1[0]);
            assert_eq!(commands[1], commands1[1]);
            assert_eq!(commands[2], commands2[0]);
            assert_eq!(commands[3], commands2[1]);
        }

        {
            // open, append, close
            let mut serializer = open_game(&path).unwrap();
            append_game(&mut serializer, &commands3).unwrap();
        }

        // load 2
        let commands = load_game(&path).unwrap().1;

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], commands1[0]);
        assert_eq!(commands[1], commands1[1]);
        assert_eq!(commands[2], commands2[0]);
        assert_eq!(commands[3], commands2[1]);
        assert_eq!(commands[4], commands3[0]);
    }

    #[test]
    fn test_bad_paths() {
        // File in a non-existent directory.
        let path = "/nothing/there/x.y";
        assert!(new_game(path, 1).is_err());
        assert!(open_game(path).is_err());
        assert!(load_game(path).is_err());

        // Write to read-only directory.
        let path = "/user/bad.game";
        assert!(new_game(path, 1).is_err());
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
            let mut header = Header::new(1);
            header.major_version = MAJOR_VERSION - 1;

            let mut serializer = new_with_header(&path, header).unwrap();
            let commands1 = vec![Command::Scroll(3), Command::Scroll(-10)];
            append_game(&mut serializer, &commands1).unwrap();
        }

        let err = load_game(&path).unwrap_err();
        let desc = format!("{err}");
        assert!(desc.contains("Expected file version"));
    }
}
