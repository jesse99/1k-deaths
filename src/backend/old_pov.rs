use super::details::Game2;
use super::event::Event;
use super::level::{Level, Terrain};
use super::point::Point;
use super::pov::PoV;
use fnv::FnvHashMap;

/// Locations that were visible to a character. Note that PoV overrides
/// this so, as an optimization, this may include locations that are actually
/// visible. Current;y this is only used for the Player to render locations
/// that he has seen before.
pub struct OldPoV {
    old: FnvHashMap<Point, Terrain>, // may not match the current Level state
    edition: u32,                    // current PoV edition
}

impl OldPoV {
    pub fn new() -> OldPoV {
        OldPoV {
            old: FnvHashMap::default(),
            edition: 0,
        }
    }

    pub fn posted(&mut self, _game: &Game2, event: &Event) {
        match event {
            Event::NewGame | Event::NewLevel(_) => {
                self.old.clear();
                self.edition = 0;
            }
            _ => (),
        };
    }

    pub fn update(&mut self, level: &Level, pov: &PoV) {
        if pov.edition() != self.edition {
            for loc in pov.locations() {
                if let Some(terrain) = level.terrain.get(loc) {
                    self.old.insert(*loc, *terrain);
                }
            }
            self.edition = pov.edition();
        }
    }

    pub fn get(&self, loc: &Point) -> Option<Terrain> {
        self.old.get(loc).copied()
    }
}
