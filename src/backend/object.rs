use super::tag::*;
use super::{Color, Material, Oid, Tag};
#[cfg(debug_assertions)]
use fnv::FnvHashSet;
use std::fmt::{self, Formatter};

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Symbol {
    Character(char),
    ClosedDoor,
    DeepLiquid,
    Dirt,
    OpenDoor,
    PickAxe,
    Rubble,
    ShallowLiquid,
    Sign,
    StrongSword,
    Tree,
    Unseen,
    Wall,
    WeakSword,
}

/// Objects are a very general concept: they contain state that may be combined
/// in arbitrary ways (e.g. in theory a cobra could be both a Character and a
/// wieldable Weapon). But note that it's the Action objects that encapsulate
/// behavior.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Object {
    /// Used for logging, error reporting, etc.
    dname: String, // these could be `&'static str` but for Deserialize, TODO: maybe could intern them?

    tags: Vec<Tag>,
    symbol: Symbol,
    color: Color,
    description: String,
}

impl Object {
    pub fn new<S: Into<String>, T: Into<String>>(
        dname: S,
        tags: Vec<Tag>,
        symbol: Symbol,
        color: Color,
        description: T,
    ) -> Object {
        Object {
            dname: dname.into(),
            tags,
            symbol,
            color,
            description: description.into(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn iter(&self) -> std::slice::Iter<Tag> {
        self.tags.iter()
    }

    pub fn replace(&mut self, tag: Tag) {
        let id = tag.to_id();
        let index = self.tags.iter().position(|candidate| candidate.to_id() == id).unwrap();
        self.tags[index] = tag;

        {
            #[cfg(debug_assertions)]
            self.invariant();
        }
    }

    // We use this instead of as_mut_ref to make it easier to call the invariant.
    // pub fn pick_up(&mut self, item: Object) {
    //     let inv = self.as_mut_ref(INVENTORY_ID).unwrap();
    //     inv.push(item);
    //     self.invariant();
    // }

    pub fn has(&self, id: Tid) -> bool {
        self.tags.iter().any(|candidate| candidate.to_id() == id)
    }

    pub fn to_bg_color(&self) -> Color {
        self.value(BACKGROUND_ID).expect("Expected a Background tag")
    }

    pub fn to_fg_symbol(&self) -> (Color, Symbol) {
        (self.color, self.symbol)
    }
}

pub trait TagValue<T> {
    fn value(&self, id: Tid) -> Option<T>;
}

impl TagValue<Color> for Object {
    fn value(&self, id: Tid) -> Option<Color> {
        for candidate in &self.tags {
            if candidate.to_id() == id {
                match candidate {
                    Tag::Background(value) => return Some(*value),
                    _ => panic!("{} tag doesn't have a Color", candidate),
                }
            }
        }
        None
    }
}

impl TagValue<Durability> for Object {
    fn value(&self, id: Tid) -> Option<Durability> {
        for candidate in &self.tags {
            if candidate.to_id() == id {
                match candidate {
                    Tag::Durability(value) => return Some(*value),
                    _ => panic!("{} tag doesn't have a Durability", candidate),
                }
            }
        }
        None
    }
}

impl TagValue<Material> for Object {
    fn value(&self, id: Tid) -> Option<Material> {
        for candidate in &self.tags {
            if candidate.to_id() == id {
                match candidate {
                    Tag::Material(value) => return Some(*value),
                    _ => panic!("{} tag doesn't have a Material", candidate),
                }
            }
        }
        None
    }
}

impl TagValue<String> for Object {
    fn value(&self, id: Tid) -> Option<String> {
        for candidate in &self.tags {
            if candidate.to_id() == id {
                match candidate {
                    Tag::Name(value) => return Some(value.clone()),
                    _ => panic!("{} tag doesn't have a String", candidate),
                }
            }
        }
        None
    }
}

impl Object {
    // TODO: add a trait for these?
    pub fn as_ref(&self, id: Tid) -> Option<&Vec<Oid>> {
        for candidate in self.tags.iter() {
            if candidate.to_id() == id {
                match candidate {
                    Tag::Inventory(value) => return Some(value),
                    _ => panic!("{} tag doesn't have a Vec<Oid>", candidate),
                }
            }
        }
        None
    }

    pub fn as_mut_ref(&mut self, id: Tid) -> Option<&mut Vec<Oid>> {
        for candidate in self.tags.iter_mut() {
            if candidate.to_id() == id {
                match candidate {
                    Tag::Inventory(value) => return Some(value),
                    _ => panic!("{} tag doesn't have a Vec<Oid>", candidate),
                }
            }
        }
        None
    }
}

// Debug support
impl Object {
    #[cfg(debug_assertions)]
    pub fn invariant(&self) {
        assert!(!self.description.is_empty(), "Must have a description: {self:?}");
        if self.has(TERRAIN_ID) {
            assert!(
                self.has(BACKGROUND_ID),
                "Terrain objects must have a Background: {self:?}",
            );
            assert!(
                !self.has(CHARACTER_ID),
                "Terrain objects cannot also be Characters: {self:?}",
            );
            assert!(!self.has(PORTABLE_ID), "Terrain objects cannot be Portable: {self:?}");

            // TODO: May want to add similar checks: one sort of character, one sort of
            // portable.
            let count = self
                .tags
                .iter()
                .filter(|t| {
                    matches!(
                        t,
                        Tag::Wall
                            | Tag::ClosedDoor
                            | Tag::Ground
                            | Tag::ShallowWater
                            | Tag::DeepWater
                            | Tag::Vitr
                            | Tag::OpenDoor
                    )
                })
                .count();
            assert!(count == 1, "Terrain objects must be one sort of terrain: {self:?}");
        }
        if self.has(CLOSED_DOOR_ID) {
            if let Some::<Durability>(durability) = self.value(DURABILITY_ID) {
                assert!(
                    durability.current > 0,
                    "Destroyed doors should change to Ground: {self:?}"
                );
            }
        }
        if self.has(WALL_ID) {
            if let Some::<Durability>(durability) = self.value(DURABILITY_ID) {
                assert!(
                    durability.current > 0,
                    "Destroyed walls should change to Ground: {self:?}"
                );
            }
        }
        if self.has(CHARACTER_ID) {
            assert!(self.has(NAME_ID), "Character's must have a name: {self:?}");
            assert!(!self.has(PORTABLE_ID), "Character objects cannot be Portable: {self:?}",);
        }
        if self.has(PLAYER_ID) {
            assert!(self.has(CHARACTER_ID), "Player must be a Character: {self:?}")
        }
        if self.has(PORTABLE_ID) {
            assert!(self.has(NAME_ID), "Portable objects must have a Name: {self:?}")
        }

        let mut ids = FnvHashSet::default();
        for tag in &self.tags {
            let id = tag.to_id();
            assert!(!ids.contains(&id), "'{}' has duplicate tags: {self:?}", self.dname);
            ids.insert(id);
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
