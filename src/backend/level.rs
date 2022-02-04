use super::make;
use super::tag::*;
use super::{Cell, Event, Object, Point};
use fnv::FnvHashMap;

pub struct Level {
    player: Point,
    default: Object,
    cells: FnvHashMap<Point, Cell>,
    constructing: bool,
}

// TODO:
// move Level into Game
// should be able to get rid of those internal events
impl Level {
    /// The default object is used if the player (or an NPC) somehow goes
    /// outside the current level (e.g. by digging through a wall).
    pub fn new(default: Object) -> Level {
        Level {
            player: Point::origin(),
            default,
            cells: FnvHashMap::default(),
            constructing: true,
        }
    }

    pub fn player(&self) -> Point {
        self.player
    }

    pub fn get(&self, loc: &Point, tag: TagId) -> Option<(ObjId, &Object)> {
        self.cells.get(loc).unwrap()
    }

    pub fn get_mut(&mut self, loc: &Point) -> &mut Cell {
        self.ensure_neighbors(loc);
        self.cells.get_mut(loc).unwrap()
    }

    // This should only be called by the pov code.
    pub fn ensure_cell(&mut self, loc: &Point) -> bool {
        if self.constructing {
            self.cells.contains_key(loc)
        } else {
            self.ensure_neighbors(loc);
            true
        }
    }

    /// Should not normally be used.
    pub fn try_get(&self, loc: &Point) -> Option<&Cell> {
        self.cells.get(loc)
    }

    pub fn posted(&mut self, event: &Event) {
        match event {
            Event::BeginConstructLevel => {
                self.cells = FnvHashMap::default();
                self.constructing = true;
            }
            Event::EndConstructLevel => {
                self.constructing = false;
            }
            Event::AddObject(loc, obj) => {
                if obj.has(PLAYER_ID) {
                    self.player = *loc;
                }
                let cell = self.cells.entry(*loc).or_insert_with(Cell::new);
                cell.append(obj.clone());
            }
            Event::ChangeObject(loc, id, obj) => {
                let cell = self.cells.get_mut(loc).unwrap();
                cell.replace(*id, obj.clone());
            }
            Event::DestroyObject(loc, id) => {
                self.destroy_object(loc, *id);
            }
            Event::PlayerMoved(loc) => {
                let cell = self.cells.get_mut(&self.player).unwrap();
                let obj = cell.remove(PLAYER_ID);
                self.player = *loc;
                let cell = self.cells.entry(self.player).or_insert_with(Cell::new);
                cell.append(obj);
                self.ensure_neighbors(loc);
            }
            Event::NPCMoved(old, new) => {
                let cell = self.cells.get_mut(old).unwrap();
                let obj = cell.remove(CHARACTER_ID);
                let cell = self.cells.entry(*new).or_insert_with(Cell::new);
                cell.append(obj);
                self.ensure_neighbors(new);
            }
            _ => (),
        };
    }

    fn destroy_object(&mut self, loc: &Point, id: TagId) {
        let cell = self.cells.get_mut(loc).unwrap();
        let obj = cell.get(id);
        if obj.has(TERRAIN_ID) {
            // Terrain cannot be destroyed but has to be mutated into something
            // else.
            if obj.has(WALL_ID) {
                cell.replace(TERRAIN_ID, make::rubble());
            } else {
                error!("Need to better handle destroying TagId {id}"); // Doors, trees, etc
                cell.replace(TERRAIN_ID, make::dirt());
            }
        } else {
            // If it's just a normal object or character we can just nuke
            // the object.
            cell.remove(id);
        }
    }

    // Ideally we would have get_mut and get create a new default cell for
    // the given location. That's easy for get_mut but get would require
    // interior mutability. Also easy..until you start handing out references
    // as get wants to do. We could do that too but then clients have a really
    // annoying constraint: they cannot call get if code anywhere in the call
    // chain has an outstanding cell reference (because get requires that a
    // new mutable reference be taken).
    //
    // So what we do instead is ensure that:
    // 1) When we modify a cell (via get_mut) that all the neighbors are
    // present. This case is for something like destroying a wall.
    // 2) When a character moves we ensure that the new location has all
    // neighbors. This is for something like being able to move into a wall
    // (or something like deep shadow).
    fn ensure_neighbors(&mut self, loc: &Point) {
        if !self.constructing {
            let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
            for delta in deltas {
                let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
                let _ = self.cells.entry(new_loc).or_insert_with(|| {
                    let mut cell = Cell::new();
                    cell.append(self.default.clone());
                    cell
                });
            }
        }
    }
}
