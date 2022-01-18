use super::{Color, Material, Tag, Unique};
use fnv::FnvHashSet;
use std::fmt::{self, Formatter};

/// Objects are a very general concept: they contain state that may be combined
/// in arbitrary ways (e.g. in theory a cobra could be both a Character and a
/// wieldable Weapon). But note that it's the Action objects that encapsulate
/// behavior.
#[derive(Clone, Eq, PartialEq)]
pub struct Object {
    /// Used for logging, error reporting, etc.
    pub dname: String,
    pub tags: Vec<Tag>,
    pub symbol: char,
    pub color: Color,
    pub description: String,
}

// Tag accessors
impl Object {
    pub fn iter(&self) -> std::slice::Iter<Tag> {
        self.tags.iter()
    }

    pub fn replace(&mut self, tag: Tag) {
        let i = tag.to_index();
        let index = self
            .tags
            .iter()
            .position(|candidate| candidate.to_index() == i)
            .unwrap();
        self.tags[index] = tag;
        self.invariant();
    }

    pub fn character(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_character())
    }

    pub fn player(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_player())
    }

    pub fn unique(&self) -> Option<Unique> {
        self.tags.iter().find_map(|tag| tag.as_unique())
    }

    // pub fn emp_sword(&self) -> bool {
    //     self.tags.iter().any(|tag| tag.is_emp_sword())
    // }

    pub fn inventory(&self) -> Option<&Vec<Object>> {
        self.tags.iter().find_map(|tag| tag.as_inventory())
    }

    // We use this instead of inventory_mut to make it easier to call
    // invariant.
    pub fn pick_up(&mut self, item: Object) {
        let inv = self
            .tags
            .iter_mut()
            .find_map(|tag| tag.as_mut_inventory())
            .unwrap();
        inv.push(item);
        self.invariant();
    }

    /// Returns open or closed (or None if there is no door).
    pub fn door(&self) -> Option<bool> {
        if self.tags.iter().any(|tag| tag.is_open_door()) {
            Some(true)
        } else if self.tags.iter().any(|tag| tag.is_closed_door()) {
            Some(false)
        } else {
            None
        }
    }

    pub fn portable(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_portable())
    }

    pub fn sign(&self) -> Option<&String> {
        if self.tags.iter().any(|tag| tag.is_sign()) {
            Some(&self.description)
        } else {
            None
        }
    }

    // pub fn ground(&self) -> bool {
    //     self.tags.iter().any(|tag| tag.is_ground())
    // }

    // pub fn is_shallow_water(&self) -> bool {
    //     self.tags.iter().any(|tag| tag.is_shallow_water())
    // }

    pub fn is_deep_water(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_deep_water())
    }

    pub fn is_vitr(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_vitr())
    }

    pub fn terrain(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_terrain())
    }

    pub fn tree(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_tree())
    }

    pub fn wall(&self) -> bool {
        self.tags.iter().any(|tag| tag.is_wall())
    }

    pub fn background(&self) -> Option<Color> {
        self.tags.iter().find_map(|tag| tag.as_background())
    }

    /// Returns (current max) durability (or None).
    pub fn durability(&self) -> Option<(i32, i32)> {
        self.tags.iter().find_map(|tag| tag.as_durability())
    }

    pub fn material(&self) -> Option<Material> {
        self.tags.iter().find_map(|tag| tag.as_material())
    }

    pub fn name(&self) -> Option<&String> {
        self.tags.iter().find_map(|tag| tag.as_name())
    }

    /// This uses to_index so it will consider tags like Material(Stone) and
    /// Material(Metal) as equal.
    pub fn has(&self, tag: &Tag) -> bool {
        let index = tag.to_index();
        self.tags
            .iter()
            .any(|candidate| candidate.to_index() == index)
    }

    pub fn to_bg_color(&self) -> Color {
        match self.background() {
            Some(color) => color,
            None => panic!("Expected a background tag"),
        }
    }

    pub fn to_fg_symbol(&self) -> (Color, char) {
        (self.color, self.symbol)
    }
}

// Debug support
impl Object {
    #[cfg(debug_assertions)]
    pub fn invariant(&self) {
        assert!(
            !self.description.is_empty(),
            "Must have a description: {self}"
        );
        if self.terrain() {
            assert!(
                self.background().is_some(),
                "Terrain objects must have a Background: {self}",
            );
            assert!(
                !self.character(),
                "Terrain objects cannot also be Characters: {self}",
            );
            assert!(
                !self.portable(),
                "Terrain objects cannot be Portable: {self}",
            );
        }
        if self.door().is_some() {
            if let Some((current, _max)) = self.durability() {
                assert!(
                    current > 0,
                    "Destroyed doors should change to Ground: {self}"
                );
            }
        }
        if self.wall() {
            if let Some((current, _max)) = self.durability() {
                assert!(
                    current > 0,
                    "Destroyed walls should change to Ground: {self}"
                );
            }
        }
        if self.character() {
            assert!(
                self.name().is_some(),
                "Character's must have a name: {self}"
            );
            assert!(
                !self.portable(),
                "Character objects cannot be Portable: {self}",
            );
        }
        if self.player() {
            assert!(self.character(), "Player must be a Character: {self}")
        }
        if self.portable() {
            assert!(
                self.name().is_some(),
                "Portable objects must have a Name: {self}"
            )
        }

        let mut indexes = FnvHashSet::default();
        for tag in &self.tags {
            let index = tag.to_index();
            assert!(
                !indexes.contains(&index),
                "'{}' has duplicate tags: {self}",
                self.dname
            );
            indexes.insert(index);
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dname)
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tags: Vec<String> = self.tags.iter().map(|tag| format!("{tag}")).collect();
        let tags = tags.join(", ");
        write!(f, "dname: {} tags: {}", self.dname, tags)
    }
}
