use crate::backend::support::{
    self, Behavior, Disposition, Durability, Material, ObjectName, Symbol, Terrain, Time, Weapon,
};

use super::super::{Color, Event, Object, Oid, Slot, Tag};
use enum_map::EnumMap;
use fnv::FnvHashMap;

pub struct OidToObj {
    table: FnvHashMap<Oid, Object>,
    next_id: u64, // 0 is the player, 1 is the default object
    last_oid: Oid,
}

impl OidToObj {
    pub fn new() -> OidToObj {
        OidToObj {
            table: FnvHashMap::default(),
            next_id: 2,
            last_oid: Oid(0),
        }
    }

    pub fn lookup(&self, oid: Oid) -> &Object {
        self.table.get(&oid).unwrap()
    }

    pub fn last_oid(&self) -> Oid {
        self.last_oid
    }

    pub fn process(&mut self, event: Event) {
        match event {
            Event::Create(name) => {
                self.last_oid = if name == ObjectName::Player {
                    Oid(0)
                } else {
                    let o = Oid(self.next_id);
                    self.next_id += 1;
                    o
                };
                self.table.insert(self.last_oid, new_obj(name));
            }
            _ => (),
        }
    }
}

fn new_obj(name: ObjectName) -> Object {
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
                Tag::Delay(support::secs(32)),
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
                Tag::Delay(support::secs(5)),
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
                Tag::Delay(support::secs(3)),
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
                Tag::Delay(support::secs(3)),
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
                Tag::Delay(support::secs(2)),
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
                Tag::Delay(support::secs(4)),
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
                Tag::Delay(support::secs(3)),
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
                Tag::Delay(support::secs(3)),
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
                Tag::Delay(support::secs(5)),
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
                Tag::Delay(support::secs(3)),
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
                Tag::Delay(support::secs(3)),
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
                Tag::Delay(support::secs(5)),
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
