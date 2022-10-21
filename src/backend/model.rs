use super::*;
use fnv::FnvHashMap;

/// Used to construct a Model.
pub struct ModelBuilder {
    objects: FnvHashMap<Oid, Object>,   // all existing objects are here
    cells: FnvHashMap<Point, Vec<Oid>>, // objects within each cell on the map
    default_cell: Vec<Oid>,             // used for locations not in cells
    player_loc: Point,
}

impl ModelBuilder {
    pub fn new() -> ModelBuilder {
        ModelBuilder {
            objects: FnvHashMap::default(),
            cells: FnvHashMap::default(),
            default_cell: vec![],
            player_loc: Point::new(0, 0),
        }
    }

    pub fn set_default(&mut self, oid: Oid, obj: Object) {
        assert!(!self.objects.contains_key(&oid));
        assert!(obj.has(TERRAIN_ID));
        self.objects.insert(oid, obj);

        self.default_cell = vec![oid];
    }

    /// Used for objects that don't match any of the other methods, i.e. not a
    /// character and not terrain.
    pub fn set_item(&mut self, loc: Point, oid: Oid, obj: Object) {
        assert!(!obj.has(CHARACTER_ID));
        assert!(!obj.has(TERRAIN_ID));

        let old = self.objects.insert(oid, obj);
        assert!(old.is_none());

        let oids = self.cells.entry(loc).or_insert_with(Vec::new);
        let index = match oids.last() {
            Some(old) if self.objects[old].has(TERRAIN_ID) => oids.len() - 1,
            Some(_) => oids.len(),
            None => 0,
        };
        oids.insert(index, oid);
    }

    pub fn set_npc(&mut self, loc: Point, oid: Oid, obj: Object) {
        assert!(obj.has(CHARACTER_ID));
        assert!(!obj.has(PLAYER_ID));

        let old = self.objects.insert(oid, obj);
        assert!(old.is_none());

        let oids = self.cells.entry(loc).or_insert_with(Vec::new);
        let old = oids.first(); // cell must be empty or not have a character
        assert!(old.map_or_else(|| true, |oid| !self.objects[oid].has(CHARACTER_ID)));
        oids.insert(0, oid);
    }

    pub fn set_player(&mut self, loc: Point, obj: Object) {
        assert!(obj.has(PLAYER_ID));

        let oid = Oid(0);
        let old = self.objects.insert(oid, obj);
        assert!(old.is_none());

        let oids = self.cells.entry(loc).or_insert_with(Vec::new);
        let old = oids.first(); // cell must be empty or not have a character
        assert!(old.map_or_else(|| true, |oid| !self.objects[oid].has(CHARACTER_ID)));
        oids.insert(0, oid);

        self.player_loc = loc;
    }

    pub fn set_terrain(&mut self, loc: Point, oid: Oid, obj: Object) {
        assert!(obj.has(TERRAIN_ID));

        let old = self.objects.insert(oid, obj);
        assert!(old.is_none());

        let oids = self.cells.entry(loc).or_insert_with(Vec::new);
        match oids.last_mut() {
            Some(old) => *old = oid, // we'll allow builders to overwrite terrain
            None => oids.push(oid),
        }
    }

    // Ensure that all the setter methods have been called.
    fn validate(&self) {
        assert!(self.cells.len() > 1); // at a minimum should have terrain and the player
        assert!(self.cells.contains_key(&self.player_loc));
        assert!(self.default_cell.len() == 1); // note that Model.get and add_default expect only one entry
        for oid in &self.default_cell {
            assert!(self.objects.contains_key(oid));
        }
        assert!(self.objects.contains_key(&Oid(0)));
    }
}

impl From<ModelBuilder> for Model {
    fn from(builder: ModelBuilder) -> Self {
        builder.validate();
        Model::with_builder(builder)
    }
}

/// This is the normalized data associated with a level.
pub struct Model {
    objects: FnvHashMap<Oid, Object>,   // all existing objects are here
    cells: FnvHashMap<Point, Vec<Oid>>, // objects within each cell on the map
    default_cell: Vec<Oid>,             // used for locations not in cells
    player_loc: Point,
    next_id: u64, // 0 is the player
}

impl Model {
    fn with_builder(builder: ModelBuilder) -> Model {
        Model {
            objects: builder.objects,
            cells: builder.cells,
            default_cell: builder.default_cell,
            player_loc: builder.player_loc,
            next_id: 1,
        }
    }

    pub fn player_loc(&self) -> Point {
        self.player_loc
    }

    pub fn get(&self, loc: Point, tag: Tid) -> Option<(Oid, &Object)> {
        if let Some(oids) = self.cells.get(&loc) {
            for oid in oids.iter().rev() {
                let obj = self
                    .objects
                    .get(oid)
                    .expect("All objects in the model should still exist");
                if obj.has(tag) {
                    return Some((*oid, &obj));
                }
            }
        }

        let oid = self.default_cell[0];
        let default = &self.objects[&oid];
        if default.has(tag) {
            // Note that if this cell is converted into a real cell the oid will change.
            // I don't think that this will be a problem in practice...
            Some((oid, default))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, loc: Point, tag: Tid) -> Option<(Oid, &mut Object)> {
        if !self.cells.contains_key(&loc) {
            self.add_default(loc);
        }

        let mut oid = None;
        if let Some(oids) = self.cells.get(&loc) {
            for candidate in oids.iter().rev() {
                let obj = self
                    .objects
                    .get(candidate)
                    .expect("All objects in the model should still exist");
                if obj.has(tag) {
                    oid = Some(candidate);
                    break;
                }
            }
        }

        if let Some(oid) = oid {
            let obj = self.objects.get_mut(oid).unwrap();
            return Some((*oid, obj));
        }

        None
    }

    /// Typically this will be a terrain object.
    pub fn get_bottom(&self, loc: Point) -> (Oid, &Object) {
        if let Some(oids) = self.cells.get(&loc) {
            let oid = oids
                .first()
                .expect("cells should always have at least a terrain object");
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the model should still exist");
            (*oid, obj)
        } else {
            let oid = self.default_cell[0];
            let default = &self.objects[&oid];
            (oid, default)
        }
    }

    /// Character, item, door, or if all else fails terrain.
    pub fn get_top(&self, loc: Point) -> (Oid, &Object) {
        if let Some(oids) = self.cells.get(&loc) {
            let oid = oids.last().expect("cells should always have at least a terrain object");
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the model should still exist");
            (*oid, obj)
        } else {
            let oid = self.default_cell[0];
            let default = &self.objects[&oid];
            (oid, default)
        }
    }

    pub fn cell(&self, loc: Point) -> &Vec<Oid> {
        if let Some(ref oids) = self.cells.get(&loc) {
            oids
        } else {
            &self.default_cell
        }
    }

    /// Iterates over the objects at loc starting with the topmost object.
    pub fn cell_iter(&self, loc: Point) -> impl Iterator<Item = (Oid, &Object)> {
        CellIterator::new(self, loc)
    }

    pub fn obj(&self, oid: Oid) -> &Object {
        self.objects.get(&oid).expect(&format!("oid {oid} isn't in objects"))
    }

    /// Normally Oids are guaranteed to exist but it is possible for objects
    /// to cache an oid that later is destroyed.
    pub fn try_obj(&self, oid: Oid) -> Option<&Object> {
        self.objects.get(&oid)
    }

    pub fn add(&mut self, obj: Object, loc: Option<Point>) -> Oid {
        let oid = self.next_oid(&obj);
        if let Some(loc) = loc {
            trace!("adding {obj} {oid} to {loc}");
        } else {
            trace!("adding {obj} {oid} (no loc)");
        }

        if obj.has(CHARACTER_ID) {
            assert!(loc.is_some(), "Characters should be on the map: {obj:?}");
            if oid.0 == 0 {
                self.player_loc = loc.unwrap();
            }
        }

        if let Some(loc) = loc {
            self.add_to_cell(oid, loc);
        }

        let old = self.objects.insert(oid, obj);
        assert!(old.is_none(), "Level already had oid {oid}");

        oid
    }

    /// Typically this will be a drop from an inventory (or equipped).
    pub fn add_oid(&mut self, oid: Oid, loc: Point) {
        let obj = self.objects.get(&oid).expect(&format!("oid {oid} isn't in objects"));
        assert!(!obj.has(TERRAIN_ID)); // our logic doesn't handle these
        assert!(!obj.has(CHARACTER_ID));

        let oids = self.cells.entry(loc).or_insert_with(Vec::new);
        let index = match oids.first() {
            Some(old) if self.objects[old].has(CHARACTER_ID) => 1,
            _ => 0,
        };
        oids.insert(index, oid);
    }

    /// This is the inverse of add but functions more like destroy.
    pub fn remove(&mut self, oid: Oid, loc: Point) {
        let old = self.objects.remove(&oid).expect(&format!("oid {oid} isn't in objects"));
        assert!(!old.has(TERRAIN_ID)); // terrain can't be removed (but can be replaced)
        trace!("removing {old} {oid} which was at {loc}");

        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == oid).unwrap();
        oids.remove(index);
    }

    /// Character at loc adds oid at loc to its inventory.
    pub fn pickup(&mut self, loc: Point, oid: Oid) {
        let obj = self
            .objects
            .get_mut(&oid)
            .expect(&format!("oid {oid} isn't in objects"));
        assert!(obj.has(PORTABLE_ID));

        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == oid).unwrap();
        oids.remove(index);

        let inv = obj.inventory_value_mut().unwrap();
        inv.push(oid);
    }

    /// Replace oid at loc with a new object/oid.
    pub fn replace(&mut self, loc: Point, old_oid: Oid, new_obj: Object) -> Oid {
        // Fix up npcs.
        let old_obj = &self
            .objects
            .get(&old_oid)
            .expect(&format!("oid {old_oid} isn't in objects"));
        let old_name = old_obj.dname();
        let new_oid = self.next_oid(&new_obj);
        trace!("replacing {old_name} {old_oid} with {new_obj} {new_oid} at {loc}");

        // Fix up objects.
        let old = self.objects.insert(new_oid, new_obj);
        assert!(old.is_none(), "Level already had oid {new_oid}");

        self.objects.remove(&old_oid);

        // Fix up cells.
        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        oids[index] = new_oid;

        new_oid
    }

    pub fn change_loc(&mut self, oid: Oid, from: Point, to: Point) {
        let obj = self.objects.get(&oid).expect(&format!("oid {oid} isn't in objects"));
        assert!(obj.has(PORTABLE_ID) || obj.has(CHARACTER_ID));

        let oids = self.cells.get_mut(&from).unwrap();
        let index = oids
            .iter()
            .position(|id| *id == oid)
            .expect(&format!("expected {oid} at {from}"));
        oids.remove(index);

        // TODO: probably add_to_cell needs to (re)get obj
        self.add_to_cell(oid, to);
        if oid.0 == 0 {
            self.player_loc = to;
        }
    }

    /// Ensure that points around loc are in cells. This is typically used after something
    /// like a dig action so that characters can interact with newly exposed cells.
    pub fn ensure_neighbors(&mut self, loc: Point) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if !self.cells.contains_key(&new_loc) {
                self.add_default(new_loc);
            }
        }
    }

    fn add_to_cell(&mut self, oid: Oid, loc: Point) {
        // Could avoid some duplicate work by having the caller do this
        // but the borrow checker doesn't like that.
        let obj = self.objects.get(&oid).expect(&format!("oid {oid} isn't in objects"));

        let oids = self.cells.entry(loc).or_insert_with(Vec::new);
        let index = if obj.has(TERRAIN_ID) {
            match oids.last() {
                Some(old) if self.objects[old].has(TERRAIN_ID) => panic!("already have terrain"),
                _ => oids.len(),
            }
        } else if obj.has(CHARACTER_ID) {
            match oids.first() {
                Some(old) if self.objects[old].has(CHARACTER_ID) => panic!("already have a character"),
                _ => 0,
            }
        } else {
            match oids.first() {
                Some(old) if self.objects[old].has(CHARACTER_ID) => 1,
                _ => 0,
            }
        };

        oids.insert(index, oid);
    }

    fn add_default(&mut self, new_loc: Point) {
        let old_oid = self.default_cell[0];
        let old_obj = &self.objects[&old_oid];

        let new_obj = old_obj.clone();
        let new_oid = self.next_oid(&new_obj);
        let old = self.objects.insert(new_oid, new_obj);
        assert!(old.is_none());

        let old = self.cells.insert(new_loc, vec![new_oid]);
        assert!(old.is_none());
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

    // These always run in debug.
    #[cfg(debug_assertions)]
    pub fn cheap_invariants(&self, loc: Point) {
        let oid = Oid(self.next_id);
        assert!(!self.objects.contains_key(&oid), "next_oid {oid} is in objects");

        // Can only have one oid in default cell (several methods assume that this is true).
        // It's an array so that we can avoid creating a temporary array.
        let num_default = self.default_cell.len();
        assert!(num_default == 1, "default_cell has len {num_default}");

        // Characters should not be in impassible terrain.
        if let Some((_, ch)) = self.get(loc, CHARACTER_ID) {
            let terrain = self.get(loc, TERRAIN_ID).unwrap().1;
            assert!(
                ch.impassible_terrain(terrain).is_none(),
                "{ch} shouldn't be in {terrain}"
            );
        }
        // Player must be present.
        assert!(self.objects.contains_key(&Oid(0)), "player oid is not in objects");

        let oids = self.cells.get(&self.player_loc).expect("player_loc is not in cells");
        let obj = oids.iter().find(|oid| oid.0 == 0);
        assert!(obj.is_some(), "there's no player at {}", self.player_loc);

        // This is potentially expensive but we're only doing it for one cell.
        let oids = self.cells.get(&loc).expect("cells is missing {loc}");
        self.cell_invariant(loc, oids);
    }

    // These run when --invariants is used.
    #[cfg(debug_assertions)]
    pub fn expensive_invariants(&self) {
        let mut inv_oids = FnvHashSet::default();
        for obj in self.objects.values() {
            if let Some(equipped) = obj.equipped_value() {
                for item in equipped.values() {
                    if let Some(oid) = item {
                        let added = inv_oids.insert(oid);
                        assert!(added, "oid {oid} is in inventory more than once",);
                        assert!(self.objects.contains_key(oid), "equipped oid {oid} is not in objects");
                    }
                }
            }
            if let Some(inv) = obj.inventory_value() {
                for oid in inv.iter() {
                    let added = inv_oids.insert(oid);
                    assert!(added, "oid {oid} is in inventory more than once",);
                    assert!(self.objects.contains_key(oid), "inventory oid {oid} is not in objects");
                }
            }
        }

        let mut level_oids = FnvHashMap::default();
        for (loc, oids) in &self.cells {
            for oid in oids {
                let old = level_oids.insert(loc, oid);
                if let Some(old_loc) = old {
                    panic!("oid {oid} is at {loc} and at {old_loc}");
                }
                assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
                assert!(!inv_oids.contains(&oid), "oid {oid} is in inventory and is at {loc}");
            }
        }

        assert!(
            inv_oids.len() + level_oids.len() == self.objects.len(),
            "all oids must be either on the level or in inventory"
        );

        for oids in self.cells.values() {
            for oid in oids {
                assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
            }
        }
        for oid in &self.default_cell {
            assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
        }

        for (loc, oids) in self.cells.iter() {
            self.cell_invariant(*loc, oids);
        }
    }

    // Cells must be layered properly.
    #[cfg(debug_assertions)]
    fn cell_invariant(&self, loc: Point, oids: &Vec<Oid>) {
        assert!(!oids.is_empty(), "{loc} has no objects"); // should have at least terrain

        let num_chars = oids
            .iter()
            .map(|oid| self.objects.get(oid).unwrap())
            .filter(|obj| obj.has(CHARACTER_ID))
            .count();
        assert!(num_chars <= 1, "{loc} has {num_chars} Characters");

        if num_chars > 0 {
            let oid = oids.first().unwrap();
            let obj = self.objects.get(&oid).unwrap();
            assert!(
                obj.has(CHARACTER_ID),
                "{loc} has a Character but it's not the first object"
            );
        }

        let num_terrain = oids
            .iter()
            .map(|oid| self.objects.get(oid).unwrap())
            .filter(|obj| obj.has(TERRAIN_ID))
            .count();
        assert!(num_terrain == 1, "{loc} has {num_terrain} terrain objects");

        let oid = oids.last().unwrap();
        let obj = self.objects.get(&oid).unwrap();
        assert!(obj.has(TERRAIN_ID), "{loc} has {obj} at the end (expected terrain)");

        for oid in oids {
            let obj = self.objects.get(&oid).unwrap();
            obj.invariant();
        }
    }
}

struct CellIterator<'a> {
    model: &'a Model,
    oids: Option<&'a Vec<Oid>>,
    index: i32,
}

impl<'a> CellIterator<'a> {
    fn new(model: &'a Model, loc: Point) -> CellIterator<'a> {
        let oids = model.cells.get(&loc);
        CellIterator {
            model,
            oids,
            index: oids.map(|list| list.len() as i32).unwrap_or(-1),
        }
    }
}

impl<'a> Iterator for CellIterator<'a> {
    type Item = (Oid, &'a Object);

    fn next(&mut self) -> Option<Self::Item> {
        let oids = self.oids?;
        self.index -= 1;
        if self.index >= 0 {
            let index = self.index as usize;
            let oid = oids[index];
            Some((oid, &self.model.objects.get(&oid).unwrap()))
        } else {
            None // finished iteration
        }
    }
}
