use super::{Character, Content, Game, Point, Portable, Terrain};
use fnv::FnvHashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct OldContent {
    pub terrain: Terrain,
    pub character: Option<Character>,
    pub portables: Option<Portable>, // TODO: may want to keep the full list
}

impl OldContent {
    fn new(content: &Content) -> OldContent {
        OldContent {
            terrain: content.terrain,
            character: content.character,
            portables: content.portables.last().copied(),
        }
    }
}

/// Locations that were visible to a character. Note that PoV overrides
/// this so, as an optimization, this may include locations that are actually
/// visible. Currently this is only used for the Player to render locations
/// that he has seen before.
pub struct OldPoV {
    old: FnvHashMap<Point, OldContent>, // may not match the current Level state
    edition: u32,                       // current PoV edition
}

impl OldPoV {
    pub fn new() -> OldPoV {
        OldPoV {
            old: FnvHashMap::default(),
            edition: 0,
        }
    }

    // This can't be an ordinary method or we run into all sorts of borrowing grief.
    pub fn update(game: &mut Game) {
        if game.level.pov.edition() != game.level.old_pov.edition {
            for loc in game.level.pov.locations() {
                match game.tile(*loc) {
                    super::Tile::Visible(content) => game.level.old_pov.old.insert(*loc, OldContent::new(&content)),
                    super::Tile::Stale(_) => panic!("locations in pov should be visible"),
                    super::Tile::NotVisible => panic!("locations in pov should be visible"),
                };
            }
            game.level.old_pov.edition = game.level.pov.edition();
        }
    }

    pub fn get(&self, loc: Point) -> Option<&OldContent> {
        self.old.get(&loc)
    }
}
