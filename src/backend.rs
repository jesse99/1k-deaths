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

use event::Event;
use level::Level;
use pov::PoV;

// define a Game object with
//    event stream vector
//    level object
//    later other state objects, eg NPCs in view
// when an event is posted notify all the state objects
trait EventPosted {
    fn posted(&mut self, event: Event);
}

pub struct Game {
    // This is the canonical state of the game.
    stream: Vec<Event>,

    // These are synthesized state objects that store state based on the event stream
    // to make it easier to write the backend logic and render the UI. When a new event
    // is posted the posted event for each of these is called.
    level: Level,
    pov: PoV,
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
        self.post(Event::NewLevel { width, height });
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
        self.level.posted(event);
        self.pov.posted(event);
    }
}
