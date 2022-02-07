use super::details::Game2;
use super::{Event, Game, Point};
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

    pub fn posting(&mut self, _game: &Game2, event: &Event) {
        match event {
            Event::NewGame | Event::BeginConstructLevel | Event::EndConstructLevel => {
                self.old.clear();
                self.edition = 0;
            }
            _ => (),
        };
    }

    // This can't me an ordinary method or we run into all sorts of borrowing grief.
    pub fn update(game: &mut Game) {
        if game.pov.edition() != game.old_pov.edition {
            for loc in game.pov.locations() {
                let (_, obj) = game.get_top(loc);
                let (_, symbol) = obj.to_fg_symbol();
                game.old_pov.old.insert(*loc, symbol);
            }
            game.old_pov.edition = game.pov.edition();
        }
    }

    pub fn get(&self, loc: &Point) -> Option<char> {
        self.old.get(loc).copied()
    }
}
