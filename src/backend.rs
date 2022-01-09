//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod event;
mod fov;
mod level;
mod point;
mod pov;
mod size;
mod vec2d;

pub use level::Terrain;
pub use point::Point;
pub use size::Size;

use event::Event;
use level::Level;
use pov::PoV;

/// Top-level backend object encapsulating the game state.
pub struct Game {
    // This is the canonical state of the game.
    stream: Vec<Event>,

    // These are synthesized state objects that store state based on the event stream
    // to make it easier to write the backend logic and render the UI. When a new event
    // is added to the stream the posted method is called for each of these.
    level: Level,
    pov: PoV,
}

mod details {
    /// View into game after posting an event to Level.
    pub struct Game1<'a> {
        pub level: &'a super::Level,
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            stream: Vec::new(),
            level: Level::new(),
            pov: PoV::new(),
        }
    }

    pub fn start(&mut self) {
        let width = 200;
        let height = 60;

        self.post(Event::NewGame);
        self.post(Event::NewLevel(Size::new(width, height)));
        self.post(Event::PlayerMoved(Point::new(20, 10)));

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

    pub fn player(&self) -> Point {
        self.level.player
    }

    // TODO: probably want to return something to indicate whether a UI refresh is neccesary
    // TODO: maybe something fine grained, like only update messages
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        if self.level.can_move(dx, dy) {
            let new_loc = Point::new(self.level.player.x + dx, self.level.player.y + dy);
            self.post(Event::PlayerMoved(new_loc));
        }
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None.
    pub fn terrain(&mut self, loc: &Point) -> Option<Terrain> {
        // mutable because state objects may have been invalidated
        if self.pov.visible(&self.level.player, &self.level, loc) {
            self.level.terrain.get(loc).copied()
        } else {
            None
        }
    }

    fn post(&mut self, event: Event) {
        self.stream.push(event);

        // This is the type state pattern: as events are posted new state
        // objects are updated and upcoming state objects can safely reference
        // them. To enforce this at compile time Game1, Game2, etc objects
        // are used to provide views into Game.
        self.level.posted(event);

        let game1 = details::Game1 { level: &self.level };
        self.pov.posted(&game1, event);
    }
}
