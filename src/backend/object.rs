use super::Color;

/// Affects behavior of items like burning oil ir a pick axe. Also affects
/// spell behavior and whether characters can move through terrain.
#[derive(Clone, Copy)]
enum Material {
    Wood,
    Stone,
    Metal,
}

/// Object state and properties consist of a list of these tags. Objects can
/// be classified as Terrain, Weapon, etc but note that this is a fuzzy
/// concept because those classes can be combined.
#[derive(Clone)]
enum Tag {
    /// Terrain is normally the only object that includes this.
    BackGround(Color),
    /// Typically this will also be a Terrain object. Doors with a Binding
    /// are locked and can only be opened if the player has an item with a
    /// matching Binding (e.g. a key).
    Door {
        open: bool,
    },
    /// Typically at zero durability an object will change somehow, e.g. a
    /// door will become open or a character will die.
    Durability(i32),
    /// Used for some terrain objects, e.g. walls and doors.
    Material(Material),
    // Used for objects that are the lowest layer in a Cell, e.g. grassy ground.
    Terrain,
}

struct Object {
    name: String,
    description: String,
    symbol: char,
    color: Color,
    tags: Vec<Tag>,
}

// TODO: add a builder to build terrain

// TODO: need a wrapper to ensure consistency:
// - first object must have terrain tag
// - terrain tags must have background color
// - terrain cannot have portable tag
// - setting terrain replaces existing terrain
// - can't have two characters, relax this later?
struct Cell {
    // TODO: move this into its own module
    objects: Vec<Object>,
}
