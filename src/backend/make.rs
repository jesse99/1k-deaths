use super::*;
use rand::prelude::*;

pub fn level(game: &mut Game, map: &str) {
    let mut loc = Point::origin();
    for ch in map.chars() {
        // TODO: If we keep these level files we may want to add a symbol
        // mapping section so that characters can do things like refer to
        // different uniques.
        let _ = match ch {
            ' ' => game.add_object(&loc, dirt()),
            '#' => game.add_object(&loc, stone_wall()),
            'M' => game.add_object(&loc, metal_wall()),
            '+' => game.add_object(&loc, closed_door()),
            '~' => game.add_object(&loc, shallow_water()),
            'V' => game.add_object(&loc, vitr()),
            'T' => game.add_object(&loc, tree()),
            'W' => game.add_object(&loc, deep_water()),
            'P' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, player())
            }
            'D' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, doorman())
            }
            'I' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, icarium())
            }
            'g' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, guard())
            }
            'o' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, spectator())
            }
            'R' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, rhulad())
            }
            's' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, weak_sword(game))
            }
            'p' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, pick_axe())
            }
            'S' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, mighty_sword())
            }
            'a' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, sign("a sign that says 'the Lesser Armory'"))
            }
            'b' => {
                game.add_object(&loc, dirt());
                game.add_object(&loc, sign("a sign that says 'the Greater Armory'"))
            }
            '\n' => Oid(0),
            _ => {
                game.messages.push(Message {
                    topic: Topic::Error,
                    text: format!("Ignoring map char '{ch}'"),
                });
                game.add_object(&loc, dirt())
            }
        };
        if ch == '\n' {
            loc = Point::new(0, loc.y + 1);
        } else {
            loc = Point::new(loc.x + 1, loc.y);
        }
    }
    add_extras(game);
}

fn add_extras(game: &mut Game) {
    add_extra(game, leather_hat());
    add_extra(game, leather_chest());
    add_extra(game, leather_gloves());
    add_extra(game, leather_legs());
    add_extra(game, leather_sandals());
}

fn add_extra(game: &mut Game, obj: Object) {
    let mut count = 0;
    {
        let rng = &mut *game.rng();
        while rng.gen_bool(0.5) {
            count += 1;
        }
    }

    for _ in 0..count {
        for _ in 0..5 {
            // we'll try 5x to add count instances of obj
            let loc = game.level.random_loc(&game.rng);
            let has_char = game.level.get(&loc, CHARACTER_ID).is_some();
            let terrain = game.level.get_bottom(&loc).1;
            if Terrain::Ground == terrain.terrain_value().unwrap() {
                if !has_char {
                    game.add_object(&loc, obj.clone());
                    break;
                }
            }
        }
    }
}

// -- characters -------------------------------------------------------------------------
// https://malazan.fandom.com/wiki/The_Seven_Faces_in_the_Rock
pub fn broken(i: usize) -> Object {
    let names = vec![
        "Beroke Soft Voice",
        "Halad Rack Bearer",
        "Imroth the Cruel",
        "Kahlb the Silent Hunter",
        "Siballe the Unfound",
        "Thenik the Shattered",
        "Urugal the Woven",
    ];
    Object::new(
        names[i],
        names[i],
        Symbol::Npc('u'),
        Color::Red,
        vec![
            Tag::Strength(10),
            Tag::Dexterity(10),
            Tag::Disposition(Disposition::Aggressive),
            Tag::Behavior(Behavior::Wandering(Time::max())),
            Tag::Damage(35),
            Tag::Delay(time::secs(5)),
            Tag::Durability(Durability { current: 170, max: 170 }),
            Tag::Name(names[i]),
            Tag::Scheduled,
            Tag::Character,
        ],
    )
}

fn doorman() -> Object {
    Object::new(
        "doorman",
        "a royal guard",
        Symbol::Npc('D'),
        Color::Green,
        vec![
            Tag::Disposition(Disposition::Friendly),
            Tag::Name("Doorman"),
            Tag::Doorman,
            Tag::Character,
        ],
    )
}

pub fn guard() -> Object {
    Object::new(
        "a guard",
        "a low level guard",
        Symbol::Npc('g'),
        Color::Green,
        vec![
            Tag::Strength(10),
            Tag::Dexterity(10),
            Tag::Disposition(Disposition::Neutral),
            Tag::Behavior(Behavior::Sleeping),
            Tag::Damage(6),
            Tag::Delay(time::secs(3)),
            Tag::Flees(50),
            Tag::Hearing(0),
            Tag::Durability(Durability { current: 30, max: 30 }),
            Tag::Name("a guard"),
            Tag::Guard,
            Tag::Scheduled,
            Tag::Character,
        ],
    )
}

fn icarium() -> Object {
    Object::new(
        "icarium",
        "Icarium Lifestealer, a mixed blood Jahgut. He looks extremely dangerous",
        Symbol::Npc('I'),
        Color::LightGrey,
        vec![
            Tag::Strength(10),
            Tag::Dexterity(20),
            Tag::Disposition(Disposition::Neutral),
            Tag::Behavior(Behavior::Wandering(Time::max())),
            Tag::Damage(45),
            Tag::Delay(time::secs(3)),
            Tag::Durability(Durability { current: 500, max: 500 }),
            Tag::Name("Icarium"),
            Tag::Icarium,
            Tag::Scheduled,
            Tag::Character,
        ],
    )
}

pub fn player() -> Object {
    Object::new(
        "player",
        "yourself",
        Symbol::Player,
        Color::Linen,
        vec![
            Tag::Strength(10),
            Tag::Dexterity(10),
            Tag::Durability(Durability { current: 100, max: 100 }),
            Tag::Damage(6),
            Tag::Delay(time::secs(2)),
            Tag::Inventory(Vec::new()),
            Tag::Name("yourself"),
            Tag::CanOpenDoor,
            Tag::Player,
            Tag::Scheduled,
            Tag::Character,
        ],
    )
}

pub fn rhulad() -> Object {
    Object::new(
        "rhulad",
        "the Emperor of a Thousand Deaths",
        Symbol::Npc('R'),
        Color::Red,
        vec![
            Tag::Strength(10),
            Tag::Dexterity(10),
            Tag::Disposition(Disposition::Aggressive),
            Tag::Behavior(Behavior::Sleeping),
            Tag::Damage(24),
            Tag::Delay(time::secs(4)),
            Tag::Durability(Durability { current: 100, max: 100 }),
            Tag::Name("Rhulad"),
            Tag::Rhulad,
            Tag::Scheduled,
            Tag::Character,
        ],
    )
}

fn spectator() -> Object {
    Object::new(
        "spectator",
        "a spectator",
        Symbol::Npc('s'),
        Color::Plum,
        vec![
            Tag::Strength(10),
            Tag::Dexterity(10),
            Tag::Disposition(Disposition::Neutral),
            Tag::Behavior(Behavior::Sleeping),
            Tag::Hearing(0),
            Tag::Durability(Durability { current: 33, max: 33 }),
            Tag::Name("Spectator"),
            Tag::Spectator,
            Tag::Scheduled,
            Tag::Character,
        ],
    )
}

// -- items ------------------------------------------------------------------------------
pub fn sign(text: &'static str) -> Object {
    Object::new("sign", text, Symbol::Sign, Color::Pink, vec![Tag::Sign])
}

pub fn emp_sword() -> Object {
    Object::new(
        "emp sword",
        "the Sword of the Crippled God",
        Symbol::StrongSword,
        Color::Silver,
        vec![
            Tag::Name("Sword of the Crippled God"),
            Tag::Weapon(Weapon::TwoHander),
            Tag::Portable,
            Tag::EmpSword,
            Tag::Damage(50),
            Tag::Delay(time::secs(5)),
            Tag::Crit(3),
            Tag::Strength(7),
            Tag::Dexterity(20),
        ],
    )
}

// TODO: there should be different functions for these, eg swords and daggers should have
// different stats
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
        sword.1,
        Symbol::WeakSword,
        Color::Silver,
        vec![
            Tag::Name(sword.0),
            Tag::Portable,
            Tag::Weapon(Weapon::OneHand),
            Tag::Damage(12),
            Tag::Delay(time::secs(3)),
            Tag::Strength(4),
            Tag::Dexterity(8),
            Tag::Crit(10),
        ],
    )
}

pub fn mighty_sword() -> Object {
    Object::new(
        "mighty sword",
        "the Sword of Impending Doom",
        Symbol::StrongSword,
        Color::Silver,
        vec![
            Tag::Name("Sword of Impending Doom"),
            Tag::Portable,
            Tag::Weapon(Weapon::TwoHander),
            Tag::Damage(40),
            Tag::Delay(time::secs(5)),
            Tag::Strength(6),
            Tag::Dexterity(15),
            Tag::Crit(2),
        ],
    )
}

pub fn leather_hat() -> Object {
    Object::new(
        "leather hat",
        "a leather hat",
        Symbol::Armor,
        Color::SandyBrown,
        vec![
            Tag::Name("leather hat"),
            Tag::Portable,
            Tag::Armor(Armor::Head),
            Tag::Mitigation(3),
        ],
    )
}

pub fn leather_chest() -> Object {
    Object::new(
        "leather chest",
        "a leather chest",
        Symbol::Armor,
        Color::SandyBrown,
        vec![
            Tag::Name("leather chest"),
            Tag::Portable,
            Tag::Armor(Armor::Chest),
            Tag::Mitigation(5),
        ],
    )
}

pub fn leather_gloves() -> Object {
    Object::new(
        "leather gloves",
        "a leather gloves",
        Symbol::Armor,
        Color::SandyBrown,
        vec![
            Tag::Name("leather gloves"),
            Tag::Portable,
            Tag::Armor(Armor::Hands),
            Tag::Mitigation(3),
        ],
    )
}

pub fn leather_legs() -> Object {
    Object::new(
        "leather shin guards",
        "leather shin guard",
        Symbol::Armor,
        Color::SandyBrown,
        vec![
            Tag::Name("leather shin guards"),
            Tag::Portable,
            Tag::Armor(Armor::Legs),
            Tag::Mitigation(4),
        ],
    )
}

pub fn leather_sandals() -> Object {
    Object::new(
        "leather sandals",
        "a leather sandals",
        Symbol::Armor,
        Color::SandyBrown,
        vec![
            Tag::Name("leather sandals"),
            Tag::Portable,
            Tag::Armor(Armor::Feet),
            Tag::Mitigation(3),
        ],
    )
}
// TODO: chain armor should be 10, 8, and 6%
// TODO: plate armor should be 15, 12, and 9%

pub fn pick_axe() -> Object {
    Object::new(
        "pick-axe",
        "a pick-axe",
        Symbol::PickAxe,
        Color::Tan,
        vec![
            Tag::Name("pick-axe"),
            Tag::PickAxe,
            Tag::Delay(time::secs(32)),
            Tag::Portable,
        ],
    )
}

// -- terrain ----------------------------------------------------------------------------
pub fn dirt() -> Object {
    Object::new(
        "dirt",
        "a patch of dirt",
        Symbol::Dirt,
        Color::LightSlateGray,
        vec![Tag::Terrain(Terrain::Ground), Tag::Background(Color::Black)],
    )
}

pub fn rubble() -> Object {
    Object::new(
        "rubble",
        "a destroyed wall",
        Symbol::Rubble,
        Color::Chocolate,
        vec![Tag::Terrain(Terrain::Ground), Tag::Background(Color::Black)],
    )
}

pub fn stone_wall() -> Object {
    Object::new(
        "stone wall",
        "a stone wall",
        Symbol::Wall,
        Color::Chocolate,
        wall_tags(Color::Black, Material::Stone),
    )
}

pub fn metal_wall() -> Object {
    Object::new(
        "metal wall",
        "a metal wall",
        Symbol::Wall,
        Color::Silver,
        wall_tags(Color::Black, Material::Metal),
    )
}

pub fn tree() -> Object {
    Object::new(
        "tree",
        "a tree",
        Symbol::Tree,
        Color::ForestGreen,
        vec![Tag::Terrain(Terrain::Tree), Tag::Background(Color::Black)],
    )
}

pub fn closed_door() -> Object {
    Object::new(
        "closed door",
        "a closed door",
        Symbol::ClosedDoor,
        Color::Yellow,
        door_tags(Color::Black, Material::Stone, false),
    )
}

pub fn open_door() -> Object {
    Object::new(
        "open door",
        "an open door",
        Symbol::OpenDoor,
        Color::Yellow,
        door_tags(Color::Black, Material::Stone, true),
    )
}

pub fn shallow_water() -> Object {
    Object::new(
        "shallow water",
        "shallow water",
        Symbol::ShallowLiquid,
        Color::Blue,
        vec![
            Tag::Terrain(Terrain::ShallowWater),
            Tag::Background(Color::LightBlue),
            Tag::Scheduled,
        ],
    )
}

pub fn deep_water() -> Object {
    Object::new(
        "deep water",
        "deep water",
        Symbol::DeepLiquid,
        Color::Blue,
        vec![
            Tag::Terrain(Terrain::DeepWater),
            Tag::Background(Color::LightBlue),
            Tag::Scheduled,
        ],
    )
}

pub fn vitr() -> Object {
    Object::new(
        "vitr",
        "a pool of chaotic acid",
        Symbol::DeepLiquid,
        Color::Gold,
        vec![Tag::Terrain(Terrain::Vitr), Tag::Background(Color::Black)],
    )
}

// --- tags ------------------------------------------------------------------------------
fn wall_tags(bg: Color, material: Material) -> Vec<Tag> {
    let durability = 5 * to_durability(material); // walls are quite a bit tougher than something like a door
    vec![
        Tag::Durability(Durability {
            current: durability,
            max: durability,
        }),
        Tag::Material(material),
        Tag::Terrain(Terrain::Wall),
        Tag::Background(bg),
    ]
}

fn door_tags(bg: Color, material: Material, open: bool) -> Vec<Tag> {
    let durability = to_durability(material);
    vec![
        Tag::Durability(Durability {
            current: durability,
            max: durability,
        }),
        Tag::Material(material),
        if open {
            Tag::Terrain(Terrain::OpenDoor)
        } else {
            Tag::Terrain(Terrain::ClosedDoor)
        },
        Tag::Background(bg),
    ]
}

fn to_durability(material: Material) -> i32 {
    match material {
        // Material::Wood => 10,
        Material::Stone => 100,
        Material::Metal => 1000,
    }
}
