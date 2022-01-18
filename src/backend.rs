//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod cell;
mod event;
mod interactions;
mod level;
mod make;
mod message;
mod object;
mod old_pov;
mod pov;
mod primitives;
mod tag;

pub use event::Event;
pub use message::{Message, Topic};
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

use cell::Cell;
use derive_more::Display;
use interactions::Interactions;
use level::Level;
use object::Object;
use old_pov::OldPoV;
use pov::PoV;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use std::cell::{RefCell, RefMut};
use tag::{Material, Tag};

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

#[derive(Clone, Copy, Debug, Display)]
pub enum State {
    Bumbling,
    KilledRhulad,
    WonGame,
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
    interactions: Interactions,
    pov: PoV,
    old_pov: OldPoV,
    mode: ProbeMode,
    rng: RefCell<SmallRng>,
    state: State,
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
            level: Level::new(make::stone_wall()),
            interactions: Interactions::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
            mode: ProbeMode::Moving,
            state: State::Bumbling,

            // TODO:
            // 1) Use a random seed. Be sure to log this and also allow for
            // specifying the seed (probably via a command line option).
            // 2) SmallRng is not guaranteed to be portable so results may
            // not be reproducible between platforms.
            // 3) We're going to have to be able to persist the RNG. rand_pcg
            // supports serde so that would likely work. If not we could
            // create our own simple RNG.
            rng: RefCell::new(SmallRng::seed_from_u64(100)),
        }
    }

    pub fn start(&self, events: &mut Vec<Event>) {
        events.push(Event::NewGame);
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Welcome to 1k-deaths!"),
        }));
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Are you the hero will will destroy the Crippled God's sword?"),
        }));
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from(
                "Use the arrow keys to move, 'x' to examine squares, and 'q' to quit.",
            ),
        }));
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Use the escape key to stop examining."),
        }));

        events.push(Event::NewLevel);

        // TODO: may want a SetAllTerrain variant to avoid a zillion events
        // TODO: or have NewLevel take a default terrain
        let map = include_str!("backend/maps/start.txt");
        make::level(self, map, events);
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
        self.level.player()
    }

    // TODO: When ProbeMode is not Moving should support tab and shift-tab to move
    // to the next "interesting" cell where interesting might just be a cell
    // with a non-player Character.
    pub fn probe_mode(&self, mode: ProbeMode, events: &mut Vec<Event>) {
        if self.mode != mode {
            events.push(Event::ChangeProbe(mode));
        }
    }

    /// Do something with an adjacent cell, this can be move into it, attack
    /// an enemy there, start a dialog with a friendly character, open a door,
    /// etc.
    pub fn probe(&self, dx: i32, dy: i32, events: &mut Vec<Event>) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        match self.mode {
            ProbeMode::Moving => {
                let player = self.level.player();
                let new_loc = Point::new(player.x + dx, player.y + dy);
                if !self.interact_pre_move(&player, &new_loc, events) {
                    events.push(Event::PlayerMoved(new_loc));
                }
            }
            ProbeMode::Examine(loc) => {
                let new_loc = Point::new(loc.x + dx, loc.y + dy);
                if self.pov.visible(&new_loc) {
                    let cell = self.level.get(&new_loc);
                    let descs: Vec<String> = cell
                        .iter()
                        .rev()
                        .map(|obj| obj.description.clone())
                        .collect();
                    let descs = descs.join(", and ");
                    let text = format!("You see {descs}.");
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                    events.push(Event::ChangeProbe(ProbeMode::Examine(new_loc)));
                } else if self.old_pov.get(&new_loc).is_some() {
                    let text = "You can no longer see there.".to_string();
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                    events.push(Event::ChangeProbe(ProbeMode::Examine(new_loc)));
                };
            }
        };
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None.
    pub fn tile(&self, loc: &Point) -> Tile {
        let focus = match self.mode {
            ProbeMode::Moving => false,
            ProbeMode::Examine(eloc) => *loc == eloc,
        };
        let tile = if self.pov.visible(loc) {
            let cell = self.level.get(loc);
            let (bg, fg, symbol) = cell.to_bg_fg_symbol();
            Tile::Visible {
                bg,
                fg,
                symbol,
                focus,
            }
        } else {
            match self.old_pov.get(loc) {
                Some(symbol) => Tile::Stale { symbol, focus },
                None => Tile::NotVisible, // not visible and never seen
            }
        };

        tile
    }

    // In order to ensure that games are replayable mutation should only happen
    // because of an event. To help ensure that this should be the only public
    // mutable Game method.
    pub fn post(&mut self, events: Vec<Event>) {
        for event in events {
            self.internal_post(event);
        }

        self.old_pov.update(&self.level, &self.pov);
        self.pov.refresh(&self.level.player(), &mut self.level);
    }
}

impl Game {
    // This should only be called by post_events.
    fn internal_post(&mut self, event: Event) {
        debug!("processing {event:?}"); // TODO: may want to nuke this once we start saving games
        self.stream.push(event.clone());

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

        if let Event::PlayerMoved(new_loc) = event {
            // Icky recursion: when we do stuff like move into a square
            // we want to immediately take various actions, like printing
            // "You splash through the water".
            let mut events = Vec::new();
            self.interact_post_move(&new_loc, &mut events);
            for child in events {
                self.internal_post(child);
            }
        }
    }

    // We're using a RefCell to avoid taking too many mutable Game references.
    fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    fn handled_game_event(&mut self, event: &Event) -> bool {
        match event {
            Event::StateChanged(state) => {
                self.state = *state;
                true
            }
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
                let item = {
                    let cell = &mut self.level.get_mut(loc);
                    let item = cell.remove(&Tag::Portable);
                    let name = item.name().unwrap();
                    let mesg = Message {
                        topic: Topic::Normal,
                        text: format!("You pick up the {name}."),
                    };
                    self.messages.push(mesg);
                    item
                };

                let cell = &mut self.level.get_mut(&self.level.player());
                let mut obj = cell.get_mut(&Tag::Player);
                obj.pick_up(item);
                true
            }
            _ => false,
        }
    }

    fn interact_pre_move_with_tag(
        &self,
        tag0: &Tag,
        player_loc: &Point,
        new_loc: &Point,
        events: &mut Vec<Event>,
    ) -> bool {
        let cell1 = self.level.get(new_loc);
        for obj1 in cell1.iter().rev() {
            for tag1 in obj1.iter() {
                if self
                    .interactions
                    .pre_move(tag0, tag1, self, player_loc, new_loc, events)
                {
                    return true;
                }
            }
        }
        false
    }

    // Player attempting to interact with an adjacent cell.
    fn interact_pre_move(
        &self,
        player_loc: &Point,
        new_loc: &Point,
        events: &mut Vec<Event>,
    ) -> bool {
        // First see if an inventory item can interact with the new cell.
        let cell = self.level.get(player_loc);
        let obj = cell.get(&Tag::Player);
        let inv = obj.inventory().unwrap();
        for obj in inv.iter() {
            for tag0 in obj.iter() {
                if self.interact_pre_move_with_tag(tag0, player_loc, new_loc, events) {
                    return true;
                }
            }
        }
        // Failing that see if the player itself can interact with the cell.
        if self.interact_pre_move_with_tag(&Tag::Player, player_loc, new_loc, events) {
            return true;
        }
        false
    }

    // Player interacting with a cell he has just moved into.
    fn interact_post_move(&self, new_loc: &Point, events: &mut Vec<Event>) {
        let cell = self.level.get(new_loc);
        for obj in cell.iter().rev() {
            for tag in obj.iter() {
                self.interactions.post_move(tag, self, new_loc, events)
            }
        }
    }
}
