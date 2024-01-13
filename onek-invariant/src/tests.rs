#[cfg(test)]
use super::invariant::*;
#[cfg(test)]
use onek_types::*;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use std::sync::OnceLock;

// Snapshot tests need to run sequentially because they talk to external processes like
// the state service. Unit tests can be run using one thread via `--test-threads=1` but
// that doesn't seem to work with `cargo insta test`. So we'll use this mutex to serialize
// them.
#[cfg(test)]
static MUTEX: OnceLock<Mutex<i32>> = OnceLock::new();

#[cfg(test)]
trait ToSnapshot {
    fn to_snapshot(&self, state: &StateIO) -> String;
}

#[cfg(test)]
fn terrain_to_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Dirt => ' ',
        Terrain::Wall => '#',
    }
}

#[cfg(test)]
impl ToSnapshot for StateResponse {
    fn to_snapshot(&self, state: &StateIO) -> String {
        match self {
            StateResponse::Map(map) => map.to_snapshot(state),
            _ => panic!("snapshots are not supported for {self}"),
        }
    }
}

#[cfg(test)]
impl ToSnapshot for View {
    fn to_snapshot(&self, _test: &StateIO) -> String {
        let mut result = String::with_capacity(200);
        for y in self.top_left.y..=self.top_left.y + self.size().height {
            for x in self.top_left.x..=self.top_left.x + self.size().width {
                let loc = Point::new(x, y);
                match self.cells.get(&loc) {
                    Some(cell) => {
                        if cell.character.unwrap_or(NULL_ID) == PLAYER_ID {
                            result.push('@');
                        } else {
                            result.push(terrain_to_char(cell.terrain));
                        }
                    }
                    None => result.push(' '),
                }
            }
            result.push('\n');
        }
        // At some point will need to use tx to include details about objects.
        result
    }
}

#[cfg(test)]
impl ToSnapshot for Note {
    fn to_snapshot(&self, _state: &StateIO) -> String {
        let mut result = String::with_capacity(200);
        result.push_str(&format!("[{:?}] {}\n", self.kind, self.text));
        result
    }
}

#[cfg(test)]
struct GameInfo {
    player_loc: Point,
    view: View,
    notes: Vec<Note>,
}

#[cfg(test)]
impl GameInfo {
    fn new(state: &StateIO) -> GameInfo {
        const NUM_NOTES: usize = 8;

        let player_loc = state.get_player_loc();
        let view = state.get_player_view();
        let notes = state.get_notes(NUM_NOTES);
        GameInfo {
            player_loc,
            view,
            notes,
        }
    }
}

#[cfg(test)]
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
        "####\n\
             #@ #\n\
             ####",
    );
    let logic = LogicIO::new();
    logic.bump(PLAYER_ID, Point::new(0, 1));

    invariant(&state);

    // TODO: other tests should use new goo
    let info = GameInfo::new(&state);
    insta::assert_display_snapshot!(info.to_snapshot(&state));
}
