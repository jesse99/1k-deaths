use super::details::Game2;
use super::{Event, Level, PoV, Point};
use fnv::FnvHashMap;

/// Locations that were visible to a character. Note that PoV overrides
/// this so, as an optimization, this may include locations that are actually
/// visible. Current;y this is only used for the Player to render locations
/// that he has seen before.
pub struct OldPoV {
    old: FnvHashMap<Point, char>, // may not match the current Level state
    edition: u32,                 // current PoV edition
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
            Event::NewGame | Event::NewLevel => {
                self.old.clear();
                self.edition = 0;
            }
            _ => (),
        };
    }

    pub fn update(&mut self, level: &Level, pov: &PoV) {
        if pov.edition() != self.edition {
            for loc in pov.locations() {
                if let Some(cell) = level.cells.get(loc) {
                    let (_, _, symbol) = cell.to_bg_fg_symbol();
                    self.old.insert(*loc, symbol);
                }
            }
            self.edition = pov.edition();
        }
    }

    pub fn get(&self, loc: &Point) -> Option<char> {
        self.old.get(loc).copied()
    }
}
