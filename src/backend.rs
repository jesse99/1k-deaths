//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod cell;
mod event;
mod level;
mod make;
mod message;
mod object;
mod old_pov;
mod pov;
mod primitives;
mod tag;

pub use message::{Message, Topic};
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

use cell::Cell;
use derive_more::Display;
use event::Event;
use level::Level;
use object::Object;
use old_pov::OldPoV;
use pov::PoV;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use std::cell::{RefCell, RefMut};
use tag::{Liquid, Material, Tag, Unique};

const MAX_MESSAGES: usize = 1000;

/// This changes the behavior of the movement keys associated with the player.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum ProbeMode {
    /// Move the player to empty cells (or attempt to interact with an object at that cell).
    Moving,
    /// Print descriptions for objects at the cell.
    Examine(Point),
    // TODO: need something like Focus or Target to allow the user to select a cell/character
    // for stuff like ranged attacks
}

pub enum Tile {
    /// player can see this
    Visible {
        bg: Color,
        fg: Color,
        symbol: char,
        focus: bool,
    },
    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale { symbol: char, focus: bool },
    /// player has never seen this location (and it may not exist)
    NotVisible,
}

/// Top-level backend object encapsulating the game state.
pub struct Game {
    // This is the canonical state of the game.
    stream: Vec<Event>,

    // These are synthesized state objects that store state based on the event stream
    // to make it easier to write the backend logic and render the UI. When a new event
    // is added to the stream the posted method is called for each of these.
    messages: Vec<Message>,
    level: Level,
    pov: PoV,
    old_pov: OldPoV,
    mode: ProbeMode,
    rng: RefCell<SmallRng>,
}

mod details {
    /// View into game after posting an event to Level.
    pub struct Game1<'a> {
        pub level: &'a super::Level,
    }

    pub struct Game2<'a> {
        pub level: &'a super::Level,
        pub pov: &'a super::PoV,
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            stream: Vec::new(),
            messages: Vec::new(),
            level: Level::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
            mode: ProbeMode::Moving,

            // TODO:
            // 1) Use a random seed. Be sure to log this and also allow for
            // specifying the seed (probably via a command line option).
            // 2) SmallRng is not guaranteed to be portable so results may
            // not be reproducible between platforms.
            // 3) We're going to have to be able to persist the RNG. rand_pcg
            // supports serde so that would likely work. If not we could
            // create our own simple RNG.
            rng: RefCell::new(SmallRng::seed_from_u64(100)), // TODO: use a random seed (be sure to log it)
        }
    }

    pub fn start(&mut self) {
        self.post(Event::NewGame);
        self.post(Event::AddMessage(Message {
            topic: Topic::NonGamePlay,
            text: String::from("Welcome to 1k-deaths!"),
        }));

        self.post(Event::NewLevel);

        // TODO: may want a SetAllTerrain variant to avoid a zillion events
        // TODO: or have NewLevel take a default terrain
        let map = include_str!("backend/maps/start.txt");
        make::level(self, map);
    }

    // We're using a RefCell to avoid taking too many mutable Game references.
    pub fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    pub fn recent_messages(&self, limit: usize) -> impl Iterator<Item = &Message> {
        let iter = self.messages.iter();
        if limit < self.messages.len() {
            iter.skip(self.messages.len() - limit)
        } else {
            iter.skip(0)
        }
    }

    pub fn player(&self) -> Point {
        self.level.player
    }

    // TODO: When ProbeMode is not Moving should support tab and shift-tab to move
    // to the next "interesting" cell where interesting might just be a cell
    // with a non-player Character.
    pub fn probe_mode(&mut self, mode: ProbeMode) {
        assert_ne!(self.mode, mode);
        self.post(Event::ChangeProbe(mode));
    }

    /// Do something with an adjacent cell, this can be move into it, attack
    /// an enemy there, start a dialog with a friendly character, open a door,
    /// etc.
    pub fn probe(&mut self, dx: i32, dy: i32) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        match self.mode {
            ProbeMode::Moving => {
                let new_loc = Point::new(self.level.player.x + dx, self.level.player.y + dy);
                if let Some(cell) = self.level.cells.get(&new_loc) {
                    if let Some(mesg) = self.impassible_terrain(cell) {
                        self.post(Event::AddMessage(mesg));
                    } else if cell.contains(&Tag::Character) {
                        self.interact_pre_move(&new_loc);
                    } else if cell.contains(&Tag::ClosedDoor) {
                        self.post(Event::ChangeObject(
                            new_loc,
                            Tag::ClosedDoor,
                            make::open_door(),
                        ));
                    } else {
                        self.post(Event::PlayerMoved(new_loc));
                    }
                }
            }
            ProbeMode::Examine(loc) => {
                let new_loc = Point::new(loc.x + dx, loc.y + dy);
                if self.pov.visible(&self.level.player, &self.level, &new_loc) {
                    let cell = self.level.cells.get(&new_loc).unwrap();
                    let descs: Vec<String> = cell
                        .iter()
                        .rev()
                        .map(|obj| obj.description.clone())
                        .collect();
                    let descs = descs.join(", and ");
                    let text = format!("You see {descs}.");
                    self.post(Event::AddMessage(Message {
                        topic: Topic::NonGamePlay,
                        text,
                    }));
                    self.post(Event::ChangeProbe(ProbeMode::Examine(new_loc)));
                } else if self.old_pov.get(&new_loc).is_some() {
                    let text = "You can no longer see there.".to_string();
                    self.post(Event::AddMessage(Message {
                        topic: Topic::NonGamePlay,
                        text,
                    }));
                    self.post(Event::ChangeProbe(ProbeMode::Examine(new_loc)));
                };
            }
        };
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None. This is mutable because state objects like Level merely set
    /// a dirty flag when events are posted and may need to refresh here.
    pub fn tile(&mut self, loc: &Point) -> Tile {
        let focus = match self.mode {
            ProbeMode::Moving => false,
            ProbeMode::Examine(eloc) => *loc == eloc,
        };
        let tile = if self.pov.visible(&self.level.player, &self.level, loc) {
            if let Some(cell) = self.level.cells.get(loc) {
                let (bg, fg, symbol) = cell.to_bg_fg_symbol();
                Tile::Visible {
                    bg,
                    fg,
                    symbol,
                    focus,
                }
            } else {
                Tile::NotVisible // completely outside the level (tho want to hide this fact from the UI)
            }
        } else {
            match self.old_pov.get(loc) {
                Some(symbol) => Tile::Stale { symbol, focus },
                None => Tile::NotVisible, // not visible and never seen
            }
        };

        // Update the old PoV with the current PoV (this is a fast operation
        // if the current PoV hasn't changed). TODO: though this (and level's
        // edition check) will be called a lot. If they show up in profiling
        // we could add some sort of lock-like object to ensure that that is
        // done before the UI starts calling the tile method).
        self.old_pov.update(&self.level, &self.pov);

        tile
    }
}

impl Game {
    fn post(&mut self, event: Event) {
        self.stream.push(event.clone());

        if let Event::PlayerMoved(new_loc) = event {
            self.interact_post_move(&new_loc);
        }

        if !self.handled_game_event(&event) {
            // This is the type state pattern: as events are posted new state
            // objects are updated and upcoming state objects can safely reference
            // them. To enforce this at compile time Game1, Game2, etc objects
            // are used to provide views into Game.
            self.level.posted(&event);

            let game1 = details::Game1 { level: &self.level };
            self.pov.posted(&game1, &event);

            let game2 = details::Game2 {
                level: &self.level,
                pov: &self.pov,
            };
            self.old_pov.posted(&game2, &event);
        }
    }

    fn handled_game_event(&mut self, event: &Event) -> bool {
        match event {
            Event::AddMessage(message) => {
                if let Topic::Error = message.topic {
                    // TODO: do we really want to do this?
                    error!("Logged error '{}'", message.text);
                }
                self.messages.push(message.clone());
                while self.messages.len() > MAX_MESSAGES {
                    self.messages.remove(0); // TODO: this is an O(N) operation for Vec, may want to switch to circular_queue
                }
                true
            }
            Event::ChangeProbe(mode) => {
                self.mode = *mode;
                true
            }
            Event::AddToInventory(loc) => {
                let cell = self.level.cells.get_mut(loc).unwrap();
                let item = cell.remove(&Tag::Portable);
                let name = item.name().unwrap();
                let mesg = Message {
                    topic: Topic::Item,
                    text: format!("You pick up the {name}."),
                };
                self.messages.push(mesg);

                let cell = self.level.cells.get_mut(&self.level.player).unwrap();
                let obj = cell.get_mut(&Tag::Player);
                let inv = obj.inventory_mut().unwrap();
                inv.push(item);
                true
            }
            _ => false,
        }
    }

    // TODO: move these into a uniques module
    fn find_empty_cell(&self, loc: &Point) -> Option<Point> {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx != 0 || dy != 0 {
                    let new_loc = Point::new(loc.x + dx, loc.y + dy);
                    if let Some(cell) = self.level.cells.get(&new_loc) {
                        if !cell.contains(&Tag::Character) {
                            return Some(new_loc);
                        }
                    }
                }
            }
        }
        None
    }

    fn interact_with_doorman(&mut self, loc: &Point) {
        let cell = self.level.cells.get(&self.level.player).unwrap();
        let obj = cell.get(&Tag::Character);
        match obj.inventory() {
            Some(items) if items.iter().any(|obj| obj.description.contains("Doom")) => {
                let mesg = Message::new(Topic::NPCSpeaks, "Ahh, a new champion for the Emperor!");
                self.post(Event::AddMessage(mesg));

                if let Some(new_loc) = self.find_empty_cell(loc) {
                    self.post(Event::NPCMoved(*loc, new_loc));
                }
            }
            _ => {
                let mesg = Message::new(Topic::NPCSpeaks, "You are not worthy.");
                self.post(Event::AddMessage(mesg));
            }
        }
    }

    fn interact_with_rhulad(&mut self, _loc: &Point) {}

    fn interact_pre_move(&mut self, loc: &Point) {
        if let Some(cell) = self.level.cells.get(loc) {
            let obj = cell.get(&Tag::Character);
            match obj.unique() {
                Some(Unique::Doorman) => self.interact_with_doorman(loc),
                Some(Unique::Rhulad) => self.interact_with_rhulad(loc),
                None => (), // Character but not a unique one, TODO: usually will want to attack it
            }
        }
    }

    fn interact_post_move(&mut self, new_loc: &Point) {
        let mut events = Vec::new();
        if let Some(cell) = self.level.cells.get(new_loc) {
            let terrain = cell.terrain();
            if let Some((Liquid::Water, false)) = terrain.liquid() {
                let mesg = Message::new(Topic::NonGamePlay, "You splash through the water.");
                events.push(Event::AddMessage(mesg));
            }
            if cell.contains(&Tag::Sign) {
                let obj = cell.get(&Tag::Sign);
                let text = obj.sign().unwrap();
                let mesg = Message {
                    topic: Topic::NonGamePlay,
                    text: format!("You see a sign {text}."),
                };
                events.push(Event::AddMessage(mesg));
            }
            if cell.contains(&Tag::Portable) {
                events.push(Event::AddToInventory(*new_loc));
            }
        }
        for evt in events {
            self.post(evt);
        }
    }

    fn impassible_terrain(&self, cell: &Cell) -> Option<Message> {
        let obj = cell.terrain();
        if obj.wall() {
            Some(Message::new(Topic::NonGamePlay, "You bump into the wall."))
        } else if obj.door().is_some() {
            // if the door is open then the player will open it once he
            // moves into it. TODO: later we may want a key (aka Binding) check.
            None
        } else if let Some((liquid, deep)) = obj.liquid() {
            match liquid {
                Liquid::Water => {
                    if deep {
                        Some(Message::new(Topic::NonGamePlay, "The water is too deep."))
                    } else {
                        None
                    }
                }
                Liquid::Vitr => Some(Message::new(
                    Topic::NonGamePlay,
                    "Do you have a death wish?",
                )),
            }
        } else {
            None
        }
    }
}
