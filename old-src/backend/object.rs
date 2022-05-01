use super::tag::*;
use super::{Color, Material, Message, Oid, Tag, Time, Topic};
use enum_map::EnumMap;
#[cfg(debug_assertions)]
use fnv::FnvHashSet;
use std::fmt::{self, Formatter};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Symbol {
    Armor,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectName {
    // Armor
    LeatherChest,
    LeatherGloves,
    LeatherHat,
    LeatherLegs,
    LeatherSandals,

    // Misc Items
    GreaterArmorySign,
    LesserArmorySign,
    PickAxe,

    // NPCs
    BerokeSoftVoice,
    Doorman,
    Guard,
    HaladRackBearer,
    Icarium,
    ImrothTheCruel,
    KahlbTheSilentHunter,
    Player,
    Rhulad,
    SiballeTheUnfound,
    Spectator,
    ThenikTheShattered,
    UrugalTheWoven,

    // Terrain
    ClosedDoor,
    DeepWater,
    Dirt,
    MetalWall,
    OpenDoor,
    Rubble,
    ShallowWater,
    StoneWall,
    Tree,
    Vitr,

    // Weapons
    Dagger,
    Broadsword,
    EmperorSword,
    LongKnife,
    LongSword,
    MightySword,
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
    name: ObjectName,

    tags: Vec<Tag>,
    symbol: Symbol,
    color: Color,

    // If we ever need a dynamic string we can continue to optimize for the common case
    // by using a special static str to cause a DyanmicDesc tag to be used instead (or
    // maybe just use that tag if it's present).
    description: &'static str,
}

impl Object {
    pub fn new(name: ObjectName, description: &'static str, symbol: Symbol, color: Color, tags: Vec<Tag>) -> Object {
        Object {
            name,
            tags,
            symbol,
            color,
            description,
        }
    }

    pub fn dname(&self) -> String {
        format!("{:?}", self.name)
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

        if cfg!(debug_assertions) {
            self.invariant();
        }
    }

    // We use this instead of as_mut_ref to make it easier to call the invariant.
    // pub fn pick_up(&mut self, item: Object) {
    //     let inv = self.as_mut_ref(INVENTORY_ID).unwrap();
    //     inv.push(item);
    //     self.invariant();
    // }

    pub fn has(&self, tid: Tid) -> bool {
        self.tags.iter().any(|candidate| candidate.to_id() == tid)
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

            let disp = self.disposition_value();
            if matches!(disp, Some(Disposition::Neutral) | Some(Disposition::Aggressive)) {
                assert!(
                    self.has(DURABILITY_ID),
                    "Character's the player can fight should be mortal: {self:?}"
                );
                assert!(
                    self.has(SCHEDULED_ID),
                    "Character's the player can fight should be able to fight back: {self:?}"
                );
            }

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
        if self.has(ARMOR_ID) {
            assert!(self.has(PORTABLE_ID), "Armor objects must be portable: {self:?}");
            assert!(self.has(MITIGATION_ID), "Armor objects must mitigate damage: {self:?}");
        }
        if self.has(WEAPON_ID) {
            assert!(self.has(PORTABLE_ID), "Weapon objects must be portable: {self:?}");
            assert!(self.has(DAMAGE_ID), "Weapon objects must cause damage: {self:?}");
            assert!(self.has(DELAY_ID), "Weapon objects must have a delay: {self:?}");
        }
        if self.has(PORTABLE_ID) {
            assert!(self.has(NAME_ID), "Portable objects must have a Name: {self:?}");
        }

        if self.has(DAMAGE_ID) {
            assert!(self.has(DELAY_ID), "Damage tags must also have a delay tag: {self:?}");
        }

        let mut ioids = FnvHashSet::default();
        if let Some(inv) = self.inventory_value() {
            for oid in inv {
                assert!(
                    !ioids.contains(&oid),
                    "'{}' has duplicate inventory oid {oid}",
                    self.dname()
                );
                ioids.insert(oid);
            }
        }

        if let Some(equipped) = self.equipped_value() {
            let mut oids = FnvHashSet::default();
            for value in equipped.values() {
                if let Some(oid) = value {
                    assert!(!ioids.contains(&oid), "'{}' has {oid} in both inv and eq", self.dname());
                    assert!(!oids.contains(&oid), "'{}' has duplicate eq oid {oid}", self.dname());
                    oids.insert(oid);
                }
            }
        }

        let mut ids = FnvHashSet::default();
        for tag in &self.tags {
            let id = tag.to_id();
            assert!(!ids.contains(&id), "'{}' has duplicate tags: {self:?}", self.dname());
            ids.insert(id);
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dname())
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tags: Vec<String> = self.tags.iter().map(|tag| format!("{tag}")).collect();
        let tags = tags.join(", ");
        write!(f, "dname: {} tags: {}", self.dname(), tags)
    }
}

// Generated by build.rs, will be at a path like ./target/debug/build/one-thousand-deaths-f4f54e60e59b18ad/out/obj.rs
// It contains functions to extract the value for Tag's that have values. The functions look like this:
// pub fn damage_value(obj: &Object) -> Option<i32>
// pub fn inventory_value(obj: &Object) -> Option<&Vec<Oid>> {
// pub fn inventory_value_mut(obj: &mut Object) -> Option<&mut Vec<Oid>> {
include!(concat!(env!("OUT_DIR"), "/obj.rs"));
