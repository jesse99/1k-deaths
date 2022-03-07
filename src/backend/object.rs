use super::tag::*;
use super::{Color, Material, Message, Oid, Tag, Time, Topic};
#[cfg(debug_assertions)]
use fnv::FnvHashSet;
use std::fmt::{self, Formatter};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Symbol {
    ClosedDoor,
    DeepLiquid,
    Dirt,
    Npc(char),
    OpenDoor,
    PickAxe,
    Player,
    Rubble,
    ShallowLiquid,
    Sign,
    StrongSword,
    Tree,
    Unseen,
    Wall,
    WeakSword,
}

// TODO: Should define a custom Clone for Object (and probably Tag) because stuff like
// InventoryTag won't clone properly. Not sure if this should do a deep clone or assert.

/// Objects are a very general concept: they contain state that may be combined
/// in arbitrary ways (e.g. in theory a cobra could be both a Character and a
/// wieldable Weapon). But note that it's the Action objects that encapsulate
/// behavior.
#[derive(Clone, Eq, PartialEq)]
pub struct Object {
    /// Used for logging, error reporting, etc.
    dname: &'static str,

    tags: Vec<Tag>,
    symbol: Symbol,
    color: Color,

    // If we ever need a dynamic string we can continue to optimize for the common case
    // by using a special static str to cause a DyanmicDesc tag to be used instead (or
    // maybe just use that tag if it's present).
    description: &'static str,
}

impl Object {
    pub fn new(dname: &'static str, description: &'static str, symbol: Symbol, color: Color, tags: Vec<Tag>) -> Object {
        Object {
            dname,
            tags,
            symbol,
            color,
            description,
        }
    }

    pub fn dname(&self) -> &'static str {
        &self.dname
    }

    pub fn description(&self) -> &'static str {
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

    pub fn blocks_los(&self) -> bool {
        match self.terrain_value().unwrap_or(Terrain::ShallowWater) {
            Terrain::ClosedDoor => true,
            Terrain::DeepWater => false,
            Terrain::Ground => false,
            Terrain::OpenDoor => false,
            Terrain::Rubble => false,
            Terrain::ShallowWater => false,
            Terrain::Tree => true,
            Terrain::Vitr => false,
            Terrain::Wall => true,
        }
    }

    pub fn to_bg_color(&self) -> Color {
        self.background_value().expect("Expected a Background tag")
    }

    pub fn to_fg_symbol(&self) -> (Color, Symbol) {
        (self.color, self.symbol)
    }

    pub fn impassible_terrain(&self, obj: &Object) -> Option<Message> {
        let terrain = obj.terrain_value().unwrap();
        obj.impassible_terrain_type(terrain)
    }

    pub fn impassible_terrain_type(&self, terrain: Terrain) -> Option<Message> {
        match terrain {
            Terrain::ClosedDoor if !self.has(CAN_OPEN_DOOR_ID) => {
                Some(Message::new(Topic::Failed, "You fail to open the door."))
            }
            Terrain::ClosedDoor => None,
            Terrain::DeepWater => Some(Message::new(Topic::Failed, "The water is too deep.")),
            Terrain::Ground => None,
            Terrain::OpenDoor => None,
            Terrain::Rubble => None,
            Terrain::ShallowWater => None,
            Terrain::Tree => Some(Message::new(
                Topic::Failed,
                "The tree's are too thick to travel through.",
            )),
            Terrain::Vitr => Some(Message::new(Topic::Failed, "Do you have a death wish?")),
            Terrain::Wall => Some(Message::new(Topic::Failed, "You bump into the wall.")),
        }
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

            let terrain = self.terrain_value().unwrap();
            if terrain == Terrain::ClosedDoor {
                if let Some(durability) = self.durability_value() {
                    assert!(
                        durability.current > 0,
                        "Destroyed doors should change to Ground: {self:?}"
                    );
                }
            } else if terrain == Terrain::Wall {
                if let Some(durability) = self.durability_value() {
                    assert!(
                        durability.current > 0,
                        "Destroyed walls should change to Ground: {self:?}"
                    );
                }
            }
        }
        if self.has(CHARACTER_ID) {
            assert!(self.has(NAME_ID), "Character's must have a name: {self:?}");
            assert!(!self.has(PORTABLE_ID), "Character objects cannot be Portable: {self:?}",);

            // This way the interactions table will find a tag for a particular NPC before
            // using the generic Character tag.
            assert!(
                self.tags.last().unwrap().to_id() == CHARACTER_ID,
                "Character tag must come last: {self:?}",
            );
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

// Generated by build.rs, will be at a path like ./target/debug/build/one-thousand-deaths-f4f54e60e59b18ad/out/obj.rs
// It contains functions to extract the value for Tag's that have values. The functions look like this:
// pub fn damage_value(obj: &Object) -> Option<i32>
// pub fn inventory_value(obj: &Object) -> Option<&Vec<Oid>> {
// pub fn inventory_value_mut(obj: &mut Object) -> Option<&mut Vec<Oid>> {
include!(concat!(env!("OUT_DIR"), "/obj.rs"));
