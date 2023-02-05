//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod level;
mod objects;
mod old_pov;
mod player_actions;
mod pov;
mod primitives;
mod store;
mod store_from_str;
mod type_id;
use std::io::{Error, Write};

use level::*;
// use player_actions::*;
use objects::*;
use old_pov::*;
use pov::*;
use store::*;
use type_id::*;

pub use objects::{Character, Message, MessageKind, Portable, Terrain};
pub use primitives::Point;
pub use primitives::Size;

// use self::relation::Character3;

#[derive(Debug, Eq, PartialEq)]
pub struct Content {
    pub terrain: Terrain,
    pub character: Option<Character>,
    pub portables: Vec<Portable>,
    // TODO: non-portable objects vector, e.g. traps or fountains
}

#[derive(Debug, Eq, PartialEq)]
pub enum Tile {
    /// player can see this
    Visible(Content),

    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(Content),

    /// player has never seen this location (and it may not exist)
    NotVisible,
}

/// External API for the game state. Largely acts as a wrapper around the Store.
pub struct Game {
    level: Level,
}

impl Game {
    pub fn new() -> Game {
        let level = Level::from(include_str!("backend/maps/start.txt"));
        let mut game = Game { level };
        game.add_message(Message {
            kind: MessageKind::Important,
            text: String::from("Welcome to 1k-deaths!"),
        });
        game.add_message(Message {
            kind: MessageKind::Important,
            text: String::from("Are you the hero who will destroy the Crippled God's sword?"),
        });
        game.add_message(Message {
            kind: MessageKind::Important,
            text: String::from("Press the '?' key for help."),
        });
        OldPoV::update(&mut game);
        PoV::refresh(&mut game.level);
        game
    }

    pub fn player_loc(&self) -> Point {
        self.level.expect_location(PLAYER_ID)
    }

    /// 1) If the loc is in the level and within the player's FoV then return the current
    /// state of the cell.
    /// 2) If the loc is a cell the player has seen in the past then return what he had
    /// seen (which may not be accurate now).
    /// 3) Otherwise return NotVisible.
    pub fn tile(&self, loc: Point) -> Tile {
        if self.level.pov.visible(&self.level, loc) {
            let terrain = self.level.get_terrain(loc);
            let character = self.level.find_char(loc);
            let portables = self.level.get_portables(loc);
            Tile::Visible(Content {
                terrain,
                character,
                portables,
            })
        } else {
            match self.level.old_pov.get(loc) {
                Some(old) => Tile::Stale(Content {
                    terrain: old.terrain,
                    character: old.character,
                    portables: old.portables.map_or(vec![], |p| vec![p]),
                }),
                None => Tile::NotVisible, // not visible and never seen
            }
        }
    }

    /// Returns the last count messages.
    pub fn messages(&self, count: usize) -> Vec<Message> {
        let num_messages = self.level.store.len::<Message>(GAME_ID);
        let range = if count < num_messages {
            (num_messages - count)..num_messages
        } else {
            0..num_messages
        };
        self.level.store.get_range::<Message>(GAME_ID, range)
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        OldPoV::update(self); // TODO: should only do these if something happened
        self.level.bump_player(dx, dy);
        PoV::refresh(&mut self.level);
    }

    pub fn add_message(&mut self, message: Message) {
        self.level.append_message(message);
    }

    pub fn dump_state<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write!(writer, "{}", self.level)
        // self.scheduler.dump(writer, self)    // TODO: want an equivalent for this
    }
}
