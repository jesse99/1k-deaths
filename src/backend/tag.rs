use super::{Color, Oid, Point, Time};
use derive_more::Display;
use enum_map::{Enum, EnumMap};
use std::fmt::{self, Formatter};

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Weapon {
    TwoHander,
    OneHand,
    //Ranged,
}

#[derive(Clone, Copy, Debug, Display, Enum, Eq, PartialEq)]
pub enum Slot {
    MainHand,
    OffHand,
    Head,
    Chest,
    Hands,
    Legs,
    Feet,
}

/// Affects behavior of items like burning oil or a pick axe. Also affects
/// spell behavior and whether characters can move through terrain.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Material {
    // Wood,
    Stone,
    Metal,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Disposition {
    /// Player cannot attack these.
    Friendly,

    /// These act friendly until attacked in which case they turn aggressive.
    Neutral,

    /// These will attack the player on sight.
    Aggressive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Behavior {
    /// NPC is attempting to attack oid at its last known location.
    Attacking(Oid, Point),

    /// NPC is moving towards the point. Typically this is because it heard noise from
    /// there.
    MovingTo(Point),

    /// NPC isn't doing anything but may wake up if there are noises.
    Sleeping,

    /// NPC will wander around until time goes past the specified time.
    Wandering(Time),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Durability {
    pub current: i32,
    pub max: i32,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Terrain {
    /// Will have Durability (and usually Material) if the door can be broken down.
    /// If it has a Binding tag then it can only be opened by characters that
    /// have a matching Binding object in their inventory (i.e. a key).
    ClosedDoor,

    DeepWater,

    /// Grass, dirt, etc.
    Ground,

    OpenDoor,

    /// Will have a Material tag.
    Rubble,

    ShallowWater,

    /// TODO: may want Material and Durability but burnt trees should probably remain impassible
    Tree,

    Vitr,

    /// Will normally have Durability and Material tags. At zero durability changes to Rubble.
    Wall,
}

// Unlike Object id's tag id's don't typically hang around for very long. So I think it's
// fine to simply make them a u16 rather than something more intelligible.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq)]
pub struct Tid(u16);

impl fmt::Display for Behavior {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Behavior::Attacking(oid, pt) => write!(f, "Behavior::Attacking({oid}, {pt})"),
            Behavior::MovingTo(pt) => write!(f, "Behavior::MovingTo({pt})"),
            Behavior::Sleeping => write!(f, "Behavior::Sleeping"),
            Behavior::Wandering(t) => write!(f, "Behavior::Wandering({t})"),
        }
    }
}

// Generated by build.rs, will be at a path like ./target/debug/build/one-thousand-deaths-f4f54e60e59b18ad/out/tag.rs
// It contains:
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub enum Tag {
//     Character,
//     Player,
//     ...
// pub const CHARACTER_ID: Tid = Tid(0);
// pub const PLAYER_ID: Tid = Tid(1);
//     ...
// impl Tag {
//     pub fn to_id(&self) -> Tid {
//     ...
// impl fmt::Display for Tag {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//     ...
include!(concat!(env!("OUT_DIR"), "/tag.rs"));
