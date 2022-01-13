use super::{Cell, Event, Point, Tag};
use fnv::FnvHashMap;

pub struct Level {
    pub player: Point,
    pub cells: FnvHashMap<Point, Cell>,
}

impl Level {
    pub fn new() -> Level {
        Level {
            player: Point::origin(),
            cells: FnvHashMap::default(),
        }
    }

    pub fn posted(&mut self, event: &Event) {
        match event {
            Event::NewLevel => {
                self.player = Point::new(20, 10); // TODO: need to do better here
                self.cells = FnvHashMap::default();
            }
            Event::AddObject(loc, obj) => {
                let cell = self.cells.entry(*loc).or_insert_with(Cell::new);
                cell.append(obj.clone());
            }
            Event::ChangeObject(loc, tag, obj) => {
                let cell = self.cells.get_mut(loc).unwrap();
                cell.replace(tag, obj.clone());
            }
            Event::PlayerMoved(loc) => {
                let cell = self.cells.get_mut(&self.player).unwrap();
                let obj = cell.remove(&Tag::Player);
                self.player = *loc;
                let cell = self.cells.entry(self.player).or_insert_with(Cell::new);
                cell.append(obj);
            }
            _ => (),
        };
    }
}
