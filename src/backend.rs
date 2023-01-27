//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod facts;
mod player_actions;
mod primitives;
mod relation;
mod store2;
mod store_from_str;
use core::num;
use std::io::{Error, Write};

use facts::*;
// use player_actions::*;
use store2::*;

pub use facts::{Character, Message, MessageKind, Portable, Terrain};
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
        let terrain = self.level.get_terrain(loc);
        let character = self.level.find_char(loc);
        let portables = self.level.get_portables(loc);
        Tile::Visible(Content {
            terrain,
            character,
            portables,
        })
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
        self.level.bump_player(dx, dy);
    }

    pub fn add_message(&mut self, message: Message) {
        self.level.append_message(message);
    }

    pub fn dump_state<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.dump_pov(writer)
        // self.scheduler.dump(writer, self)    // TODO: want an equivalent for this
    }

    // TODO: use POV
    fn dump_pov<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut details = Vec::new();

        let center = self.player_loc();
        for y in center.y - 8..center.y + 8 {
            for x in center.x - 8..center.x + 8 {
                let loc = Point::new(x, y);
                if self.level.num_objects(loc) > 0 {
                    let cp = (48 + details.len()) as u8;
                    write!(writer, "{}", (cp as char))?;
                    details.push(loc);
                } else {
                    self.dump_terrain(writer, loc)?;
                }
            }
            write!(writer, "\n")?;
        }
        write!(writer, "\n")?;

        for (i, loc) in details.iter().enumerate() {
            let cp = (48 + i) as u8;
            write!(writer, "{} at {loc}\n", (cp as char))?;

            let portables = self.level.get_portables(*loc);
            for p in portables.iter() {
                write!(writer, "   {p}\n")?;
            }

            if let Some(c) = self.level.find_char(*loc) {
                write!(writer, "   {c}\n")?; // TODO: can dump a lot more here
            }
        }
        Result::Ok(())
    }

    fn dump_terrain<W: Write>(&self, writer: &mut W, loc: Point) -> Result<(), Error> {
        let terrain = self.level.get_terrain(loc);
        match terrain {
            Terrain::ClosedDoor => write!(writer, "+")?,
            Terrain::DeepWater => write!(writer, "W")?,
            Terrain::Dirt => write!(writer, ".")?,
            Terrain::OpenDoor => write!(writer, "-")?,
            Terrain::Rubble => write!(writer, "…")?,
            Terrain::ShallowWater => write!(writer, "w")?,
            Terrain::Tree => write!(writer, "T")?,
            Terrain::Vitr => write!(writer, "V")?,
            Terrain::Wall => write!(writer, "#")?,
        }
        Result::Ok(())
    }
}
