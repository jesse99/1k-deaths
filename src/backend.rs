//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod cell;
mod event;
mod interactions;
mod level;
mod make;
mod message;
mod object;
mod old_pov;
mod persistence;
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
use std::fs::File;
use std::path::Path;
use tag::{Material, Tag};

const MAX_MESSAGES: usize = 1000;
const MAX_QUEUED_EVENTS: usize = 1_000; // TODO: make this even larger?

#[derive(Clone, Copy, Debug)]
pub enum Command {
    /// Move the player to empty cells (or attempt to interact with an object at that cell).
    /// dx and dy must be 0, +1, or -1.
    Move { dx: i32, dy: i32 },
    /// Print descriptions for objects at the cell. Note that any cell can be examined but
    /// cells that are not in the player's PoV will have either an unhelpful description or
    /// a stale description.
    Examine(Point),
}

pub enum Tile {
    /// player can see this
    Visible { bg: Color, fg: Color, symbol: char },
    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(char),
    /// player has never seen this location (and it may not exist)
    NotVisible,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Serialize, Deserialize)]
pub enum State {
    Bumbling,
    KilledRhulad,
    WonGame,
}

/// Top-level backend object encapsulating the game state.
pub struct Game {
    // This is the canonical state of the game.
    stream: Vec<Event>,
    posting: bool,
    // These are synthesized state objects that store state based on the event stream
    // to make it easier to write the backend logic and render the UI. When a new event
    // is added to the stream the posted method is called for each of these.
    messages: Vec<Message>,
    level: Level,
    interactions: Interactions,
    pov: PoV,
    old_pov: OldPoV,
    rng: RefCell<SmallRng>,
    state: State,
    file: Option<File>,
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
    /// Begins a new game session. If possible this will load an existing game and return
    /// the event stream associated with it for replay'ing. Otherwise a new game will be
    /// started.
    pub fn new() -> (Game, Vec<Event>) {
        let mut events = Vec::new();

        let mut messages = Vec::new();
        let path = "saved.game";
        let mut file = None;
        if Path::new(path).is_file() {
            info!("loading {path}");
            match persistence::load_game(path) {
                Ok(e) => events = e,
                Err(err) => {
                    info!("loading file had err: {err}");
                    messages.push(Message::new(
                        Topic::Error,
                        &format!("Couldn't open {path} for reading: {err}"),
                    ));
                }
            };

            if !events.is_empty() {
                info!("opening {path}");
                file = match persistence::open_game(path) {
                    Ok(se) => Some(se),
                    Err(err) => {
                        messages.push(Message::new(
                            Topic::Error,
                            &format!("Couldn't open {path} for appending: {err}"),
                        ));
                        None
                    }
                };
            }
        }

        // If there is no saved game or there was an error loading it create a file for a
        // brand new game.
        if file.is_none() {
            info!("new {path}");
            file = match persistence::new_game(path) {
                Ok(se) => Some(se),
                Err(err) => {
                    messages.push(Message::new(
                        Topic::Error,
                        &format!("Couldn't open {path} for writing: {err}"),
                    ));
                    None
                }
            };
        }

        (
            Game {
                stream: Vec::new(),
                posting: false,
                messages,
                level: Level::new(make::stone_wall()),
                interactions: Interactions::new(),
                pov: PoV::new(),
                old_pov: OldPoV::new(),
                state: State::Bumbling,
                file,

                // TODO:
                // 1) Use a random seed. Be sure to log this and also allow for
                // specifying the seed (probably via a command line option).
                // 2) SmallRng is not guaranteed to be portable so results may
                // not be reproducible between platforms.
                // 3) We're going to have to be able to persist the RNG. rand_pcg
                // supports serde so that would likely work. If not we could
                // create our own simple RNG.
                rng: RefCell::new(SmallRng::seed_from_u64(100)),
            },
            events,
        )
    }

    /// Should be called if new returned no events.
    pub fn new_game(&self, events: &mut Vec<Event>) {
        events.reserve(1000); // TODO: probably should tune this

        events.push(Event::NewGame);
        events.push(Event::BeginConstructLevel);
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
            text: String::from("Use the arrow keys to move, 'x' to examine squares, and 'q' to quit."),
        }));
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Use the escape key to stop examining."),
        }));

        // TODO: may want a SetAllTerrain variant to avoid a zillion events
        // TODO: or have NewLevel take a default terrain
        let map = include_str!("backend/maps/start.txt");
        make::level(self, map, events);
        events.push(Event::EndConstructLevel);
    }

    pub fn recent_messages(&self, limit: usize) -> impl Iterator<Item = &Message> {
        let iter = self.messages.iter();
        if limit < self.messages.len() {
            iter.skip(self.messages.len() - limit)
        } else {
            iter.skip(0)
        }
    }

    // TODO: this should be wizard config only
    pub fn events(&self) -> Vec<String> {
        self.stream.iter().map(|e| format!("{:?}", e)).collect()
    }

    pub fn player(&self) -> Point {
        self.level.player()
    }

    pub fn command(&self, command: Command, events: &mut Vec<Event>) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        match command {
            Command::Move { dx, dy } => {
                assert!(dx >= -1 && dx <= 1);
                assert!(dy >= -1 && dy <= 1);
                assert!(dx != 0 || dy != 0); // TODO: should this be a short rest?
                let player = self.level.player();
                let new_loc = Point::new(player.x + dx, player.y + dy);
                if !self.interact_pre_move(&player, &new_loc, events) {
                    events.push(Event::PlayerMoved(new_loc));
                }
            }
            Command::Examine(new_loc) => {
                if self.pov.visible(&new_loc) {
                    let cell = self.level.get(&new_loc);
                    let descs: Vec<String> = cell.iter().rev().map(|obj| obj.description.clone()).collect();
                    let descs = descs.join(", and ");
                    let text = format!("You see {descs}.");
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                } else if self.old_pov.get(&new_loc).is_some() {
                    let text = "You can no longer see there.".to_string();
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                };
            }
        }
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None.
    pub fn tile(&self, loc: &Point) -> Tile {
        let tile = if self.pov.visible(loc) {
            let cell = self.level.get(loc);
            let (bg, fg, symbol) = cell.to_bg_fg_symbol();
            Tile::Visible { bg, fg, symbol }
        } else {
            match self.old_pov.get(loc) {
                Some(symbol) => Tile::Stale(symbol),
                None => Tile::NotVisible, // not visible and never seen
            }
        };

        tile
    }

    pub fn target_next(&self, old_loc: &Point, delta: i32) -> Option<Point> {
        // Find the cells with Characters in the player's PoV.
        let mut chars = Vec::new();
        for &loc in self.pov.locations() {
            let cell = self.level.get(&loc);
            if cell.contains(&Tag::Character) {
                chars.push(loc);
            }
        }

        // Sort those cells by distance from the player.
        let p = self.player();
        chars.sort_by_key(|a| a.distance2(&p));

        // Find the Character closest to old_loc.
        let mut index = 0;
        let mut dist = i32::MAX;
        for (i, loc) in chars.iter().enumerate() {
            let d = loc.distance2(old_loc);
            if d < dist {
                index = i;
                dist = d;
            }
        }

        // Find the next Character to examine accounting for lame unsized math.
        if delta > 0 {
            if index < chars.len() && chars[index] != *old_loc {
                // we don't want to apply delta in this case
                assert_eq!(delta, 1);
            } else if index + (delta as usize) < chars.len() {
                index += delta as usize;
            } else {
                index = 0;
            }
        } else if (-delta as usize) <= index {
            index -= -delta as usize;
        } else {
            index = chars.len() - 1;
        }

        if index < chars.len() {
            Some(chars[index])
        } else {
            // We'll only land here in the unusual case of the player not able to see himself.
            None
        }
    }

    // In order to ensure that games are replayable mutation should only happen
    // because of an event. To help ensure that this should be the only public
    // mutable Game method.
    pub fn post(&mut self, events: Vec<Event>, replay: bool) {
        // This is bad because it messes up replay: if it is allowed then an event will
        // post a new event X both of which will be persisted. Then on replay the event
        // will post X but X will have been also saved so X is done twice.
        assert!(!self.posting, "Cannot post an event in response to an event");

        self.posting = true;
        for event in events {
            self.internal_post(event, replay);
        }

        self.old_pov.update(&self.level, &self.pov);
        self.pov.refresh(&self.level.player(), &mut self.level);
        self.posting = false;
    }
}

impl Game {
    // This should only be called by the post method.
    fn internal_post(&mut self, event: Event, replay: bool) {
        // It'd be slicker to use a different Game type when replaying. This would prevent
        // us, at compile time, from touching fields like stream or rng. In practice however
        // this isn't much of an issue because the bulk of the code is already prevented
        // from doing bad things by the Game1, Game2, etc structs.
        if !replay {
            self.stream.push(event.clone());

            if self.stream.len() >= MAX_QUEUED_EVENTS {
                self.append_stream();
            }
        }
        let mut events = Vec::new();
        events.push(event);

        while !events.is_empty() {
            let event = events.remove(0); // icky remove from front but the vector shouldn't be very large...

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
                self.interact_post_move(&new_loc, &mut events);
            }
        }
    }

    fn append_stream(&mut self) {
        if let Some(se) = &mut self.file {
            if let Err(err) = persistence::append_game(se, &self.stream) {
                self.messages
                    .push(Message::new(Topic::Error, &format!("Couldn't save game: {err}")));
            }
        }
        // If we can't save there's not much we can do other than clear. (Still worthwhile
        // appending onto the stream because we may want a wizard command to show the last
        // few events).
        self.stream.clear();
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
    fn interact_pre_move(&self, player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) -> bool {
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

impl Drop for Game {
    fn drop(&mut self) {
        self.append_stream();
    }
}
