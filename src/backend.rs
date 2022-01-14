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

pub use message::{Message, Topic};
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

use cell::Cell;
use event::Event;
use level::Level;
use object::{Liquid, Material, Object, Tag};
use old_pov::OldPoV;
use pov::PoV;

const MAX_MESSAGES: usize = 1000;

pub enum Tile {
    /// player can see this
    Visible { bg: Color, fg: Color, symbol: char },
    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(char),
    /// player has never seen this location
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
        }
    }

    pub fn start(&mut self) {
        let width = 200;
        let height = 60;

        self.post(Event::NewGame);
        self.post(Event::AddMessage(Message {
            topic: Topic::NonGamePlay,
            text: String::from("Welcome to 1k-deaths!"),
        }));

        self.post(Event::NewLevel);

        // Terrain defaults to ground
        for y in 0..height {
            for x in 0..width {
                // TODO: may want a SetAllTerrain variant to avoid a zillion events
                // TODO: or have NewLevel take a default terrain
                self.post(Event::AddObject(Point::new(x, y), make::dirt()));
            }
        }

        // Walls along the edges
        for y in 0..height {
            self.post(Event::ChangeObject(
                Point::new(0, y),
                Tag::Terrain,
                make::stone_wall(),
            ));
            self.post(Event::ChangeObject(
                Point::new(width - 1, y),
                Tag::Terrain,
                make::stone_wall(),
            ));
        }
        for x in 0..width {
            self.post(Event::ChangeObject(
                Point::new(x, 0),
                Tag::Terrain,
                make::stone_wall(),
            ));
            self.post(Event::ChangeObject(
                Point::new(x, height - 1),
                Tag::Terrain,
                make::stone_wall(),
            ));
        }

        // Small lake
        self.post(Event::ChangeObject(
            Point::new(29, 20),
            Tag::Terrain,
            make::deep_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(30, 20),
            Tag::Terrain,
            make::deep_water(),
        )); // lake center
        self.post(Event::ChangeObject(
            Point::new(31, 20),
            Tag::Terrain,
            make::deep_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(30, 19),
            Tag::Terrain,
            make::deep_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(30, 21),
            Tag::Terrain,
            make::deep_water(),
        ));

        self.post(Event::ChangeObject(
            Point::new(29, 19),
            Tag::Terrain,
            make::shallow_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(31, 19),
            Tag::Terrain,
            make::shallow_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(29, 21),
            Tag::Terrain,
            make::shallow_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(31, 21),
            Tag::Terrain,
            make::shallow_water(),
        ));

        self.post(Event::ChangeObject(
            Point::new(28, 20),
            Tag::Terrain,
            make::shallow_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(32, 20),
            Tag::Terrain,
            make::shallow_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(30, 18),
            Tag::Terrain,
            make::shallow_water(),
        ));
        self.post(Event::ChangeObject(
            Point::new(30, 22),
            Tag::Terrain,
            make::shallow_water(),
        ));

        // Large room
        let room_loc = Point::new(100, 20);
        let room_height = 30;
        let room_width = 25;
        for y in room_loc.y..(room_loc.y + room_height) {
            self.post(Event::ChangeObject(
                Point::new(room_loc.x, y),
                Tag::Terrain,
                make::stone_wall(),
            ));
            self.post(Event::ChangeObject(
                Point::new(room_loc.x + room_width - 1, y),
                Tag::Terrain,
                make::stone_wall(),
            ));
        }
        for x in room_loc.x..(room_loc.x + room_width) {
            self.post(Event::ChangeObject(
                Point::new(x, room_loc.y),
                Tag::Terrain,
                make::stone_wall(),
            ));
            self.post(Event::ChangeObject(
                Point::new(x, room_loc.y + room_height - 1),
                Tag::Terrain,
                make::stone_wall(),
            ));
        }
        self.post(Event::ChangeObject(
            Point::new(room_loc.x, room_loc.y + room_height / 2),
            Tag::Terrain,
            make::door(),
        ));

        // Now that the terrain is down we can add items and characters.
        self.post(Event::AddObject(Point::new(20, 10), make::player()));
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

    /// Do something with an adjacent cell, this can be move into it, attack
    /// an enemy there, start a dialog with a friendly character, open a door,
    /// etc.
    pub fn probe(&mut self, dx: i32, dy: i32) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        let new_loc = Point::new(self.level.player.x + dx, self.level.player.y + dy);
        if let Some(cell) = self.level.cells.get(&new_loc) {
            match self.probe_cell(cell) {
                Probe::Move(Some(msg)) => {
                    self.post(Event::AddMessage(msg));
                    self.post(Event::PlayerMoved(new_loc));
                }
                Probe::Move(None) => self.post(Event::PlayerMoved(new_loc)),
                Probe::Failed(mesg) => self.post(Event::AddMessage(mesg)),
                Probe::NoOp => {}
            }
        }
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None. This is mutable because state objects like Level merely set
    /// a dirty flag when events are posted and may need to refresh here.
    pub fn tile(&mut self, loc: &Point) -> Tile {
        let tile = if self.pov.visible(&self.level.player, &self.level, loc) {
            if let Some(cell) = self.level.cells.get(loc) {
                let (bg, fg, symbol) = cell.to_bg_fg_symbol();
                Tile::Visible { bg, fg, symbol }
            } else {
                Tile::NotVisible // completely outside the level (tho want to hide this fact from the UI)
            }
        } else {
            match self.old_pov.get(loc) {
                Some(symbol) => Tile::Stale(symbol),
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

        if let Event::AddMessage(message) = event {
            self.messages.push(message);
            while self.messages.len() > MAX_MESSAGES {
                self.messages.remove(0); // TODO: this is an O(N) operation for Vec, may want to switch to circular_queue
            }
        } else {
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

    fn probe_cell(&self, cell: &Cell) -> Probe {
        for obj in cell.iter().rev() {
            let p = self.probe_obj(obj);
            match p {
                Probe::Move(_) => return p,
                Probe::Failed(_) => return p,
                Probe::NoOp => (),
            }
        }
        panic!("Probe found nothing to do");
    }

    fn probe_obj(&self, obj: &Object) -> Probe {
        if obj.character() {
            Probe::Failed(Message::new(Topic::NonGamePlay, "There is somebody there."))
        } else if let Some(open) = obj.door() {
            if open {
                Probe::Move(None)
            } else {
                Probe::Failed(Message::new(Topic::NonGamePlay, "The door is closed."))
            }
        } else if let Some((liquid, deep)) = obj.liquid() {
            match liquid {
                Liquid::Water => {
                    if deep {
                        Probe::Failed(Message::new(Topic::NonGamePlay, "The water is too deep."))
                    } else {
                        Probe::Move(Some(Message::new(
                            Topic::NonGamePlay,
                            "You splash through the water.",
                        )))
                    }
                }
                Liquid::Vitr => Probe::Failed(Message::new(
                    Topic::NonGamePlay,
                    "Do you have a death wish?",
                )),
            }
        } else if obj.wall() {
            Probe::Failed(Message::new(Topic::NonGamePlay, "You bump into the wall."))
        } else if obj.ground() {
            Probe::Move(None)
        } else {
            Probe::NoOp
        }
    }
}

enum Probe {
    Move(Option<Message>),
    Failed(Message),
    NoOp,
    // TODO: attack, etc
}
