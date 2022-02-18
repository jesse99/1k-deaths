use crate::backend::tag::CHARACTER_ID;
use std::cell::{Cell, RefCell};

use super::*;
use fnv::FnvHashMap;

struct Entry {
    obj: Object,
    loc: Option<Point>, // some objects are in Inventory tags
}

pub struct Lookup {
    objects: FnvHashMap<Oid, Entry>,    // all existing objects are here
    cells: FnvHashMap<Point, Vec<Oid>>, // objects within each cell on the map
    npcs: RefCell<Vec<Oid>>,            // all NPCs sorted so that the first is closest to the player
    sorted: Cell<bool>,                 // false if npcs needs to be re-sorted
    next_id: u64,                       // 0 is the player, 1 is the default object
    player_loc: Point,
    default: Object,
    default_oids: Vec<Oid>,
    changed: Point,     // the loc that was last modified, used for cheap invariants
    constructing: bool, // level is in the process of being constructed
    #[cfg(debug_assertions)]
    invariants: bool, // if true then expensive checks are enabled
}

impl Lookup {
    pub fn new() -> Lookup {
        Lookup {
            objects: FnvHashMap::default(),
            cells: FnvHashMap::default(),
            npcs: RefCell::new(Vec::new()),
            sorted: Cell::new(true),
            next_id: 2,
            player_loc: Point::new(0, 0),
            default: super::make::stone_wall(),
            default_oids: vec![Oid(1)],
            changed: Point::new(0, 0),
            constructing: true,
            #[cfg(debug_assertions)]
            invariants: false,
        }
    }

    pub fn set_constructing(&mut self, value: bool) {
        self.constructing = value;
    }

    #[cfg(debug_assertions)]
    pub fn set_invariants(&mut self, enable: bool) {
        // TODO: might want a wizard command to enable these
        self.invariants = enable;
    }

    pub fn player_loc(&self) -> Point {
        self.player_loc
    }

    // pub fn has(&self, loc: &Point, tag: Tid) -> bool {
    //     if let Some(oids) = self.cells.get(loc) {
    //         for oid in oids {
    //             let entry = self
    //                 .objects
    //                 .get(oid)
    //                 .expect("All objects in the level should still exist");
    //             if entry.obj.has(tag) {
    //                 return true;
    //             }
    //         }
    //     }
    //     self.default.has(tag)
    // }

    pub fn get(&self, loc: &Point, tag: Tid) -> Option<(Oid, &Object)> {
        if let Some(oids) = self.cells.get(loc) {
            for oid in oids.iter().rev() {
                let entry = self
                    .objects
                    .get(oid)
                    .expect("All objects in the level should still exist");
                if entry.obj.has(tag) {
                    return Some((*oid, &entry.obj));
                }
            }
        }
        if self.default.has(tag) {
            // Note that if this cell is converted into a real cell the oid will change.
            // I don't think that this will be a problem in practice...
            Some((Oid(1), &self.default))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, loc: &Point, tag: Tid) -> Option<(Oid, &mut Object)> {
        if !self.cells.contains_key(loc) {
            self.add_default(loc);
        }

        let mut oid = None;
        if let Some(oids) = self.cells.get(loc) {
            for candidate in oids.iter().rev() {
                let entry = self
                    .objects
                    .get(candidate)
                    .expect("All objects in the level should still exist");
                if entry.obj.has(tag) {
                    oid = Some(candidate);
                    break;
                }
            }
        }

        if let Some(oid) = oid {
            let entry = self.objects.get_mut(oid).unwrap();
            return Some((*oid, &mut entry.obj));
        }

        None
    }

    /// Typically this will be a terrain object.
    pub fn get_bottom(&self, loc: &Point) -> (Oid, &Object) {
        if let Some(oids) = self.cells.get(loc) {
            let oid = oids
                .first()
                .expect("cells should always have at least a terrain object");
            let entry = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            (*oid, &entry.obj)
        } else {
            (Oid(1), &self.default)
        }
    }

    /// Character, item, door, or if all else fails terrain.
    pub fn get_top(&self, loc: &Point) -> (Oid, &Object) {
        if let Some(oids) = self.cells.get(loc) {
            let oid = oids.last().expect("cells should always have at least a terrain object");
            let entry = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            (*oid, &entry.obj)
        } else {
            (Oid(1), &self.default)
        }
    }

    /// Iterates over the objects at loc starting with the topmost object.
    pub fn cell_iter(&self, loc: &Point) -> impl Iterator<Item = (Oid, &Object)> {
        CellIterator::new(self, loc)
    }

    pub fn obj(&self, oid: Oid) -> (&Object, Option<Point>) {
        let entry = self.objects.get(&oid).expect(&format!("oid {oid} isn't in objects"));
        (&entry.obj, entry.loc)
    }

    pub fn cell(&self, loc: &Point) -> &Vec<Oid> {
        if let Some(ref oids) = self.cells.get(loc) {
            oids
        } else {
            &self.default_oids
        }
    }

    /// Note that this is sorted by distance from the player (closest first) and does not
    /// consider PoV.
    pub fn npcs(&self) -> impl Iterator<Item = Oid> + '_ {
        if !self.sorted.get() {
            // This will normally be mostly sorted so it should be pretty close to an O(N)
            // operation. Still it's expensive enough that we want to defer sorting until
            // we actually need it.
            self.npcs.borrow_mut().sort_by(|a, b| {
                let a = self.obj(*a).1.unwrap();
                let b = self.obj(*b).1.unwrap();
                let a = a.distance2(&self.player_loc);
                let b = b.distance2(&self.player_loc);
                a.cmp(&b)
            });
            self.sorted.set(true);
        }
        NpcsIterator {
            lookup: self,
            index: -1,
        }
    }

    pub fn add(&mut self, obj: Object, loc: Option<Point>) -> Oid {
        let oid = self.next_oid(&obj);

        if obj.has(CHARACTER_ID) {
            assert!(loc.is_some(), "Characters should be on the map: {obj:?}");
            if oid.0 == 0 {
                self.player_loc = loc.unwrap();
            } else {
                self.npcs.borrow_mut().push(oid);
                self.sorted.set(false);
            }
        }

        let old = self.objects.insert(oid, Entry { obj, loc });
        assert!(old.is_none(), "Lookup already had oid {oid}");

        if let Some(loc) = loc {
            let oids = self.cells.entry(loc).or_insert_with(Vec::new);
            oids.push(oid);
            self.changed = loc;
        }

        {
            #[cfg(debug_assertions)]
            self.invariant();
        }

        oid
    }

    pub fn remove(&mut self, oid: Oid) {
        let entry = self.objects.get(&oid).expect(&format!("oid {oid} isn't in objects"));
        if let Some(loc) = entry.loc {
            let oids = self.cells.get_mut(&loc).unwrap();
            let index = oids.iter().position(|id| *id == oid).unwrap();
            oids.remove(index);
            self.changed = loc;
        }

        if oid.0 != 0 && entry.obj.has(CHARACTER_ID) {
            let index = self.npcs.borrow().iter().position(|id| *id == oid).unwrap();
            self.npcs.borrow_mut().remove(index);
        }

        self.objects.remove(&oid);

        {
            #[cfg(debug_assertions)]
            self.invariant();
        }
    }

    pub fn pickup(&mut self, loc: &Point, oid: Oid) {
        let mut entry = self
            .objects
            .get_mut(&oid)
            .expect(&format!("oid {oid} isn't in objects"));
        if let Some(loc) = entry.loc {
            let oids = self.cells.get_mut(&loc).unwrap();
            let index = oids.iter().position(|id| *id == oid).unwrap();
            oids.remove(index);
            self.changed = loc;
        }
        entry.loc = None;
        assert!(!entry.obj.has(CHARACTER_ID));

        let obj = self.get_mut(loc, INVENTORY_ID).unwrap().1;
        let inv = obj.as_mut_ref(INVENTORY_ID).unwrap();
        inv.push(oid);

        {
            #[cfg(debug_assertions)]
            self.invariant();
        }
    }

    pub fn replace(&mut self, loc: &Point, old_oid: Oid, new_obj: Object) -> Oid {
        // Fix up npcs.
        let old_obj = &self.objects.get(&old_oid).unwrap().obj;
        if old_obj.has(CHARACTER_ID) {
            assert!(old_oid.0 > 1);
            let mut oids = self.npcs.borrow_mut();
            let index = oids.iter().position(|id| *id == old_oid).unwrap();
            oids.remove(index);
        }

        let new_oid = self.next_oid(&new_obj);
        if new_obj.has(CHARACTER_ID) {
            assert!(new_oid.0 > 1);
            self.npcs.borrow_mut().push(new_oid);
            self.sorted.set(false);
        }

        // Fix up objects.
        let old = self.objects.insert(
            new_oid,
            Entry {
                obj: new_obj,
                loc: Some(*loc),
            },
        );
        assert!(old.is_none(), "Lookup already had oid {new_oid}");

        self.objects.remove(&old_oid);

        // Fix up cells.
        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        oids[index] = new_oid;

        self.changed = *loc;
        new_oid
    }

    pub fn moved(&mut self, oid: Oid, from: &Point, to: &Point) {
        assert!(!self.constructing); // make sure this is reset once things start happening
        let entry = self
            .objects
            .get_mut(&oid)
            .expect(&format!("oid {oid} isn't in objects"));
        entry.loc = Some(*to);

        let oids = self.cells.get_mut(from).unwrap();
        let index = oids
            .iter()
            .position(|id| *id == oid)
            .expect(&format!("expected {oid} at {from}"));
        oids.remove(index);

        let oids = self.cells.entry(*to).or_insert_with(Vec::new);
        oids.push(oid);

        self.sorted.set(false);
        self.changed = *to;

        if oid.0 == 0 {
            self.player_loc = *to;
        }

        {
            #[cfg(debug_assertions)]
            self.invariant();
        }
    }

    pub fn ensure_neighbors(&mut self, loc: &Point) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if !self.cells.contains_key(&new_loc) {
                self.add_default(&new_loc);
            }
        }
    }

    fn next_oid(&mut self, obj: &Object) -> Oid {
        if obj.has(PLAYER_ID) {
            Oid(0)
        } else {
            let o = Oid(self.next_id);
            self.next_id += 1;
            o
        }
    }

    fn add_default(&mut self, new_loc: &Point) {
        let oid = Oid(self.next_id);
        self.next_id += 1;
        self.objects.insert(
            oid,
            Entry {
                obj: self.default.clone(),
                loc: Some(*new_loc),
            },
        );
        let old_oids = self.cells.insert(*new_loc, vec![oid]);
        assert!(old_oids.is_none());
    }
}

// Debugging support
impl Lookup {
    #[cfg(debug_assertions)]
    fn invariant(&self) {
        if self.constructing {
            return;
        }

        // Check what we can that isn't very expensive to do.
        let entry = self.objects.get(&Oid(0)).expect("oid 0 should always exist");
        assert!(entry.obj.has(PLAYER_ID), "oid 0 should be the player not {}", entry.obj);

        let entry = self.objects.get(&Oid(1));
        assert!(
            entry.is_none(),
            "oid 1 should be the default object, not {}",
            entry.unwrap().obj
        );

        let entry = self.objects.get(&Oid(self.next_id));
        assert!(entry.is_none(), "next_id is somehow {}", entry.unwrap().obj);

        let oids = self.cells.get(&self.player_loc).expect("player should be on the map");
        assert!(
            oids.iter().any(|oid| self.objects.get(oid).unwrap().obj.has(PLAYER_ID)),
            "player isn't at {}",
            self.player_loc
        );

        self.cheap_invariants(&self.changed);
        if self.invariants {
            self.expensive_invariants(); // some overlap with cheap_invariants but that should be OK
        }
    }

    // This only checks invariants at one cell. Not ideal but it does give us some coverage
    // of the level without being really expensive.
    #[cfg(debug_assertions)]
    fn cheap_invariants(&self, loc: &Point) {
        let oids = self.cells.get(loc).expect(&format!("cell at {loc} should exist"));
        assert!(
            !oids.is_empty(),
            "cell at {loc} is empty (should have at least a terrain object)"
        );

        if let Some((_, ch)) = self.get(loc, CHARACTER_ID) {
            let terrain = self.get(loc, TERRAIN_ID).unwrap().1;
            assert!(
                ch.impassible_terrain(terrain).is_none(),
                "{ch} shouldn't be in {terrain}"
            );
        }

        for (i, oid) in oids.iter().enumerate() {
            let entry = self
                .objects
                .get(oid)
                .expect(&format!("oid {oid} at {loc} is not in objects"));

            if i == 0 {
                assert!(
                    entry.obj.has(TERRAIN_ID),
                    "cell at {loc} has {} for the first object instead of a terrain object",
                    entry.obj
                );
            } else {
                assert!(
                    !entry.obj.has(TERRAIN_ID),
                    "cell at {loc} has {} which isn't at the bottom",
                    entry.obj
                );
            }

            if i < oids.len() - 1 {
                assert!(
                    !entry.obj.has(CHARACTER_ID),
                    "cell at {loc} has {} which isn't at the top",
                    entry.obj
                );
            }

            entry.obj.invariant();
        }
    }

    // This checks every cell and every object so it is pretty slow.
    #[cfg(debug_assertions)]
    fn expensive_invariants(&self) {
        // First we'll check global constraints.
        let mut all_oids = FnvHashSet::default();
        for (loc, oids) in &self.cells {
            for oid in oids {
                assert!(all_oids.insert(oid), "{loc} has oid {oid} which exists elsewhere");
                assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
            }
        }

        for oid in self.npcs.borrow().iter() {
            assert!(all_oids.contains(&oid), "{oid} NPC isn't on the map");
        }

        for entry in self.objects.values() {
            if let Some(oids) = entry.obj.as_ref(INVENTORY_ID) {
                for oid in oids {
                    assert!(
                        all_oids.insert(oid),
                        "{} has oid {oid} which exists elsewhere",
                        entry.obj
                    );
                    assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
                }
            }
        }

        assert_eq!(
            all_oids.len(),
            self.objects.len(),
            "all objects should be used somewhere"
        );

        // Then we'll verify that the objects in a cell are legit.
        for (loc, oids) in &self.cells {
            assert!(
                !oids.is_empty(),
                "cell at {loc} is empty (should have at least a terrain object)"
            );
            let entry = self.objects.get(&oids[0]).unwrap();
            assert!(
                entry.obj.has(TERRAIN_ID),
                "cell at {loc} has {} for the first object instead of a terrain object",
                entry.obj
            );
            assert!(
                !oids
                    .iter()
                    .skip(1)
                    .any(|oid| self.objects.get(oid).unwrap().obj.has(TERRAIN_ID)),
                "cell at {loc} has multiple terrain objects"
            );

            let index = oids.iter().position(|oid| {
                let entry = self.objects.get(oid).unwrap();
                entry.obj.has(CHARACTER_ID)
            });
            if let Some(index) = index {
                // If not cells won't render quite right.
                assert!(index == oids.len() - 1, "{loc} has a Character that is not at the top")
            }
        }

        // Finally we'll check each individual object.
        for entry in self.objects.values() {
            entry.obj.invariant();
        }
    }
}

struct CellIterator<'a> {
    lookup: &'a Lookup,
    oids: Option<&'a Vec<Oid>>,
    index: i32,
}

impl<'a> CellIterator<'a> {
    fn new(lookup: &'a Lookup, loc: &Point) -> CellIterator<'a> {
        let oids = lookup.cells.get(loc);
        CellIterator {
            lookup,
            oids,
            index: oids.map(|list| list.len() as i32).unwrap_or(-1),
        }
    }
}

impl<'a> Iterator for CellIterator<'a> {
    type Item = (Oid, &'a Object);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(oids) = self.oids {
            self.index -= 1;
            if self.index >= 0 {
                let index = self.index as usize;
                let oid = oids[index];
                Some((oid, &self.lookup.objects.get(&oid).unwrap().obj))
            } else {
                None // finished iteration
            }
        } else {
            None // nothing at the loc
        }
    }
}

struct NpcsIterator<'a> {
    lookup: &'a Lookup,
    index: i32,
}

impl<'a> Iterator for NpcsIterator<'a> {
    type Item = Oid;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        let npcs = self.lookup.npcs.borrow();
        let index = self.index as usize;
        if index < npcs.len() {
            Some(npcs[index])
        } else {
            None
        }
    }
}
