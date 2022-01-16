use super::{Color, Object, Tag};
use std::fmt::{self, Formatter};

/// Levels are composed of Cell's and cells contain Object's.
pub struct Cell {
    objects: Vec<Object>,
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            objects: Vec::new(),
        }
    }

    pub fn to_bg_fg_symbol(&self) -> (Color, Color, char) {
        let bg = self.objects.first().unwrap().to_bg_color();
        let (fg, symbol) = self.objects.last().unwrap().to_fg_symbol();
        (bg, fg, symbol)
    }

    // pub fn len(&self) -> usize {
    //     self.objects.len()
    // }

    pub fn iter(&self) -> std::slice::Iter<'_, Object> {
        self.objects.iter()
    }

    pub fn append(&mut self, object: Object) {
        self.objects.push(object);
        self.invariant();
    }

    // /// Used for things like mutating terrain.
    // pub fn replace(&mut self, index: usize, object: Object) {
    //     self.objects[index] = object;
    //     self.invariant();
    // }

    // pub fn remove(&mut self, index: usize) -> Object {
    //     let obj = self.objects.remove(index);
    //     self.invariant();
    //     obj
    // }

    pub fn terrain(&self) -> &Object {
        &self.objects[0]
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        self.objects.iter().any(|obj| obj.has(tag))
    }

    pub fn get(&self, tag: &Tag) -> &Object {
        if let Some(index) = self.objects.iter().position(|obj| obj.has(tag)) {
            &self.objects[index]
        } else {
            panic!("failed to find tag {}", tag);
        }
    }

    pub fn get_mut(&mut self, tag: &Tag) -> &mut Object {
        if let Some(index) = self.objects.iter().position(|obj| obj.has(tag)) {
            &mut self.objects[index]
        } else {
            panic!("failed to find tag {}", tag);
        }
    }

    pub fn remove(&mut self, tag: &Tag) -> Object {
        if let Some(index) = self.objects.iter().position(|obj| obj.has(tag)) {
            let obj = self.objects.remove(index);
            self.invariant();
            obj
        } else {
            panic!("failed to find tag {}", tag);
        }
    }

    pub fn replace(&mut self, tag: &Tag, new_obj: Object) {
        let index = self.objects.iter().position(|obj| obj.has(tag)).unwrap();
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
            self.objects[0].terrain(),
            "First object in a Cell must be a Terrain object: {self}"
        );

        let count = self.objects.iter().filter(|obj| obj.terrain()).count();
        assert!(
            count == 1,
            "There must be only one terrain object in a Cell: {self}"
        );

        let count = self.objects.iter().filter(|obj| obj.character()).count();
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
