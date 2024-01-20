#![cfg(test)]

use super::invariant::*;
use onek_shared::*;
use std::sync::Mutex;
use std::sync::OnceLock;

// Snapshot tests need to run sequentially because they talk to external processes like
// the state service. Unit tests can be run using one thread via `--test-threads=1` but
// that doesn't seem to work with `cargo insta test`. So we'll use this mutex to serialize
// them.
static MUTEX: OnceLock<Mutex<i32>> = OnceLock::new();

trait ToSnapshot {
    fn to_snapshot(&self, state: &StateIO) -> String;
}

fn cell_to_char(cell: &Cell) -> char {
    for obj in cell.iter().rev() {
        if let Some(value) = obj.get("symbol") {
            return value.to_char();
        }
    }
    '?'
}

impl ToSnapshot for StateResponse {
    fn to_snapshot(&self, state: &StateIO) -> String {
        match self {
            StateResponse::Map(map) => map.to_snapshot(state),
            _ => panic!("snapshots are not supported for {self}"),
        }
    }
}

impl ToSnapshot for View {
    fn to_snapshot(&self, _test: &StateIO) -> String {
        let mut result = String::with_capacity(200);
        for y in self.top_left.y..=self.top_left.y + self.size().height {
            for x in self.top_left.x..=self.top_left.x + self.size().width {
                let loc = Point::new(x, y);
                match self.cells.get(&loc) {
                    Some(cell) => result.push(cell_to_char(cell)),
                    None => result.push(' '),
                }
            }
            result.push('\n');
        }
        // At some point will need to use tx to include details about objects.
        result
    }
}

impl ToSnapshot for Note {
    fn to_snapshot(&self, _state: &StateIO) -> String {
        let mut result = String::with_capacity(200);
        result.push_str(&format!("[{:?}] {}\n", self.kind, self.text));
        result
    }
}

struct GameInfo {
    player_loc: Point,
    view: View,
    notes: Vec<Note>,
}

impl GameInfo {
    fn new(state: &StateIO) -> GameInfo {
        const NUM_NOTES: usize = 8;

        let player_loc = state.get_player_loc();
        let view = state.get_player_view();
        let notes = state.get_notes(NUM_NOTES);
        eprintln!("notes: {notes:?}");
        GameInfo {
            player_loc,
            view,
            notes,
        }
    }
}

impl ToSnapshot for GameInfo {
    fn to_snapshot(&self, state: &StateIO) -> String {
        let mut result = String::with_capacity(800);

        result.push_str(&format!("player_loc: {}\n", self.player_loc));
        result.push_str(&format!("view:\n{}\n", self.view.to_snapshot(state)));
        result.push_str("notes:\n");
        for (i, note) in self.notes.iter().enumerate() {
            let s = note.to_snapshot(state);
            result.push_str(&format!("{i}) {s}"));
        }
        result
    }
}

#[test]
fn test_from_str() {
    let _guard = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let state = StateIO::new("/tmp/state-to-test");
    state.reset(
        "test_from_str",
        "###\n\
             #@#\n\
             ###",
    );

    invariant(&state);

    let view = state.get_player_view();
    insta::assert_display_snapshot!(view.to_snapshot(&state));
}

#[test]
fn test_bump_move() {
    let _guard = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let state = StateIO::new("/tmp/state-to-test");
    state.reset(
        "test_bump_move",
        "####\n\
             #@ #\n\
             ####",
    );
    let logic = LogicIO::new();
    logic.bump(PLAYER_ID, Point::new(2, 1));

    invariant(&state);

    let view = state.get_player_view();
    insta::assert_display_snapshot!(view.to_snapshot(&state));
}

#[test]
fn test_bump_wall() {
    let _guard = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let state = StateIO::new("/tmp/state-to-test");
    state.reset(
        "test_bump_wall",
        "####\n\
             #@ #\n\
             ####",
    );
    let logic = LogicIO::new();
    logic.bump(PLAYER_ID, Point::new(0, 1));

    invariant(&state);

    let info = GameInfo::new(&state);
    insta::assert_display_snapshot!(info.to_snapshot(&state));
}

#[test]
fn test_bump_shallow() {
    let _guard = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let state = StateIO::new("/tmp/state-to-test");
    state.reset(
        "test_bump_shallow",
        "####\n\
             #@~#\n\
             ####",
    );
    let logic = LogicIO::new();
    logic.bump(PLAYER_ID, Point::new(2, 1));

    invariant(&state);

    let info = GameInfo::new(&state);
    insta::assert_display_snapshot!(info.to_snapshot(&state));
}

#[test]
fn test_bump_deep() {
    let _guard = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let state = StateIO::new("/tmp/state-to-test");
    state.reset(
        "test_bump_deep",
        "####\n\
             #@W#\n\
             ####",
    );
    let logic = LogicIO::new();
    logic.bump(PLAYER_ID, Point::new(2, 1));

    invariant(&state);

    let info = GameInfo::new(&state);
    insta::assert_display_snapshot!(info.to_snapshot(&state));
}

// There are LOS unit tests so we don't need a lot here.
#[test]
fn test_los() {
    let _guard = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let state = StateIO::new("/tmp/state-to-test");
    state.reset(
        "test_los",
        "############\n\
             #          #\n\
             #   @   #  #\n\
             #   #      #\n\
             ############",
    );
    let _ = LogicIO::new();
    invariant(&state);

    let info = GameInfo::new(&state);
    insta::assert_display_snapshot!(info.to_snapshot(&state));
}
