use super::{Color, Liquid, Tag};
use fnv::FnvHashSet;
use std::fmt::{self, Formatter};

/// Objects are a very general concept: they contain state that may be combined
/// in arbitrary ways (e.g. in theory a cobra could be both a Character and a
/// wieldable Weapon). But note that it's the Action objects that encapsulate
/// behavior.
#[derive(Clone)]
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
    pub fn character(&self) -> bool {
        for tag in &self.tags {
            if let Tag::Character = tag {
                return true;
            }
        }
        false
    }

    pub fn player(&self) -> bool {
        for tag in &self.tags {
            if let Tag::Player = tag {
                return true;
            }
        }
        false
    }

    /// Returns open or closed (or None if there is no door).
    pub fn door(&self) -> Option<bool> {
        for tag in &self.tags {
            if let Tag::OpenDoor = tag {
                return Some(true);
            } else if let Tag::ClosedDoor = tag {
                return Some(false);
            }
        }
        None
    }

    pub fn portable(&self) -> bool {
        for tag in &self.tags {
            if let Tag::Portable = tag {
                return true;
            }
        }
        false
    }

    pub fn sign(&self) -> Option<String> {
        for tag in &self.tags {
            if let Tag::Sign = tag {
                return Some(self.description.clone());
            }
        }
        None
    }

    pub fn ground(&self) -> bool {
        for tag in &self.tags {
            if let Tag::Ground = tag {
                return true;
            }
        }
        false
    }

    /// Returns (Liquid, deep) (or None if there's no liquid).
    pub fn liquid(&self) -> Option<(Liquid, bool)> {
        for tag in &self.tags {
            if let Tag::Liquid { liquid, deep } = tag {
                return Some((*liquid, *deep));
            }
        }
        None
    }

    pub fn terrain(&self) -> bool {
        for tag in &self.tags {
            if let Tag::Terrain = tag {
                return true;
            }
        }
        false
    }

    pub fn wall(&self) -> bool {
        for tag in &self.tags {
            if let Tag::Wall = tag {
                return true;
            }
        }
        false
    }

    pub fn background(&self) -> Option<Color> {
        for tag in &self.tags {
            if let Tag::Background(bg) = tag {
                return Some(*bg);
            }
        }
        None
    }

    /// Returns (current max) durability (or None).
    pub fn durability(&self) -> Option<(i32, i32)> {
        for tag in &self.tags {
            if let Tag::Durability { current, max } = tag {
                return Some((*current, *max));
            }
        }
        None
    }

    // pub fn material(&self) -> Option<Material> {
    //     for tag in &self.tags {
    //         if let Tag::Material(material) = tag {
    //             return Some(*material);
    //         }
    //     }
    //     None
    // }

    pub fn name(&self) -> Option<String> {
        for tag in &self.tags {
            if let Tag::Name(name) = tag {
                return Some(name.clone());
            }
        }
        None
    }

    /// This uses to_index so it will consider tags like Material(Stone) and
    /// Material(Metal) as equal.
    pub fn has(&self, tag: &Tag) -> bool {
        let index = to_index(tag);
        self.tags
            .iter()
            .any(|candidate| to_index(candidate) == index)
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
            // TODO: terrain cannot have the Portable tag
            assert!(
                self.background().is_some(),
                "Terrain objects must have a Background: {self}",
            );
            assert!(
                !self.character(),
                "Terrain objects cannot also be Characters: {self}",
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
            )
        }
        if self.player() {
            assert!(self.character(), "Player must be a Character: {self}")
        }
        // TODO: portable must have a name

        let mut indexes = FnvHashSet::default();
        for tag in &self.tags {
            let index = to_index(tag);
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

// TODO: Could use enum_index instead although that does require that variant
// values implement the Default trait.
fn to_index(tag: &Tag) -> i32 {
    match tag {
        Tag::Character => 1,
        Tag::Player => 2,

        Tag::Portable => 3,
        Tag::Sign => 4,

        Tag::ClosedDoor => 5,
        Tag::Ground => 6,
        Tag::Liquid { liquid: _, deep: _ } => 7,
        Tag::OpenDoor => 8,
        Tag::Terrain => 9,
        Tag::Tree => 10,
        Tag::Wall => 11,

        Tag::Background(_bg) => 12,
        Tag::Durability { current: _, max: _ } => 13,
        Tag::Material(_material) => 14,
        Tag::Name(_name) => 15,
    }
}
