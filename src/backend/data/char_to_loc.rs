use super::super::{Event, Oid, Point};
use fnv::FnvHashMap;

pub struct CharToLoc {
    table: FnvHashMap<Oid, Point>,
}

impl CharToLoc {
    pub fn new() -> CharToLoc {
        CharToLoc {
            table: FnvHashMap::default(),
        }
    }

    pub fn lookup(&self, oid: Oid) -> Point {
        *self.table.get(&oid).unwrap()
    }

    pub fn process(&mut self, event: Event) {
        match event {
            Event::AddChar(oid, loc) => {
                self.table.insert(oid, loc);
            }
            Event::MoveChar(oid, loc) => {
                let removed = self.table.remove(&oid);
                assert!(removed.unwrap() != loc); // oid should have had a loc distinct from the new loc
                self.table.insert(oid, loc);
            }
            _ => (),
        }
    }
}
