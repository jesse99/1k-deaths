use super::details::Game1;
use super::primitives::FoV;
use super::{Cell, Event, Level, Object, Point};
use fnv::FnvHashSet;

/// Field of View for a character. These are invalidated for certain events
/// (e.g. terrain changes).
pub struct PoV {
    edition: u32, // incremented each time visible is updated
    visible: FnvHashSet<Point>,
    dirty: bool, // true if visible is invalid
}

impl PoV {
    pub fn new() -> PoV {
        PoV {
            edition: 0,
            visible: FnvHashSet::default(),
            dirty: true,
        }
    }

    // TODO: visibility checks need to be some sort action, i.e. double dispatch
    // would it help if objects kept track of a location (point, in inv, or equipped)?
    // or do we need to give objects a unique id? could use a new state object to track that
    //    think we'll need something like that for stuff like ranged combat
    //    want to be able to easily attack the same NPC even if it moved
    pub fn posted(&mut self, _game: &Game1, event: &Event) {
        match event {
            Event::NewGame => self.dirty = true,
            Event::NewLevel => self.dirty = true,
            Event::AddObject(_loc, new_obj) => {
                if !self.dirty && obj_blocks_los(new_obj) {
                    self.dirty = true;
                }
            }
            Event::ChangeObject(_loc, _tag, _new_obj) => {
                // TODO: This is currently only used for terrain, e.g. to open
                // a door. These changes will normally require dirtying the PoV
                // so, in theory, we could be smarter about this (but note that
                // the Level has already changed).
                self.dirty = true;
            }
            // TODO: this should dirty only if the origin changes. Maybe we can add an id to PoV
            // and check to see if loc matches that id's location.
            Event::PlayerMoved(_loc) => self.dirty = true,
            _ => (),
        };
    }

    pub fn edition(&self) -> u32 {
        self.edition
    }

    /// Returns an iterator covering all the visible locations.
    pub fn locations(&self) -> std::collections::hash_set::Iter<'_, Point> {
        self.visible.iter()
    }

    /// Returns true if loc is visible from origin.
    pub fn visible(&mut self, origin: &Point, level: &Level, loc: &Point) -> bool {
        if self.dirty {
            self.refresh(origin, level);
            self.edition = self.edition.wrapping_add(1);
            self.dirty = false;
        }

        self.visible.contains(loc)
    }

    fn refresh(&mut self, origin: &Point, level: &Level) {
        self.visible.clear();

        let mut view = FoV {
            start: *origin,
            radius: 15, // TODO: do better with this
            visible_tile: |loc| {
                self.visible.insert(loc);
            },
            blocks_los: |loc| match level.cells.get(&loc) {
                Some(cell) => blocks_los(cell),
                None => true,
            },
        };
        view.visit();
    }
}

fn obj_blocks_los(obj: &Object) -> bool {
    if let Some(false) = obj.door() {
        true
    } else {
        obj.wall()
    }
}

fn blocks_los(cell: &Cell) -> bool {
    cell.iter().any(obj_blocks_los)
}
