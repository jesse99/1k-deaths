use super::{Color, ObjId};
use derive_more::Display;
use std::fmt::{self, Formatter};

/// Affects behavior of items like burning oil or a pick axe. Also affects
/// spell behavior and whether characters can move through terrain.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Serialize, Deserialize)]
pub enum Material {
    // Wood,
    Stone,
    Metal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Durability {
    pub current: i32,
    pub max: i32,
}

// TODO: generate this file (Display trait may require some sort of escape hatch)

/// Object state and properties consist of a list of these tags. Objects can
/// be classified as Terrain, Weapon, etc but note that this is a fuzzy
/// concept because those classes can be combined.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Tag {
    /// Player, monsters, special entities. Triggers an interaction when
    /// players try to move into them. Will also have a Name tag. May have
    /// an Inventory tag.
    Character,

    /// Will also have Character and Inventory tags. Might have a CanOpenDoor tag.
    Player,
    Doorman,
    Rhulad,
    Spectator,

    /// Objects that a Character has picked up.
    Inventory(Vec<ObjId>),

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
    Durability(Durability),

    /// Used for some terrain objects, e.g. walls and doors.
    Material(Material),

    /// Characters and portable objects all have names.
    Name(String),
}

// Unlike Object id's tag id's don't typically hang around for very long. So I think it's
// fine to simply make them a u16 rather than something more intelligible.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TagId(u16);

pub const CHARACTER_ID: TagId = TagId(0);
pub const PLAYER_ID: TagId = TagId(1);
pub const DOORMAN_ID: TagId = TagId(2);
pub const RHULAD_ID: TagId = TagId(3);
pub const SPECTATOR_ID: TagId = TagId(4);
pub const INVENTORY_ID: TagId = TagId(5);
pub const PORTABLE_ID: TagId = TagId(6);
pub const PICK_AXE_ID: TagId = TagId(7);
pub const SIGN_ID: TagId = TagId(8);
pub const EMP_SWORD_ID: TagId = TagId(9);
pub const CLOSED_DOOR_ID: TagId = TagId(10);
pub const GROUND_ID: TagId = TagId(11);
pub const SHALLOW_WATER_ID: TagId = TagId(12);
pub const DEEP_WATER_ID: TagId = TagId(13);
pub const VITR_ID: TagId = TagId(14);
pub const OPEN_DOOR_ID: TagId = TagId(15);
pub const TERRAIN_ID: TagId = TagId(16);
pub const TREE_ID: TagId = TagId(17);
pub const WALL_ID: TagId = TagId(18);
pub const BACKGROUND_ID: TagId = TagId(19);
pub const DURABILITY_ID: TagId = TagId(20);
pub const MATERIAL_ID: TagId = TagId(21);
pub const NAME_ID: TagId = TagId(22);
pub const CAN_OPEN_DOOR_ID: TagId = TagId(23);

impl Tag {
    pub fn to_id(&self) -> TagId {
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
            Tag::ClosedDoor => CLOSED_DOOR_ID,
            Tag::Ground => GROUND_ID,
            Tag::ShallowWater => SHALLOW_WATER_ID,
            Tag::DeepWater => DEEP_WATER_ID,
            Tag::Vitr => VITR_ID,
            Tag::OpenDoor => OPEN_DOOR_ID,
            Tag::Terrain => TERRAIN_ID,
            Tag::Tree => TREE_ID,
            Tag::Wall => WALL_ID,
            Tag::Background(_) => BACKGROUND_ID,
            Tag::Durability(_) => DURABILITY_ID,
            Tag::Material(_) => MATERIAL_ID,
            Tag::Name(_) => NAME_ID,
            Tag::CanOpenDoor => CAN_OPEN_DOOR_ID,
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
            Tag::Durability(durability) => write!(f, "Durability({}, {})", durability.current, durability.max),
            Tag::Material(material) => write!(f, "Material({material})"),
            Tag::Name(text) => write!(f, "Name({text})"),
            Tag::CanOpenDoor => write!(f, "CanOpenDoor"),
        }
    }
}
