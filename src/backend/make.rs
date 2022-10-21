use super::*;
use enum_map::EnumMap;
use rand::prelude::*;

pub fn level(game: &mut Game, map: &str) {
    let mut loc = Point::origin();
    for ch in map.chars() {
        // TODO: If we keep these level files we may want to add a symbol
        // mapping section so that characters can do things like refer to
        // different uniques.
        let _ = match ch {
            ' ' => game.add_object(loc, new_obj(ObjectName::Dirt)),
            '#' => game.add_object(loc, new_obj(ObjectName::StoneWall)),
            'M' => game.add_object(loc, new_obj(ObjectName::MetalWall)),
            '+' => game.add_object(loc, new_obj(ObjectName::ClosedDoor)),
            '~' => game.add_object(loc, new_obj(ObjectName::ShallowWater)),
            'V' => game.add_object(loc, new_obj(ObjectName::Vitr)),
            'T' => game.add_object(loc, new_obj(ObjectName::Tree)),
            'W' => game.add_object(loc, new_obj(ObjectName::DeepWater)),
            'P' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::Player))
            }
            'D' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::Doorman))
            }
            'I' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::Icarium))
            }
            'g' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::Guard))
            }
            'o' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::Spectator))
            }
            'R' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::Rhulad))
            }
            's' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, weak_sword(game))
            }
            'p' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::PickAxe))
            }
            'S' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::MightySword))
            }
            'a' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::LesserArmorySign))
            }
            'b' => {
                game.add_object(loc, new_obj(ObjectName::Dirt));
                game.add_object(loc, new_obj(ObjectName::GreaterArmorySign))
            }
            '\n' => Oid(0),
            _ => {
                game.messages.push(Message {
                    topic: Topic::Error,
                    text: format!("Ignoring map char '{ch}'"),
                });
                game.add_object(loc, new_obj(ObjectName::Dirt))
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
    add_extra(game, new_obj(ObjectName::LeatherHat));
    add_extra(game, new_obj(ObjectName::LeatherChest));
    add_extra(game, new_obj(ObjectName::LeatherGloves));
    add_extra(game, new_obj(ObjectName::LeatherLegs));
    add_extra(game, new_obj(ObjectName::LeatherSandals));
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
            let has_char = game.level.get(loc, CHARACTER_ID).is_some();
            let terrain = game.level.get_bottom(loc).1;
            if Terrain::Ground == terrain.terrain_value().unwrap() {
                if !has_char {
                    game.add_object(loc, obj.clone());
                    break;
                }
            }
        }
    }
}

fn weak_sword(game: &Game) -> Object {
    let swords = vec![
        ObjectName::LongSword,
        ObjectName::Broadsword,
        ObjectName::LongKnife,
        ObjectName::Dagger,
    ];
    let sword = swords.iter().choose(&mut *game.rng()).unwrap();
    new_obj(*sword)
}

fn broken_name(name: ObjectName) -> &'static str {
    use ObjectName::*;
    match name {
        BerokeSoftVoice => "Beroke Soft Voice",
        HaladRackBearer => "Halad Rack Bearer",
        ImrothTheCruel => "Imroth the Cruel",
        KahlbTheSilentHunter => "Kahlb the SilentHunter",
        SiballeTheUnfound => "Siballe the Unfound",
        ThenikTheShattered => "Thenik the Shattered",
        UrugalTheWoven => "Urugal the Woven",
        _ => panic!("expected one of the broken, not {name:?}"),
    }
}

pub fn new_obj(name: ObjectName) -> Object {
    use ObjectName::*;
    match name {
        // Armor
        // TODO: chain armor should be 10, 8, and 6%
        // TODO: plate armor should be 15, 12, and 9%
        LeatherChest => Object::new(
            name,
            "a leather chest",
            Symbol::Armor,
            Color::SandyBrown,
            vec![
                Tag::Name("leather chest"),
                Tag::Portable,
                Tag::Armor(Slot::Chest),
                Tag::Mitigation(5),
            ],
        ),
        LeatherGloves => Object::new(
            name,
            "a leather gloves",
            Symbol::Armor,
            Color::SandyBrown,
            vec![
                Tag::Name("leather gloves"),
                Tag::Portable,
                Tag::Armor(Slot::Hands),
                Tag::Mitigation(3),
            ],
        ),
        LeatherHat => Object::new(
            name,
            "a leather hat",
            Symbol::Armor,
            Color::SandyBrown,
            vec![
                Tag::Name("leather hat"),
                Tag::Portable,
                Tag::Armor(Slot::Head),
                Tag::Mitigation(3),
            ],
        ),
        LeatherLegs => Object::new(
            name,
            "leather shin guard",
            Symbol::Armor,
            Color::SandyBrown,
            vec![
                Tag::Name("leather shin guards"),
                Tag::Portable,
                Tag::Armor(Slot::Legs),
                Tag::Mitigation(4),
            ],
        ),
        LeatherSandals => Object::new(
            name,
            "a leather sandals",
            Symbol::Armor,
            Color::SandyBrown,
            vec![
                Tag::Name("leather sandals"),
                Tag::Portable,
                Tag::Armor(Slot::Feet),
                Tag::Mitigation(3),
            ],
        ),

        // Misc Items
        GreaterArmorySign => Object::new(
            name,
            "a sign that says 'the Greater Armory'",
            Symbol::Sign,
            Color::Pink,
            vec![Tag::Sign],
        ),
        LesserArmorySign => Object::new(
            name,
            "a sign that says 'the Lesser Armory'",
            Symbol::Sign,
            Color::Pink,
            vec![Tag::Sign],
        ),
        PickAxe => Object::new(
            name,
            "a pick-axe",
            Symbol::PickAxe,
            Color::Tan,
            vec![
                Tag::Name("pick-axe"),
                Tag::PickAxe,
                Tag::Delay(time::secs(32)),
                Tag::Portable,
            ],
        ),

        // NPCs
        // https://malazan.fandom.com/wiki/The_Seven_Faces_in_the_Rock
        BerokeSoftVoice | HaladRackBearer | ImrothTheCruel | KahlbTheSilentHunter | SiballeTheUnfound
        | ThenikTheShattered | UrugalTheWoven => Object::new(
            name,
            "One of seven broken Logros T'lan Imass worshipped as gods by the Teblor.",
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
                Tag::Name(broken_name(name)),
                Tag::Scheduled,
                Tag::Character,
            ],
        ),
        Doorman => Object::new(
            name,
            "a royal guard",
            Symbol::Npc('D'),
            Color::Green,
            vec![
                Tag::Disposition(Disposition::Friendly),
                Tag::Name("Doorman"),
                Tag::Doorman,
                Tag::Character,
            ],
        ),
        Guard => Object::new(
            name,
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
        ),
        Icarium => Object::new(
            name,
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
        ),
        Player => Object::new(
            name,
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
                Tag::Equipped(EnumMap::default()),
                Tag::Name("yourself"),
                Tag::CanOpenDoor,
                Tag::Player,
                Tag::Scheduled,
                Tag::Character,
            ],
        ),
        Rhulad => Object::new(
            name,
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
        ),
        Spectator => Object::new(
            name,
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
        ),

        // Terrain
        ClosedDoor => Object::new(
            name,
            "a closed door",
            Symbol::ClosedDoor,
            Color::Yellow,
            door_tags(Color::Black, Material::Stone, false),
        ),
        DeepWater => Object::new(
            name,
            "deep water",
            Symbol::DeepLiquid,
            Color::Blue,
            vec![
                Tag::Terrain(Terrain::DeepWater),
                Tag::Background(Color::LightBlue),
                Tag::Scheduled,
            ],
        ),
        Dirt => Object::new(
            name,
            "a patch of dirt",
            Symbol::Dirt,
            Color::LightSlateGray,
            vec![Tag::Terrain(Terrain::Ground), Tag::Background(Color::Black)],
        ),
        MetalWall => Object::new(
            name,
            "a metal wall",
            Symbol::Wall,
            Color::Silver,
            wall_tags(Color::Black, Material::Metal),
        ),
        OpenDoor => Object::new(
            name,
            "an open door",
            Symbol::OpenDoor,
            Color::Yellow,
            door_tags(Color::Black, Material::Stone, true),
        ),
        Rubble => Object::new(
            name,
            "a destroyed wall",
            Symbol::Rubble,
            Color::Chocolate,
            vec![Tag::Terrain(Terrain::Ground), Tag::Background(Color::Black)],
        ),
        ShallowWater => Object::new(
            name,
            "shallow water",
            Symbol::ShallowLiquid,
            Color::Blue,
            vec![
                Tag::Terrain(Terrain::ShallowWater),
                Tag::Background(Color::LightBlue),
                Tag::Scheduled,
            ],
        ),
        StoneWall => Object::new(
            name,
            "a stone wall",
            Symbol::Wall,
            Color::Chocolate,
            wall_tags(Color::Black, Material::Stone),
        ),
        Tree => Object::new(
            name,
            "a tree",
            Symbol::Tree,
            Color::ForestGreen,
            vec![Tag::Terrain(Terrain::Tree), Tag::Background(Color::Black)],
        ),
        Vitr => Object::new(
            name,
            "a pool of chaotic acid",
            Symbol::DeepLiquid,
            Color::Gold,
            vec![Tag::Terrain(Terrain::Vitr), Tag::Background(Color::Black)],
        ),

        // Weapons
        Broadsword => Object::new(
            name,
            "a dull broadsword",
            Symbol::WeakSword,
            Color::Silver,
            vec![
                Tag::Name("broadsword"),
                Tag::Portable,
                Tag::Weapon(Weapon::OneHand),
                Tag::Damage(12),
                Tag::Delay(time::secs(3)),
                Tag::Strength(4),
                Tag::Dexterity(8),
                Tag::Crit(10),
            ],
        ),
        Dagger => Object::new(
            // TODO: need to re-balance these (and differentiate the weak swords)
            name,
            "a pointy dagger",
            Symbol::WeakSword,
            Color::Silver,
            vec![
                Tag::Name("dagger"),
                Tag::Portable,
                Tag::Weapon(Weapon::OneHand),
                Tag::Damage(12),
                Tag::Delay(time::secs(3)),
                Tag::Strength(4),
                Tag::Dexterity(8),
                Tag::Crit(10),
            ],
        ),
        EmperorSword => Object::new(
            name,
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
        ),
        LongKnife => Object::new(
            name,
            "a shiny long knife",
            Symbol::WeakSword,
            Color::Silver,
            vec![
                Tag::Name("long knife"),
                Tag::Portable,
                Tag::Weapon(Weapon::OneHand),
                Tag::Damage(12),
                Tag::Delay(time::secs(3)),
                Tag::Strength(4),
                Tag::Dexterity(8),
                Tag::Crit(10),
            ],
        ),
        LongSword => Object::new(
            name,
            "a nicked long sword",
            Symbol::WeakSword,
            Color::Silver,
            vec![
                Tag::Name("long sword"),
                Tag::Portable,
                Tag::Weapon(Weapon::OneHand),
                Tag::Damage(12),
                Tag::Delay(time::secs(3)),
                Tag::Strength(4),
                Tag::Dexterity(8),
                Tag::Crit(10),
            ],
        ),
        MightySword => Object::new(
            name,
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
        ),
    }
}

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

fn to_durability(material: Material) -> i32 {
    match material {
        // Material::Wood => 10,
        Material::Stone => 100,
        Material::Metal => 1000,
    }
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
