use super::{Color, Oid, Point, Time};
use derive_more::Display;
use std::fmt::{self, Formatter};

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

// TODO: generate this file (Display trait may require some sort of escape hatch)
// TODO: can we also generate something like the Object value methods?

/// Object state and properties consist of a list of these tags. Objects can
/// be classified as Terrain, Weapon, etc but note that this is a fuzzy
/// concept because those classes can be combined.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tag {
    /// Player, monsters, special entities. Triggers an interaction when players try to
    /// move into them. These will have a Name tag. Often they will also have Scheduled,
    /// and CanOpenDoor tags. NPCs will also have Behavior, Damage, Disposition,
    /// Durability, Flees, Hearing, and Inventory tags.
    Character,

    Player,
    Doorman, // TODO: might want to use a UniqueNpc for these
    Guard,
    Icarium,
    Rhulad,
    Spectator,

    /// Present for objects that perform actions using the Scheduler.
    Scheduled,

    /// This is typically a base damage and is scaled by things like skill and strength.
    Damage(i32),

    /// Objects that a Character has picked up.
    Inventory(Vec<Oid>),

    /// Used for Characters that start fleeing when their HPs is at the specified percent.
    Flees(i32), // TODO: should this be smarter? or maybe a second type of flee tag that considers both attacker and defender HPs

    /// Scaling factor applied to the probability of responding to noise. 100 is no scaling,
    /// 120 is 20% more likely, and 80 is 20% less likely.
    Hearing(i32),

    CanOpenDoor,

    /// The object is something that can be picked up and placed into a
    /// Character's inventory.
    Portable,

    /// Can be used to dig through wood or stone structures (i.e. doors and
    /// walls). Ineffective against metal.
    PickAxe,

    /// Description will have the sign's message.
    Sign,
    EmpSword, // TODO: do we want UniqueNPC and UniqueItem?

    /// Used for objects that are the lowest layer in a Cell, e.g. grassy ground.
    /// Note that this can be used for unusual objects such as a ballista. Will
    /// have a Background tag.
    Terrain(Terrain),

    /// Normally only used with Terrain.
    Background(Color),

    Disposition(Disposition),
    Behavior(Behavior),

    /// Typically at zero durability an object will change somehow, e.g. a
    /// door will become open or a character will die.
    Durability(Durability),

    /// Used for some terrain objects, e.g. walls and doors.
    Material(Material),

    /// Characters and portable objects all have names.
    Name(&'static str),
}

// Unlike Object id's tag id's don't typically hang around for very long. So I think it's
// fine to simply make them a u16 rather than something more intelligible.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq)]
pub struct Tid(u16);

pub const CHARACTER_ID: Tid = Tid(0);
pub const PLAYER_ID: Tid = Tid(1);
pub const DOORMAN_ID: Tid = Tid(2);
pub const RHULAD_ID: Tid = Tid(3);
pub const SPECTATOR_ID: Tid = Tid(4);
pub const INVENTORY_ID: Tid = Tid(5);
pub const PORTABLE_ID: Tid = Tid(6);
pub const PICK_AXE_ID: Tid = Tid(7);
pub const SIGN_ID: Tid = Tid(8);
pub const EMP_SWORD_ID: Tid = Tid(9);
pub const TERRAIN_ID: Tid = Tid(16);
pub const BACKGROUND_ID: Tid = Tid(19);
pub const DURABILITY_ID: Tid = Tid(20);
pub const MATERIAL_ID: Tid = Tid(21);
pub const NAME_ID: Tid = Tid(22);
pub const CAN_OPEN_DOOR_ID: Tid = Tid(23);
pub const SCHEDULED_ID: Tid = Tid(24);
pub const DISPOSITION_ID: Tid = Tid(25);
pub const ICARIUM_ID: Tid = Tid(26);
pub const BEHAVIOR_ID: Tid = Tid(27);
pub const GUARD_ID: Tid = Tid(28);
pub const FLEES_ID: Tid = Tid(29);
pub const HEARING_ID: Tid = Tid(30);
pub const DAMAGE_ID: Tid = Tid(31);

impl Tag {
    pub fn to_id(&self) -> Tid {
        match self {
            Tag::Character => CHARACTER_ID,
            Tag::Player => PLAYER_ID,
            Tag::Doorman => DOORMAN_ID,
            Tag::Rhulad => RHULAD_ID,
            Tag::Spectator => SPECTATOR_ID,
            Tag::Inventory(_) => INVENTORY_ID,
            Tag::Portable => PORTABLE_ID,
            Tag::EmpSword => EMP_SWORD_ID,
            Tag::PickAxe => PICK_AXE_ID,
            Tag::Sign => SIGN_ID,
            Tag::Terrain(_) => TERRAIN_ID,
            Tag::Background(_) => BACKGROUND_ID,
            Tag::Durability(_) => DURABILITY_ID,
            Tag::Material(_) => MATERIAL_ID,
            Tag::Name(_) => NAME_ID,
            Tag::CanOpenDoor => CAN_OPEN_DOOR_ID,
            Tag::Scheduled => SCHEDULED_ID,
            Tag::Disposition(_) => DISPOSITION_ID,
            Tag::Icarium => ICARIUM_ID,
            Tag::Guard => GUARD_ID,
            Tag::Behavior(_) => BEHAVIOR_ID,
            Tag::Flees(_) => FLEES_ID,
            Tag::Hearing(_) => HEARING_ID,
            Tag::Damage(_) => DAMAGE_ID,
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Character => write!(f, "Character"),
            Tag::Player => write!(f, "Player"),
            Tag::Doorman => write!(f, "Doorman"),
            Tag::Rhulad => write!(f, "Rhulad"),
            Tag::Spectator => write!(f, "Spectator"),
            Tag::Inventory(_) => write!(f, "Inventory"),
            Tag::Portable => write!(f, "Portable"),
            Tag::EmpSword => write!(f, "EmpSword"),
            Tag::PickAxe => write!(f, "PickAxe"),
            Tag::Sign => write!(f, "Sign"),
            Tag::Terrain(t) => write!(f, "Terrain({t})"),
            Tag::Background(color) => write!(f, "Background({color})"),
            Tag::Durability(durability) => write!(f, "Durability({}, {})", durability.current, durability.max),
            Tag::Material(material) => write!(f, "Material({material})"),
            Tag::Name(text) => write!(f, "Name({text})"),
            Tag::CanOpenDoor => write!(f, "CanOpenDoor"),
            Tag::Scheduled => write!(f, "Scheduled"),
            Tag::Disposition(dis) => write!(f, "Disposition({dis})"),
            Tag::Icarium => write!(f, "Icarium"),
            Tag::Guard => write!(f, "Guard"),
            Tag::Behavior(b) => write!(f, "Behavior({b})"),
            Tag::Flees(p) => write!(f, "Flees({p})"),
            Tag::Hearing(p) => write!(f, "Hearing({p})"),
            Tag::Damage(p) => write!(f, "Damage({p})"),
        }
    }
}

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
