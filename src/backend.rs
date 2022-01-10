//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod color;
mod event;
mod fov;
mod level;
mod old_pov;
mod point;
mod pov;
mod size;
mod vec2d;

pub use color::Color;
pub use level::Terrain;
pub use point::Point;
pub use size::Size;

use event::Event;
use level::Level;
use old_pov::OldPoV;
use pov::PoV;

pub enum Tile {
    /// player can see this
    Visible(Terrain),
    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(Terrain),
    /// player has never seen this location
    NotVisible,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Topic {
    // /// An operation could not be completed.
    // Error,
    /// Something that doesn't affect the game, e.g. bumping into a wall.
    NonGamePlay,
    // /// NPC was damaged (but not by the player).
    // NpcIsDamaged, // TODO: might want to have a separate Topic for player allies

    // /// NPC was attacked but not damaged (but not by the player).
    // NpcIsNotDamaged,

    // /// The player has caused damage.
    // PlayerDidDamage,

    // /// The player attacked but did no damage.
    // PlayerDidNoDamage,

    // /// The player has taken damage.
    // PlayerIsDamaged,

    // /// The player was attacked but took no damage.
    // PlayerIsNotDamaged,

    // /// The player will operate less well.
    // PlayerIsImpaired, // TODO: probably also want a PlayerEnchanced

    // /// The player is at risk of taking damage.
    // PlayerIsThreatened,

    // /// An operation was not completely successful.
    // Warning,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Message {
    pub topic: Topic,
    pub text: String,
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
        self.post(Event::NewLevel(Size::new(width, height)));
        self.post(Event::PlayerMoved(Point::new(20, 10)));

        self.post(Event::AddMessage(Message {
            topic: Topic::NonGamePlay,
            text: String::from("Welcome to 1k-deaths!"),
        }));

        // Terrain defaults to ground
        for y in 0..height {
            for x in 0..width {
                // TODO: may want a SetAllTerrain variant to avoid a zillion events
                // TODO: or have NewLevel take a default terrain
                self.post(Event::SetTerrain(Point::new(x, y), Terrain::Ground));
            }
        }

        // Walls along the edges
        for y in 0..height {
            self.post(Event::SetTerrain(Point::new(0, y), Terrain::Wall));
            self.post(Event::SetTerrain(Point::new(width - 1, y), Terrain::Wall));
        }
        for x in 0..width {
            self.post(Event::SetTerrain(Point::new(x, 0), Terrain::Wall));
            self.post(Event::SetTerrain(Point::new(x, height - 1), Terrain::Wall));
        }

        // Small lake
        self.post(Event::SetTerrain(Point::new(29, 20), Terrain::DeepWater));
        self.post(Event::SetTerrain(Point::new(30, 20), Terrain::DeepWater)); // lake center
        self.post(Event::SetTerrain(Point::new(31, 20), Terrain::DeepWater));
        self.post(Event::SetTerrain(Point::new(30, 19), Terrain::DeepWater));
        self.post(Event::SetTerrain(Point::new(30, 21), Terrain::DeepWater));

        self.post(Event::SetTerrain(Point::new(29, 19), Terrain::ShallowWater));
        self.post(Event::SetTerrain(Point::new(31, 19), Terrain::ShallowWater));
        self.post(Event::SetTerrain(Point::new(29, 21), Terrain::ShallowWater));
        self.post(Event::SetTerrain(Point::new(31, 21), Terrain::ShallowWater));

        self.post(Event::SetTerrain(Point::new(28, 20), Terrain::ShallowWater));
        self.post(Event::SetTerrain(Point::new(32, 20), Terrain::ShallowWater));
        self.post(Event::SetTerrain(Point::new(30, 18), Terrain::ShallowWater));
        self.post(Event::SetTerrain(Point::new(30, 22), Terrain::ShallowWater));

        // Large room
        let room_loc = Point::new(100, 20);
        let room_height = 30;
        let room_width = 25;
        for y in room_loc.y..(room_loc.y + room_height) {
            self.post(Event::SetTerrain(Point::new(room_loc.x, y), Terrain::Wall));
            self.post(Event::SetTerrain(
                Point::new(room_loc.x + room_width - 1, y),
                Terrain::Wall,
            ));
        }
        for x in room_loc.x..(room_loc.x + room_width) {
            self.post(Event::SetTerrain(Point::new(x, room_loc.y), Terrain::Wall));
            self.post(Event::SetTerrain(
                Point::new(x, room_loc.y + room_height - 1),
                Terrain::Wall,
            ));
        }
        self.post(Event::SetTerrain(
            Point::new(room_loc.x, room_loc.y + room_height / 2),
            Terrain::ClosedDoor,
        ));
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

    // TODO: probably want to return something to indicate whether a UI refresh is neccesary
    // TODO: maybe something fine grained, like only need to update messages
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let new_loc = Point::new(self.level.player.x + dx, self.level.player.y + dy);
        if self.level.can_move(dx, dy) {
            self.post(Event::PlayerMoved(new_loc));
        }

        if let Some(message) = self.moved_message(new_loc) {
            self.post(Event::AddMessage(message));
        }
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None. This is mutable because state objects like Level merely set
    /// a dirty flag when events are posted and may need to refresh here.
    pub fn tile(&mut self, loc: &Point) -> Tile {
        let tile = if self.pov.visible(&self.level.player, &self.level, loc) {
            match self.level.terrain.get(loc) {
                Some(terrain) => Tile::Visible(*terrain),
                None => Tile::NotVisible, // completely outside the level (tho want to hide this fact from the UI)
            }
        } else {
            match self.old_pov.get(loc) {
                Some(terrain) => Tile::Stale(terrain),
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

    fn moved_message(&self, new_loc: Point) -> Option<Message> {
        match self.level.terrain.get(&new_loc).unwrap() {
            Terrain::ClosedDoor => None,
            Terrain::DeepWater => Some(Message {
                topic: Topic::NonGamePlay,
                text: String::from("That water is too deep."),
            }),
            Terrain::ShallowWater => Some(Message {
                topic: Topic::NonGamePlay,
                text: String::from("You splash through the water."),
            }),
            Terrain::Wall => Some(Message {
                topic: Topic::NonGamePlay,
                text: String::from("You bump into the wall."),
            }),
            Terrain::Ground => None,
        }
    }

    fn post(&mut self, event: Event) {
        self.stream.push(event.clone());

        if let Event::AddMessage(message) = event {
            self.messages.push(message);
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
}
