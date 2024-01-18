use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NoteKind {
    /// Something happened in the world potentially affecting the player, e.g. heard a
    /// noise outside his LOS.
    Environmental,

    /// Player can't do some action, e.g. walking into a wall.
    Error,
}

/// These are in-game messages for the player, e.g. combat results or status messages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub text: String,
    pub kind: NoteKind,
}

impl Note {
    pub fn new(kind: NoteKind, text: String) -> Note {
        Note { text: text, kind }
    }
}

/// First Object will be terrain.
pub type Cell = Vec<Object>;

/// Represents a portion of a level. Typically cells visible to a character. Note that
/// cells that were previously visible are returned in truncated form: they only include
/// "description", "symbol", "color", "back_color" fields plus "id" which is set to
/// "stale". This is because those cells are now outside the player's LOS and cannot be
/// interacted with and may not even exist now.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub cells: HashMap<Point, Cell>,

    /// top_left and bottom_right are the smallest rectangle enclosing all the locations.
    pub top_left: Point,
    pub bottom_right: Point,
}

impl View {
    pub fn new() -> View {
        View {
            cells: HashMap::new(),
            top_left: Point::origin(),
            bottom_right: Point::origin(),
        }
    }

    pub fn insert(&mut self, loc: Point, cell: Cell) {
        if self.cells.is_empty() {
            self.top_left = loc;
            self.bottom_right = loc;
        } else {
            if loc.x < self.top_left.x {
                self.top_left.x = loc.x;
            }
            if loc.y < self.top_left.y {
                self.top_left.y = loc.y;
            }

            if loc.x > self.bottom_right.x {
                self.bottom_right.x = loc.x;
            }
            if loc.y > self.bottom_right.y {
                self.bottom_right.y = loc.y;
            }
        }

        self.cells.insert(loc, cell);
    }

    pub fn size(&self) -> Size {
        self.bottom_right - self.top_left
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateQueries {
    CellAt(Point),
    Notes(usize),
    PlayerView(),
    PlayerLoc(),
}

/// These update internal state and then send a StateResponse.Updated message to services
/// that used RegisterForUpdate. These return StateResponse.Mutated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateMutators {
    /// Read transactions allow services to get a consistent view of state: any mutation
    /// requests that come in during the transaction are deferred until the transaction
    /// ends. The String argument is an ID used to match up the begin and end of transactions.
    BeginReadTransaction(String),
    EndReadTransaction(String),

    AddNote(Note),
    MovePlayer(Point),
    Reset(String, String), // reason, map could include an arg to map weird chars to some sort of object enum
}

/// Messages that the state service receives.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateMessages {
    /// These do not send a reply.
    Mutate(StateMutators),

    /// Reply is sent to ChannelName.
    Query(ChannelName, StateQueries),

    /// Registers a channel to be used for Query replies.
    RegisterForQuery(ChannelName),

    /// Registers a channel to be used for push notifications on mutations.
    RegisterForUpdate(ChannelName),
}

/// Messages that the state service sends to other services.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateResponse {
    // TODO: ready to move should include all objects that are ready
    Cell(Cell),
    Location(Point),
    Map(View),
    Notes(Vec<Note>),
    Updated(EditCount),
}

mod display_impl {
    use super::*;

    impl fmt::Display for StateMessages {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl fmt::Display for StateResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view() {
        #[rustfmt::skip]
        let mut view = View::new();

        let value: ron::Value = ron::from_str("Terrain(id: \"dirt\")").unwrap();
        let cell: Cell = value.into_rust().unwrap();

        view.insert(Point::new(10, 10), cell.clone());
        assert_eq!(view.top_left, Point::new(10, 10));
        assert_eq!(view.bottom_right, Point::new(10, 10));

        view.insert(Point::new(15, 15), cell.clone());
        assert_eq!(view.top_left, Point::new(10, 10));
        assert_eq!(view.bottom_right, Point::new(15, 15));

        view.insert(Point::new(5, 12), cell.clone());
        assert_eq!(view.top_left, Point::new(5, 10));
        assert_eq!(view.bottom_right, Point::new(15, 15));

        view.insert(Point::new(12, 20), cell.clone());
        assert_eq!(view.top_left, Point::new(5, 10));
        assert_eq!(view.bottom_right, Point::new(15, 20));
    }
}
