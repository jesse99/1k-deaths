use crate::backend::tag::CHARACTER_ID;
// use rand::prelude::*;
use rand::rngs::SmallRng;
use std::cell::{Cell, RefCell};

use super::model::*;
use super::*;
use fnv::FnvHashMap;

pub struct Level {
    model: Model,
    locations: FnvHashMap<Oid, Point>,
    npcs: RefCell<Vec<Oid>>, // all NPCs sorted so that the first is closest to the player
    sorted: Cell<bool>,      // false if npcs needs to be re-sorted
    changed: Point,          // the loc that was last modified, used for cheap invariants
    #[cfg(debug_assertions)]
    invariants: bool, // if true then expensive checks are enabled
}

impl Level {
    pub fn new(model: Model) -> Level {
        Level {
            model,
            locations: FnvHashMap::default(),
            npcs: RefCell::new(Vec::new()),
            sorted: Cell::new(true),
            changed: Point::new(0, 0),
            #[cfg(debug_assertions)]
            invariants: false,
        }
    }

    #[cfg(debug_assertions)]
    pub fn set_invariants(&mut self, enable: bool) {
        // TODO: might want a wizard command to enable these
        self.invariants = enable;
    }

    pub fn player_loc(&self) -> Point {
        self.model.player_loc()
    }

    // TODO: move these into a Model forwarding Impl
    pub fn get(&self, loc: Point, tag: Tid) -> Option<(Oid, &Object)> {
        self.model.get(loc, tag)
    }

    pub fn get_mut(&mut self, loc: Point, tag: Tid) -> Option<(Oid, &mut Object)> {
        self.model.get_mut(loc, tag)
    }

    /// Typically this will be a terrain object.
    pub fn get_bottom(&self, loc: Point) -> (Oid, &Object) {
        self.model.get_bottom(loc)
    }

    /// Character, item, door, or if all else fails terrain.
    pub fn get_top(&self, loc: Point) -> (Oid, &Object) {
        self.model.get_top(loc)
    }

    /// Iterates over the objects at loc starting with the topmost object.
    pub fn cell_iter(&self, loc: Point) -> impl Iterator<Item = (Oid, &Object)> {
        self.model.cell_iter(loc)
    }

    pub fn obj(&self, oid: Oid) -> &Object {
        self.model.obj(oid)
    }

    // TODO: think we need an oid => location map
    pub fn obj_loc(&self, oid: Oid) -> Option<&Point> {
        self.locations.get(&oid)
    }

    pub fn try_obj(&self, oid: Oid) -> Option<&Object> {
        self.model.try_obj(oid)
    }

    pub fn try_loc(&self, oid: Oid) -> Option<&Point> {
        self.locations.get(&oid)
    }

    pub fn cell(&self, loc: Point) -> &Vec<Oid> {
        self.model.cell(loc)
    }

    /// Note that this is sorted by distance from the player (closest first) and does not
    /// consider PoV.
    pub fn npcs(&self) -> impl Iterator<Item = Oid> + '_ {
        if !self.sorted.get() {
            // This will normally be mostly sorted so it should be pretty close to an O(N)
            // operation. Still it's expensive enough that we want to defer sorting until
            // we actually need it.
            self.npcs.borrow_mut().sort_by(|a, b| {
                let a = self.obj_loc(*a).unwrap();
                let b = self.obj_loc(*b).unwrap();
                let a = a.distance2(self.model.player_loc());
                let b = b.distance2(self.model.player_loc());
                a.cmp(&b)
            });
            self.sorted.set(true);
        }
        NpcsIterator { level: self, index: -1 }
    }

    /// Returns a random cell on the map.
    pub fn random_loc(&self, rng: &RefCell<SmallRng>) -> Point {
        // Note that this is O(N) because values is an iterator not a slice.
        // If this is a problem we could have a random list of locations
        // behind a RefCell.
        *self.locations.values().choose(&mut *rng.borrow_mut()).unwrap()
    }

    pub fn add(&mut self, obj: Object, loc: Option<Point>) -> Oid {
        let is_char = obj.has(CHARACTER_ID);
        let oid = self.model.add(obj, loc);
        let is_npc = is_char && oid.0 != 0;

        if let Some(loc) = loc {
            let old = self.locations.insert(oid, loc);
            assert!(old.is_none(), "oid {oid} was already in locations");
            self.changed = loc;
        }

        if is_npc {
            self.npcs.borrow_mut().push(oid);
            self.sorted.set(false);
        }

        if cfg!(debug_assertions) {
            self.invariant();
        }

        oid
    }

    /// Typically this will be a drop from an inventory (or equipped).
    pub fn add_oid(&mut self, oid: Oid, loc: Point) {
        let obj = self.model.obj(oid);
        assert!(!obj.has(CHARACTER_ID));

        let old = self.locations.insert(oid, loc);
        assert!(old.is_none(), "oid {oid} was already in locations");
        self.changed = loc;

        if cfg!(debug_assertions) {
            self.invariant();
        }

        if cfg!(debug_assertions) {
            self.invariant();
        }
    }

    pub fn remove(&mut self, oid: Oid, loc: Point) {
        let old = self.locations.remove(&oid);
        assert!(old.is_some(), "oid {oid} was not in locations");
        self.changed = loc;

        let obj = self.model.obj(oid);
        if oid.0 != 0 && obj.has(CHARACTER_ID) {
            let index = self.npcs.borrow().iter().position(|id| *id == oid).unwrap();
            self.npcs.borrow_mut().remove(index);
        }

        self.model.remove(oid, loc);

        if cfg!(debug_assertions) {
            self.invariant();
        }
    }

    pub fn pickup(&mut self, loc: Point, oid: Oid) {
        let obj = self.model.obj(oid);
        assert!(!obj.has(CHARACTER_ID));

        let old = self.locations.remove(&oid);
        assert!(old.is_some(), "oid {oid} was not in locations");
        self.changed = loc;

        self.model.pickup(loc, oid);

        if cfg!(debug_assertions) {
            self.invariant();
        }
    }

    pub fn replace(&mut self, loc: Point, old_oid: Oid, new_obj: Object) -> Oid {
        let old = self.locations.remove(&old_oid);
        assert!(old.is_some(), "oid {old_oid} is missing from locations");

        let old_obj = self.model.obj(old_oid);
        let old_name = old_obj.dname();
        if old_obj.has(CHARACTER_ID) {
            assert!(old_oid.0 > 1);
            let mut oids = self.npcs.borrow_mut();
            let index = oids.iter().position(|id| *id == old_oid).unwrap();
            oids.remove(index);
        }

        let is_char = new_obj.has(CHARACTER_ID);
        let new_oid = self.model.replace(loc, old_oid, new_obj);
        self.locations.insert(new_oid, loc);
        if is_char {
            assert!(new_oid.0 > 1);
            self.npcs.borrow_mut().push(new_oid);
            self.sorted.set(false);
        }

        self.changed = loc;
        if cfg!(debug_assertions) {
            self.invariant();
        }

        new_oid
    }

    pub fn change_loc(&mut self, oid: Oid, from: Point, to: Point) {
        let old = self.locations.insert(oid, to);
        assert!(old.unwrap() == from);

        self.model.change_loc(oid, from, to);

        self.sorted.set(false); // technically we should do this only if oid has a CHARACTER_ID, but very little moves other than characters
        self.changed = to;
        if cfg!(debug_assertions) {
            self.invariant();
        }
    }

    pub fn ensure_neighbors(&mut self, loc: Point) {
        self.model.ensure_neighbors(loc);
    }
}

// Debugging support
impl Level {
    #[cfg(debug_assertions)]
    fn invariant(&self) {
        self.model.cheap_invariants(self.changed);

        if self.invariants {
            self.model.expensive_invariants();
            self.expensive_invariants();
        }
    }

    // every char should be in locations
    // every oid in npcs should exist
    // every oid in npcs should be a non-player character
    // should model have a changed flag? or maybe its invariant can take a changed flag?
    #[cfg(debug_assertions)]
    fn expensive_invariants(&self) {
        // TODO: should we get lists of portable and characters from model?
        // then could assert that lists match
        // would probably have to sort these
        // getters could be protected by debug_assertions
        // maybe just one getter (could return a tuple)
        for (oid, loc) in self.locations {
            assert!(self.model.try_obj(oid).is_some(), "oid {oid} does not exist");
            let oids = self.model.cell(loc);
            assert!(oids.contains(&oid), "{loc} is missing oid {oid}");
        }
    }
}
struct NpcsIterator<'a> {
    level: &'a Level,
    index: i32,
}

impl<'a> Iterator for NpcsIterator<'a> {
    type Item = Oid;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        let npcs = self.level.npcs.borrow();
        let index = self.index as usize;
        if index < npcs.len() {
            Some(npcs[index])
        } else {
            None
        }
    }
}
