use super::tag::*;
use super::{Color, Object};
use std::fmt::{self, Formatter};
use std::ops::{Deref, DerefMut};

/// Levels are composed of Object's and cells contain Object's.
pub struct Cell {
    objects: Vec<Object>,
}

impl Cell {
    pub fn new() -> Cell {
        Cell { objects: Vec::new() }
    }

    pub fn to_bg_fg_symbol(&self) -> (Color, Color, char) {
        let bg = self.objects.first().unwrap().to_bg_color();
        let (fg, symbol) = self.objects.last().unwrap().to_fg_symbol();
        (bg, fg, symbol)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Object> {
        self.objects.iter()
    }

    pub fn append(&mut self, object: Object) {
        self.objects.push(object);
        self.invariant();
    }

    pub fn terrain(&self) -> &Object {
        &self.objects[0]
    }

    pub fn contains(&self, id: u16) -> bool {
        self.objects.iter().any(|obj| obj.has(id))
    }

    pub fn get(&self, id: u16) -> &Object {
        if let Some(index) = self.objects.iter().position(|obj| obj.has(id)) {
            &self.objects[index]
        } else {
            panic!("failed to find id {}", id);
        }
    }

    pub fn get_mut(&mut self, id: u16) -> DerefObj<'_> {
        if let Some(index) = self.objects.iter().position(|obj| obj.has(id)) {
            DerefObj { cell: self, index }
        } else {
            panic!("failed to find id {}", id);
        }
    }

    pub fn remove(&mut self, id: u16) -> Object {
        if let Some(index) = self.objects.iter().position(|obj| obj.has(id)) {
            let obj = self.objects.remove(index);
            self.invariant();
            obj
        } else {
            panic!("failed to find id {}", id);
        }
    }

    pub fn replace(&mut self, id: u16, new_obj: Object) {
        let index = self.objects.iter().position(|obj| obj.has(id)).unwrap();
        self.objects[index] = new_obj;
        self.invariant();
    }

    #[cfg(debug_assertions)] // TODO: make sure that this works
    fn invariant(&self) {
        for obj in &self.objects {
            obj.invariant();
        }

        assert!(
            !self.objects.is_empty(),
            "Cells must have at least a Terrain object: {self}"
        );
        assert!(
            self.objects[0].has(TERRAIN_ID),
            "First object in a Cell must be a Terrain object: {self}"
        );

        let count = self.objects.iter().filter(|obj| obj.has(TERRAIN_ID)).count();
        assert!(count == 1, "There must be only one terrain object in a Cell: {self}");

        let count = self.objects.iter().filter(|obj| obj.has(CHARACTER_ID)).count();
        assert!(
            count <= 1,
            "There cannot be more than one Character object in a Cell: {self}"
        );
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s: Vec<String> = self.objects.iter().map(|obj| format!("{obj}")).collect();
        let s = s.join(", ");
        write!(f, "[{s}]")
    }
}

// Bit of machinery that allows us to call Cell::invariant after clients
// modify the interior bits with Cell::get_mut.
pub struct DerefObj<'a> {
    cell: &'a mut Cell,
    index: usize,
}

// We don't directly use this but DerefMut requires that it be defined.
impl Deref for DerefObj<'_> {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.cell.objects[self.index]
    }
}

impl DerefMut for DerefObj<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cell.objects[self.index]
    }
}

impl Drop for DerefObj<'_> {
    fn drop(&mut self) {
        self.cell.invariant();
    }
}
