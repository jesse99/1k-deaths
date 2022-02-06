use super::{Action, Color, Durability, Event, Game, Material, Message, Object, Point, Tag, Topic};
use rand::prelude::*;

fn add(loc: Point, obj: Object, events: &mut Vec<Event>) {
    let action = Action::AddObject(loc, obj);
    events.push(Event::Action(action))
}

pub fn level(game: &Game, map: &str, events: &mut Vec<Event>) {
    let mut loc = Point::origin();
    for ch in map.chars() {
        // TODO: If we keep these level files we may want to add a symbol
        // mapping section so that characters can do things like refer to
        // different uniques.
        match ch {
            ' ' => add(loc, dirt(), events),
            '#' => add(loc, stone_wall(), events),
            'M' => add(loc, metal_wall(), events),
            '+' => add(loc, closed_door(), events),
            '~' => add(loc, shallow_water(), events),
            'V' => add(loc, vitr(), events),
            'T' => add(loc, tree(), events),
            'W' => add(loc, deep_water(), events),
            'P' => {
                add(loc, dirt(), events);
                add(loc, player(), events);
            }
            'D' => {
                add(loc, dirt(), events);
                add(loc, doorman(), events);
            }
            'o' => {
                add(loc, dirt(), events);
                add(loc, spectator(), events);
            }
            'R' => {
                add(loc, dirt(), events);
                add(loc, rhulad(), events);
            }
            's' => {
                add(loc, dirt(), events);
                add(loc, weak_sword(game), events);
            }
            'p' => {
                add(loc, dirt(), events);
                add(loc, pick_axe(), events);
            }
            'S' => {
                add(loc, dirt(), events);
                add(loc, mighty_sword(), events);
            }
            'a' => {
                add(loc, dirt(), events);
                add(loc, sign("the Lesser Armory"), events);
            }
            'b' => {
                add(loc, dirt(), events);
                add(loc, sign("the Greater Armory"), events);
            }
            '\n' => (),
            _ => {
                add(loc, dirt(), events);
                events.push(Event::AddMessage(Message {
                    topic: Topic::Error,
                    text: format!("Ignoring map char '{ch}'"),
                }));
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
        '.',
        Color::LightSlateGray,
        "a patch of dirt",
    )
}

pub fn rubble() -> Object {
    Object::new(
        "rubble",
        ground_tags(Color::Black),
        '…',
        Color::Chocolate,
        "a destroyed wall",
    )
}

pub fn stone_wall() -> Object {
    Object::new(
        "stone wall",
        wall_tags(Color::Black, Material::Stone),
        '#',
        Color::Chocolate,
        "a stone wall",
    )
}

pub fn metal_wall() -> Object {
    Object::new(
        "metal wall",
        wall_tags(Color::Black, Material::Metal),
        '#',
        Color::Silver,
        "a metal wall",
    )
}

pub fn tree() -> Object {
    Object::new("tree", tree_tags(), 'T', Color::ForestGreen, "a tree")
}

pub fn closed_door() -> Object {
    Object::new(
        "closed door",
        door_tags(Color::Black, Material::Stone, false),
        '+',
        Color::Yellow,
        "a closed door",
    )
}

pub fn open_door() -> Object {
    Object::new(
        "open door",
        door_tags(Color::Black, Material::Stone, true),
        '-',
        Color::Yellow,
        "an open door",
    )
}

pub fn shallow_water() -> Object {
    Object::new("shallow water", shallow_water_tags(), '~', Color::Blue, "shallow water")
}

pub fn deep_water() -> Object {
    Object::new("deep water", deep_water_tags(), 'W', Color::Blue, "deep water")
}

pub fn vitr() -> Object {
    Object::new("vitr", vitr_tags(), 'V', Color::Gold, "a pool of chaotic acid")
}

fn doorman() -> Object {
    Object::new(
        "doorman",
        npc_tags("Doorman", Tag::Doorman),
        'D',
        Color::Green,
        "a royal guard",
    )
}

fn rhulad() -> Object {
    Object::new(
        "rhulad",
        npc_tags("Rhulad", Tag::Rhulad),
        'R',
        Color::Red,
        "the Emperor of a Thousand Deaths",
    )
}

fn spectator() -> Object {
    Object::new(
        "spectator",
        npc_tags("Spectator", Tag::Spectator),
        'o',
        Color::Plum,
        "a spectator",
    )
}

fn player() -> Object {
    Object::new("player", player_tags(), '@', Color::Blue, "yourself")
}

pub fn sign(text: &str) -> Object {
    Object::new(
        "sign",
        sign_tags(),
        'i',
        Color::Pink,
        format!("a sign that says '{text}'"),
    )
}

pub fn emp_sword() -> Object {
    Object::new(
        "emp sword",
        emp_sword_tags(),
        'C',
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
    Object::new("weak sword", weak_sword_tags(sword.0), 's', Color::Silver, sword.1)
}

pub fn mighty_sword() -> Object {
    Object::new(
        "mighty sword",
        mighty_sword_tags(),
        'S',
        Color::Silver,
        "the Sword of Impending Doom",
    )
}

pub fn pick_axe() -> Object {
    Object::new("pick-axe", pick_axe_tags(), 'p', Color::Tan, "a pick-axe")
}

fn ground_tags(bg: Color) -> Vec<Tag> {
    vec![Tag::Ground, Tag::Background(bg), Tag::Terrain]
}

fn shallow_water_tags() -> Vec<Tag> {
    vec![Tag::ShallowWater, Tag::Background(Color::LightBlue), Tag::Terrain]
}

fn deep_water_tags() -> Vec<Tag> {
    vec![Tag::DeepWater, Tag::Background(Color::LightBlue), Tag::Terrain]
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

// TODO: Once we get rid of the doorman add CanOpenDoor.
fn npc_tags(name: &str, tag: Tag) -> Vec<Tag> {
    vec![
        Tag::Character,
        // Tag::Inventory(Vec::new()),
        Tag::Name(name.to_string()),
        tag,
    ]
}

fn player_tags() -> Vec<Tag> {
    vec![
        Tag::Character,
        Tag::Durability(Durability { current: 100, max: 100 }),
        Tag::Inventory(Vec::new()),
        Tag::Name(String::from("yourself")),
        Tag::CanOpenDoor,
        Tag::Player,
    ]
}

fn weak_sword_tags(name: &str) -> Vec<Tag> {
    vec![Tag::Name(name.to_string()), Tag::Portable]
}

fn mighty_sword_tags() -> Vec<Tag> {
    vec![Tag::Name(String::from("Sword of Impending Doom")), Tag::Portable]
}

fn pick_axe_tags() -> Vec<Tag> {
    vec![Tag::Name(String::from("a pick-axe")), Tag::PickAxe, Tag::Portable]
}

fn emp_sword_tags() -> Vec<Tag> {
    vec![
        Tag::Name(String::from("Sword of the Crippled God")),
        Tag::Portable,
        Tag::EmpSword,
    ]
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
