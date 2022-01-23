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
            Event::BeginConstructLevel => self.dirty = true,
            Event::EndConstructLevel => self.dirty = true,
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
    pub fn visible(&self, loc: &Point) -> bool {
        assert!(!self.dirty);
        self.visible.contains(loc)
    }

    pub fn refresh(&mut self, origin: &Point, level: &mut Level) {
        if self.dirty {
            self.do_refresh(origin, level);
            self.edition = self.edition.wrapping_add(1);
            self.dirty = false;
        }
    }

    // Level is mutable so that we can create a Cell if one isn't already there.
    fn do_refresh(&mut self, origin: &Point, level: &mut Level) {
        self.visible.clear();

        let mut new_locs = Vec::new();
        let mut view = FoV {
            start: *origin,
            radius: 15, // TODO: do better with this
            visible_tile: |loc| {
                new_locs.push(loc);
            },
            blocks_los: { |loc| blocks_los(level.try_get(&loc)) },
        };
        view.visit();

        for loc in new_locs {
            if level.ensure_cell(&loc) {
                self.visible.insert(loc);
            }
        }
    }
}

fn obj_blocks_los(obj: &Object) -> bool {
    if let Some(false) = obj.door() {
        true
    } else {
        obj.wall()
    }
}

fn blocks_los(cell: Option<&Cell>) -> bool {
    match cell {
        Some(cell) => cell.iter().any(obj_blocks_los),
        None => true,
    }
}
