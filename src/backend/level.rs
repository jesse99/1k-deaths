use super::{Cell, Event, Point, Tag};
use fnv::FnvHashMap;

pub struct Level {
    pub player: Point,
    pub cells: FnvHashMap<Point, Cell>,
}

// TODO: levels should have a default object so if a player does something like
// dig through walls he'll never reach an end
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
                self.cells = FnvHashMap::default();
            }
            Event::AddObject(loc, obj) => {
                if obj.player() {
                    self.player = *loc;
                }
                let cell = self.cells.entry(*loc).or_insert_with(Cell::new);
                cell.append(obj.clone());
            }
            Event::ChangeObject(loc, tag, obj) => {
                let cell = self.cells.get_mut(loc).unwrap();
                cell.replace(tag, obj.clone());
            }
            Event::DestroyObject(loc, tag) => {
                let cell = self.cells.get_mut(loc).unwrap();
                cell.remove(tag);
            }
            Event::PlayerMoved(loc) => {
                let cell = self.cells.get_mut(&self.player).unwrap();
                let obj = cell.remove(&Tag::Player);
                self.player = *loc;
                let cell = self.cells.entry(self.player).or_insert_with(Cell::new);
                cell.append(obj);
            }
            Event::NPCMoved(old, new) => {
                let cell = self.cells.get_mut(old).unwrap();
                let obj = cell.remove(&Tag::Character);
                let cell = self.cells.entry(*new).or_insert_with(Cell::new);
                cell.append(obj);
            }
            _ => (),
        };
    }
}
