mod data;
pub mod player;
mod primitives;
pub mod render;
mod support;

pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;
pub use support::{Message, Topic};

use self::data::*;
use self::support::*;
use derive_more::Display;

#[derive(Clone, Copy, Debug)]
enum Event {
    AddChar(Oid, Point),
    AddItem(Oid, Point),
    Create(ObjectName),
    MoveChar(Oid, Point),
    SetTerrain(Oid, Point),
}

// TODO: These numbers are not very intelligible. If that becomes an issue we could use
// a newtype string (e.g. "wall 97") or a simple struct with a static string ref and a
// counter.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq)]
struct Oid(u64);

/// Encapsulates all the backend game state. All the fields and methods are private so
/// UIs must use the render and input sub-modules.
pub struct State {
    oid_to_obj: OidToObj,
    char_to_loc: CharToLoc,
    messages: Vec<Message>, // messages shown to the player
    stream: Vec<Event>,     // TODO: need to persist this
}

impl State {
    pub fn new() -> State {
        let mut messages = Vec::new();
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Welcome to 1k-deaths!"),
        });
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Are you the hero who will destroy the Crippled God's sword?"),
        });
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Press the '?' key for help."),
        });

        let mut state = State {
            oid_to_obj: OidToObj::new(),
            char_to_loc: CharToLoc::new(),
            messages,
            stream: Vec::new(),
        };

        for y in 0..40 {
            for x in 0..40 {
                state.create_terrain(&Point::new(x, y), ObjectName::Dirt);
            }
        }
        for i in 0..40 {
            state.create_terrain(&Point::new(i, 0), ObjectName::StoneWall);
            state.create_terrain(&Point::new(i, 40), ObjectName::StoneWall);
            state.create_terrain(&Point::new(0, i), ObjectName::StoneWall);
            state.create_terrain(&Point::new(40, i), ObjectName::StoneWall);
        }
        state.create_char(&Point::new(10, 10), ObjectName::Player);
        state
    }
}

impl State {
    fn process(&mut self, event: Event) {
        self.oid_to_obj.process(event);
        self.char_to_loc.process(event);
    }

    fn create_char(&mut self, loc: &Point, name: ObjectName) {
        self.process(Event::Create(name));
        let oid = self.oid_to_obj.last_oid();
        self.process(Event::AddChar(oid, *loc));
    }

    fn create_item(&mut self, loc: &Point, name: ObjectName) {
        self.process(Event::Create(name));
        let oid = self.oid_to_obj.last_oid();
        self.process(Event::AddItem(oid, *loc));
    }

    fn create_terrain(&mut self, loc: &Point, name: ObjectName) {
        self.process(Event::Create(name));
        let oid = self.oid_to_obj.last_oid();
        self.process(Event::SetTerrain(oid, *loc));
    }
}
