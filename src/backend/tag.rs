use super::{Color, Object};
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

/// Object state and properties consist of a list of these tags. Objects can
/// be classified as Terrain, Weapon, etc but note that this is a fuzzy
/// concept because those classes can be combined.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tag {
    /// Player, monsters, special entities. Triggers an interaction when
    /// players try to move into them. Will also have a Name tag. May have
    /// an Inventory tag.
    Character,
    /// Will also have Character and Inventory tags.
    Player,
    Doorman,
    Rhulad,
    Spectator,
    /// Objects that a Character has picked up.
    Inventory(Vec<Object>),

    /// The object is something that can be picked up and placed into a
    /// Character's inventory.
    Portable,
    /// Can be used to dig through wood or stone structures (i.e. doors and
    /// walls). Ineffective against metal.
    PickAxe,
    /// Description will have the sign's message.
    Sign,
    EmpSword, // TODO: do we want UniqueNPC and UniqueItem?

    /// Normally also has a terrain tag.
    /// Will have Durability (and usually Material) if the door can be broken down.
    /// If it has a Binding tag then it can only be opened by characters that
    /// have a matching Binding object in their inventory (i.e. a key).
    ClosedDoor,
    /// Each level will have one of these. Will also have the Character tag.
    /// Grass, dirt, etc. Will have a Terrain tag,
    Ground,
    /// Will have a Terrain tag,
    ShallowWater,
    /// Will have a Terrain tag,
    DeepWater,
    /// Will have a Terrain tag,
    Vitr,
    /// Normally also has a terrain tag. This will also share tags with
    /// ClosedDoor so that they can be preserved as doors transition from
    /// open to closed.
    OpenDoor,
    /// Used for objects that are the lowest layer in a Cell, e.g. grassy ground.
    /// Note that this can be used for unusual objects such as a ballista. Will
    /// have a Background tag.
    Terrain,
    /// Will have a terrain tag. TODO: may want Material and Durability but
    /// burnt trees should probably remain impassible
    Tree,
    /// Will have a terrain tag and normally Durability and Material tags.
    /// At zero durability the wall is broken through.
    Wall,

    /// Normally only used with Terrain.
    Background(Color),
    /// Typically at zero durability an object will change somehow, e.g. a
    /// door will become open or a character will die.
    Durability {
        current: i32,
        max: i32,
    },
    /// Used for some terrain objects, e.g. walls and doors.
    Material(Material),
    /// Characters and portable objects all have names.
    Name(String),
}

impl Tag {
    pub fn is_character(&self) -> bool {
        matches!(self, Tag::Character)
    }

    pub fn is_player(&self) -> bool {
        matches!(self, Tag::Player)
    }
    pub fn is_portable(&self) -> bool {
        matches!(self, Tag::Portable)
    }
    pub fn is_sign(&self) -> bool {
        matches!(self, Tag::Sign)
    }
    pub fn is_closed_door(&self) -> bool {
        matches!(self, Tag::ClosedDoor)
    }
    pub fn is_open_door(&self) -> bool {
        matches!(self, Tag::OpenDoor)
    }
    pub fn is_terrain(&self) -> bool {
        matches!(self, Tag::Terrain)
    }
    pub fn is_wall(&self) -> bool {
        matches!(self, Tag::Wall)
    }

    pub fn as_inventory(&self) -> Option<&Vec<Object>> {
        match self {
            Tag::Inventory(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_mut_inventory(&mut self) -> Option<&mut Vec<Object>> {
        match self {
            Tag::Inventory(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_background(&self) -> Option<Color> {
        match *self {
            Tag::Background(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_durability(&self) -> Option<(i32, i32)> {
        match *self {
            Tag::Durability { current, max } => Some((current, max)),
            _ => None,
        }
    }
    pub fn as_material(&self) -> Option<Material> {
        match *self {
            Tag::Material(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_name(&self) -> Option<&String> {
        match self {
            Tag::Name(result) => Some(result),
            _ => None,
        }
    }

    // TODO: Could use enum_index instead although that does require that variant
    // values implement the Default trait.
    pub fn to_index(&self) -> i32 {
        match self {
            Tag::Character => 1,
            Tag::Player => 2,
            Tag::Doorman => 3,
            Tag::Rhulad => 4,
            Tag::Spectator => 5,
            Tag::Inventory(_) => 6,

            Tag::Portable => 7,
            Tag::Sign => 8,
            Tag::EmpSword => 9,
            Tag::PickAxe => 10,

            Tag::ClosedDoor => 11,
            Tag::Ground => 12,
            Tag::ShallowWater => 13,
            Tag::DeepWater => 14,
            Tag::Vitr => 15,
            Tag::OpenDoor => 16,
            Tag::Terrain => 17,
            Tag::Tree => 18,
            Tag::Wall => 19,

            Tag::Background(_bg) => 20,
            Tag::Durability { current: _, max: _ } => 21,
            Tag::Material(_material) => 22,
            Tag::Name(_name) => 23,
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
            Tag::ClosedDoor => write!(f, "ClosedDoor"),
            Tag::Ground => write!(f, "Ground"),
            Tag::ShallowWater => write!(f, "ShallowWater"),
            Tag::DeepWater => write!(f, "DeepWater"),
            Tag::Vitr => write!(f, "Vitr"),
            Tag::OpenDoor => write!(f, "OpenDoor"),
            Tag::Terrain => write!(f, "Terrain"),
            Tag::Tree => write!(f, "Tree"),
            Tag::Wall => write!(f, "Wall"),
            Tag::Background(color) => write!(f, "Background({color})"),
            Tag::Durability { current, max } => write!(f, "Durability({current}, {max})"),
            Tag::Material(material) => write!(f, "Material({material})"),
            Tag::Name(text) => write!(f, "Name({text})"),
        }
    }
}
