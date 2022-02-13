use super::*;
use rand::prelude::*;

pub fn level(game: &mut Game, map: &str) {
    let mut loc = Point::origin();
    for ch in map.chars() {
        // TODO: If we keep these level files we may want to add a symbol
        // mapping section so that characters can do things like refer to
        // different uniques.
        match ch {
            ' ' => game.init_cell(loc, dirt()),
            '#' => game.init_cell(loc, stone_wall()),
            'M' => game.init_cell(loc, metal_wall()),
            '+' => game.init_cell(loc, closed_door()),
            '~' => game.init_cell(loc, shallow_water()),
            'V' => game.init_cell(loc, vitr()),
            'T' => game.init_cell(loc, tree()),
            'W' => game.init_cell(loc, deep_water()),
            'P' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, player());
            }
            'D' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, doorman());
            }
            'o' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, spectator());
            }
            'R' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, rhulad());
            }
            's' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, weak_sword(game));
            }
            'p' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, pick_axe());
            }
            'S' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, mighty_sword());
            }
            'a' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, sign("a sign that says 'the Lesser Armory'"));
            }
            'b' => {
                game.init_cell(loc, dirt());
                game.init_cell(loc, sign("a sign that says 'the Greater Armory'"));
            }
            '\n' => (),
            _ => {
                game.init_cell(loc, dirt());
                game.messages.push(Message {
                    topic: Topic::Error,
                    text: format!("Ignoring map char '{ch}'"),
                });
            }
        }
        if ch == '\n' {
            loc = Point::new(0, loc.y + 1);
        } else {
            loc = Point::new(loc.x + 1, loc.y);
        }
    }
}

pub fn dirt() -> Object {
    Object::new(
        "dirt",
        ground_tags(Color::Black),
        Symbol::Dirt,
        Color::LightSlateGray,
        "a patch of dirt",
    )
}

pub fn rubble() -> Object {
    Object::new(
        "rubble",
        ground_tags(Color::Black),
        Symbol::Rubble,
        Color::Chocolate,
        "a destroyed wall",
    )
}

pub fn stone_wall() -> Object {
    Object::new(
        "stone wall",
        wall_tags(Color::Black, Material::Stone),
        Symbol::Wall,
        Color::Chocolate,
        "a stone wall",
    )
}

pub fn metal_wall() -> Object {
    Object::new(
        "metal wall",
        wall_tags(Color::Black, Material::Metal),
        Symbol::Wall,
        Color::Silver,
        "a metal wall",
    )
}

pub fn tree() -> Object {
    Object::new("tree", tree_tags(), Symbol::Tree, Color::ForestGreen, "a tree")
}

pub fn closed_door() -> Object {
    Object::new(
        "closed door",
        door_tags(Color::Black, Material::Stone, false),
        Symbol::ClosedDoor,
        Color::Yellow,
        "a closed door",
    )
}

pub fn open_door() -> Object {
    Object::new(
        "open door",
        door_tags(Color::Black, Material::Stone, true),
        Symbol::OpenDoor,
        Color::Yellow,
        "an open door",
    )
}

pub fn shallow_water() -> Object {
    Object::new(
        "shallow water",
        shallow_water_tags(),
        Symbol::ShallowLiquid,
        Color::Blue,
        "shallow water",
    )
}

pub fn deep_water() -> Object {
    Object::new(
        "deep water",
        deep_water_tags(),
        Symbol::DeepLiquid,
        Color::Blue,
        "deep water",
    )
}

pub fn vitr() -> Object {
    Object::new(
        "vitr",
        vitr_tags(),
        Symbol::DeepLiquid,
        Color::Gold,
        "a pool of chaotic acid",
    )
}

fn doorman() -> Object {
    Object::new(
        "doorman",
        npc_tags("Doorman", Tag::Doorman),
        Symbol::Npc('D'),
        Color::Green,
        "a royal guard",
    )
}

fn rhulad() -> Object {
    Object::new(
        "rhulad",
        npc_tags("Rhulad", Tag::Rhulad),
        Symbol::Npc('R'),
        Color::Red,
        "the Emperor of a Thousand Deaths",
    )
}

fn spectator() -> Object {
    Object::new(
        "spectator",
        npc_tags("Spectator", Tag::Spectator),
        Symbol::Npc('s'),
        Color::Plum,
        "a spectator",
    )
}

fn player() -> Object {
    Object::new("player", player_tags(), Symbol::Player, Color::Gold, "yourself")
}

pub fn sign(text: &'static str) -> Object {
    Object::new("sign", sign_tags(), Symbol::Sign, Color::Pink, text)
}

pub fn emp_sword() -> Object {
    Object::new(
        "emp sword",
        emp_sword_tags(),
        Symbol::StrongSword,
        Color::Silver,
        "the Sword of the Crippled God",
    )
}

pub fn weak_sword(game: &Game) -> Object {
    let swords = vec![
        ("long sword", "a nicked long sword"),
        ("broadsword", "a dull broadsword"),
        ("long knife", "a shiny long knife"),
        ("dagger", "a pointy dagger"),
    ];
    let sword = swords.iter().choose(&mut *game.rng()).unwrap();
    Object::new(
        "weak sword",
        weak_sword_tags(sword.0),
        Symbol::WeakSword,
        Color::Silver,
        sword.1,
    )
}

pub fn mighty_sword() -> Object {
    Object::new(
        "mighty sword",
        mighty_sword_tags(),
        Symbol::StrongSword,
        Color::Silver,
        "the Sword of Impending Doom",
    )
}

pub fn pick_axe() -> Object {
    Object::new("pick-axe", pick_axe_tags(), Symbol::PickAxe, Color::Tan, "a pick-axe")
}

fn ground_tags(bg: Color) -> Vec<Tag> {
    vec![Tag::Ground, Tag::Background(bg), Tag::Terrain]
}

fn shallow_water_tags() -> Vec<Tag> {
    vec![
        Tag::ShallowWater,
        Tag::Background(Color::LightBlue),
        Tag::Terrain,
        Tag::Scheduled,
    ]
}

fn deep_water_tags() -> Vec<Tag> {
    vec![
        Tag::DeepWater,
        Tag::Background(Color::LightBlue),
        Tag::Terrain,
        Tag::Scheduled,
    ]
}

fn vitr_tags() -> Vec<Tag> {
    vec![Tag::Vitr, Tag::Background(Color::Black), Tag::Terrain]
}

fn wall_tags(bg: Color, material: Material) -> Vec<Tag> {
    let durability = 5 * to_durability(material); // walls are quite a bit tougher than something like a door
    vec![
        Tag::Durability(Durability {
            current: durability,
            max: durability,
        }),
        Tag::Material(material),
        Tag::Wall,
        Tag::Background(bg),
        Tag::Terrain,
    ]
}

fn tree_tags() -> Vec<Tag> {
    vec![Tag::Tree, Tag::Background(Color::Black), Tag::Terrain]
}

fn door_tags(bg: Color, material: Material, open: bool) -> Vec<Tag> {
    let durability = to_durability(material);
    vec![
        Tag::Durability(Durability {
            current: durability,
            max: durability,
        }),
        Tag::Material(material),
        if open { Tag::OpenDoor } else { Tag::ClosedDoor },
        Tag::Background(bg),
        Tag::Terrain,
    ]
}

fn npc_tags(name: &'static str, tag: Tag) -> Vec<Tag> {
    vec![
        Tag::Character,
        // Tag::Inventory(Vec::new()),
        Tag::Name(name),
        tag,
    ]
}

fn player_tags() -> Vec<Tag> {
    vec![
        Tag::Character,
        Tag::Durability(Durability { current: 100, max: 100 }),
        Tag::Inventory(Vec::new()),
        Tag::Name("yourself"),
        Tag::CanOpenDoor,
        Tag::Player,
        Tag::Scheduled,
    ]
}

fn weak_sword_tags(name: &'static str) -> Vec<Tag> {
    vec![Tag::Name(name), Tag::Portable]
}

fn mighty_sword_tags() -> Vec<Tag> {
    vec![Tag::Name("Sword of Impending Doom"), Tag::Portable]
}

fn pick_axe_tags() -> Vec<Tag> {
    vec![Tag::Name("a pick-axe"), Tag::PickAxe, Tag::Portable]
}

fn emp_sword_tags() -> Vec<Tag> {
    vec![Tag::Name("Sword of the Crippled God"), Tag::Portable, Tag::EmpSword]
}

fn sign_tags() -> Vec<Tag> {
    vec![Tag::Sign]
}

fn to_durability(material: Material) -> i32 {
    match material {
        // Material::Wood => 10,
        Material::Stone => 100,
        Material::Metal => 1000,
    }
}
