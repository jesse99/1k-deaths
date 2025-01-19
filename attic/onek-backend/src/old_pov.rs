use super::{Cell, Game, Point, Tag, Value, DEFAULT_CELL_OID};
use fnv::{FnvHashMap, FnvHashSet};
use onek_shared::Object;

/// Locations that were visible to a character. Note that PoV overrides
/// this so, as an optimization, this may include locations that are actually
/// visible. Currently this is only used for the Player to render locations
/// that he has seen before.
pub struct OldPoV {
    stale_cells: FnvHashMap<Point, Cell>, // may not match the current level state
    last_visible: FnvHashSet<Point>,      // locs in PoV last time around
    edition: u32,                         // current PoV edition
}

impl OldPoV {
    pub fn new() -> OldPoV {
        OldPoV {
            stale_cells: FnvHashMap::default(),
            last_visible: FnvHashSet::default(),
            edition: 0,
        }
    }

    // This can't be an ordinary method or we run into all sorts of borrowing grief.
    pub fn update(game: &mut Game) {
        if game.pov.edition() != game.old_pov.edition {
            for loc in game.pov.visible.union(&game.old_pov.last_visible) {
                let was_visible = game.old_pov.last_visible.contains(&loc);
                let is_visible = game.pov.visible.contains(&loc);

                if was_visible && is_visible {
                    game.old_pov.stale_cells.remove(&loc);
                } else if was_visible && !is_visible {
                    let default = vec![DEFAULT_CELL_OID];
                    let oids = game.level.get(&loc).unwrap_or(&default);
                    let objects = oids
                        .iter()
                        .map(|oid| stale_obj(game.objects.get(&oid).unwrap()))
                        .collect();
                    game.old_pov.stale_cells.insert(*loc, objects);
                } else if !was_visible && is_visible {
                    game.old_pov.stale_cells.remove(&loc);
                } else {
                    // do nothing if not visible and wasn't visible
                }
            }

            game.old_pov.last_visible = game.pov.visible.clone();
            game.old_pov.edition = game.pov.edition();
        }
    }

    pub fn get(&self, loc: &Point) -> Option<&Cell> {
        self.stale_cells.get(loc)
    }
}

fn stale_obj(old_object: &Object) -> Object {
    let mut object = Object::default();

    object.insert("tag".to_owned(), Value::Tag(Tag("stale".to_owned())));
    for name in vec!["description", "symbol", "color", "back_color"] {
        if let Some(value) = old_object.get(name) {
            object.insert(name.to_owned(), value.clone());
        }
    }

    object
}
