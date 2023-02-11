//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod level;
mod objects;
mod old_pov;
mod persistence;
mod player_actions;
mod pov;
mod primitives;
mod store;
mod store_from_str;
mod type_id;
use serde::{Deserialize, Serialize};
use std::fs::File;
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

const MAX_QUEUED_EVENTS: usize = 1_000; // TODO: make this even larger?

/// Represents what the player wants to do next. Most of these will use up the player's
/// remaining time units, but some like (Examine) don't take any time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    // Drop(Oid),
    /// Print descriptions for objects at the cell. Note that any cell can be examined but
    /// cells that are not in the player's PoV will have either an unhelpful description or
    /// a stale description.
    Examine { loc: Point, wizard: bool },
    /// Move the player to empty cells (or attempt to interact with an object at that cell).
    /// dx and dy must be 0, +1, or -1.
    Move { dx: i32, dy: i32 },
    // /// Something other than the player did something.
    // Object,

    // Remove(Oid),

    // Rest,

    // Wear(Oid),

    // // Be sure to add new actions to the end (or saved games will break).
    // WieldMainHand(Oid),
    // WieldOffHand(Oid),
}

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
    stream: Vec<Action>, // used to reconstruct games
    file: Option<File>,  // actions are perodically saved here
}

impl Game {
    fn new(messages: Vec<Message>, _seed: u64, file: Option<File>) -> Game {
        let level = Level::from(include_str!("backend/maps/start.txt"));
        let stream = Vec::new();
        let mut game = Game { stream, file, level };
        for mesg in messages {
            game.add_message(mesg);
        }
        OldPoV::update(&mut game);
        PoV::refresh(&mut game.level);
        game
    }

    /// Start a brand new game and save it to path.
    pub fn new_game(path: &str, seed: u64) -> Game {
        let mut messages = vec![
            Message {
                kind: MessageKind::Important,
                text: String::from("Welcome to 1k-deaths!"),
            },
            Message {
                kind: MessageKind::Important,
                text: String::from("Are you the hero who will destroy the Crippled God's sword?"),
            },
            Message {
                kind: MessageKind::Important,
                text: String::from("Press the '?' key for help."),
            },
        ];
        let file = match persistence::new_game(path, seed) {
            Ok(se) => Some(se),
            Err(err) => {
                messages.push(Message {
                    kind: MessageKind::Error,
                    text: format!("Couldn't open {path} for writing: {err}"),
                });
                None
            }
        };
        Game::new(messages, seed, file)
    }

    /// Load a saved game and return the actions so that they can be replayed.
    pub fn old_game(path: &str, warnings: Vec<String>) -> (Game, Vec<Action>) {
        let mut seed = 1;
        let mut actions = Vec::new();
        let mut messages = Vec::new();

        let mut file = None;
        info!("loading {path}");
        match persistence::load_game(path) {
            Ok((s, a)) => {
                seed = s;
                actions = a;
            }
            Err(err) => {
                info!("loading file had err: {err}");
                messages.push(Message {
                    kind: MessageKind::Error,
                    text: format!("Couldn't open {path} for reading: {err}"),
                });
            }
        };

        if !actions.is_empty() {
            info!("opening {path}");
            file = match persistence::open_game(path) {
                Ok(se) => Some(se),
                Err(err) => {
                    messages.push(Message {
                        kind: MessageKind::Error,
                        text: format!("Couldn't open {path} for appending: {err}"),
                    });
                    None
                }
            };
        }

        messages.extend(warnings.iter().map(|w| Message {
            kind: MessageKind::Error,
            text: w.clone(),
        }));

        if file.is_some() {
            (Game::new(messages, seed, file), actions)
        } else {
            let mut game = Game::new_game(path, seed);
            for mesg in messages {
                game.add_message(mesg);
            }
            (game, Vec::new())
        }
    }

    pub fn replay_action(&mut self, action: Action) {
        // if let Action::Object = action {
        //     self.advance_time(true);
        // } else {
        self.do_player_acted(action, true);
        // }
    }

    pub fn player_loc(&self) -> Point {
        self.level.expect_location(PLAYER_ID)
    }

    pub fn player_acted(&mut self, action: Action) {
        self.do_player_acted(action, false);
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

    pub fn add_message(&mut self, message: Message) {
        self.level.append_message(message);
    }

    /// Wizard mode command
    pub fn dump_state<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write!(writer, "{}", self.level)
        // self.scheduler.dump(writer, self)    // TODO: want an equivalent for this
    }

    fn save_actions(&mut self) {
        if let Some(f) = &mut self.file {
            if let Err(err) = persistence::append_game(f, &self.stream) {
                self.add_message(Message {
                    kind: MessageKind::Error,
                    text: format!("Couldn't save game: {err}"),
                });
            }
        }
        // If we can't save there's not much we can do other than clear. (Still worthwhile
        // appending onto the stream because we may want a wizard command to show the last
        // few events).
        self.stream.clear();
    }

    fn do_player_acted(&mut self, action: Action, replay: bool) {
        trace!("player is doing {action:?}");
        match action {
            Action::Examine { loc: _, wizard: _ } => todo!(),
            Action::Move { dx, dy } => {
                OldPoV::update(self); // TODO: should only do these if something happened
                self.level.bump_player(dx, dy);
                PoV::refresh(&mut self.level);
            }
        }
        if !replay {
            self.stream.push(action);
            if self.stream.len() >= MAX_QUEUED_EVENTS {
                self.save_actions();
            }
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.save_actions();
    }
}
