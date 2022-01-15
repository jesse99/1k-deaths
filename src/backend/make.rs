use super::{Color, Event, Game, Liquid, Material, Message, Object, Point, Tag, Topic};

pub fn level(game: &mut Game, map: &str) {
    let mut loc = Point::origin();
    for ch in map.chars() {
        match ch {
            ' ' => game.post(Event::AddObject(loc, dirt())),
            '#' => game.post(Event::AddObject(loc, stone_wall())),
            'M' => game.post(Event::AddObject(loc, metal_wall())),
            '+' => game.post(Event::AddObject(loc, closed_door())),
            '~' => game.post(Event::AddObject(loc, shallow_water())),
            'V' => game.post(Event::AddObject(loc, vitr())),
            'T' => game.post(Event::AddObject(loc, tree())),
            'W' => game.post(Event::AddObject(loc, deep_water())),
            'P' => {
                game.post(Event::AddObject(loc, dirt()));
                game.post(Event::AddObject(loc, player()));
            }
            's' => {
                game.post(Event::AddObject(loc, dirt()));
                game.post(Event::AddObject(loc, weak_sword()));
            }
            'S' => {
                game.post(Event::AddObject(loc, dirt()));
                game.post(Event::AddObject(loc, mighty_sword()));
            }
            'a' => {
                game.post(Event::AddObject(loc, dirt()));
                game.post(Event::AddObject(loc, sign("the Lesser Armory")));
            }
            'b' => {
                game.post(Event::AddObject(loc, dirt()));
                game.post(Event::AddObject(loc, sign("the Greater Armory")));
            }
            '\n' => (),
            _ => {
                game.post(Event::AddObject(loc, dirt()));
                game.post(Event::AddMessage(Message {
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
    Object {
        dname: String::from("dirt"),
        tags: ground_tags(Color::Black),
        symbol: '.',
        color: Color::LightSlateGray,
        description: String::from("a patch of dirt"),
    }
}

pub fn stone_wall() -> Object {
    Object {
        dname: String::from("stone wall"),
        tags: wall_tags(Color::Black, Material::Stone),
        symbol: '#',
        color: Color::Chocolate,
        description: String::from("a stone wall"),
    }
}

pub fn metal_wall() -> Object {
    Object {
        dname: String::from("metal wall"),
        tags: wall_tags(Color::Black, Material::Metal),
        symbol: '#',
        color: Color::Silver,
        description: String::from("a metal wall"),
    }
}

pub fn tree() -> Object {
    Object {
        dname: String::from("tree"),
        tags: tree_tags(),
        symbol: 'T',
        color: Color::ForestGreen,
        description: String::from("a tree"),
    }
}

pub fn closed_door() -> Object {
    Object {
        dname: String::from("closed door"),
        tags: door_tags(Color::Black, Material::Stone, false),
        symbol: '+',
        color: Color::Yellow,
        description: String::from("a closed door"),
    }
}

pub fn open_door() -> Object {
    Object {
        dname: String::from("open door"),
        tags: door_tags(Color::Black, Material::Stone, true),
        symbol: '-',
        color: Color::Yellow,
        description: String::from("an open door"),
    }
}

pub fn shallow_water() -> Object {
    Object {
        dname: String::from("shallow water"),
        tags: shallow_water_tags(),
        symbol: '~',
        color: Color::Blue,
        description: String::from("shallow water"),
    }
}

pub fn deep_water() -> Object {
    Object {
        dname: String::from("deep water"),
        tags: deep_water_tags(),
        symbol: 'W',
        color: Color::Blue,
        description: String::from("deep water"),
    }
}

pub fn vitr() -> Object {
    Object {
        dname: String::from("vitr"),
        tags: vitr_tags(),
        symbol: 'V',
        color: Color::Gold,
        description: String::from("a pool of chaotic acid"),
    }
}

fn player() -> Object {
    Object {
        dname: String::from("player"),
        tags: player_tags(),
        symbol: '@',
        color: Color::Blue,
        description: String::from("yourself"),
    }
}

pub fn sign(text: &str) -> Object {
    Object {
        dname: String::from("sign"),
        tags: sign_tags(),
        symbol: 'i',
        color: Color::Pink,
        description: format!("a sign that says '{text}'"),
    }
}

pub fn weak_sword() -> Object {
    Object {
        dname: String::from("weak_sword"),
        tags: weak_sword_tags(),
        symbol: 's',
        color: Color::Silver,
        description: "a nicked long sword".to_string(),
    }
}

pub fn mighty_sword() -> Object {
    Object {
        dname: String::from("mighty_sword"),
        tags: mighty_sword_tags(),
        symbol: 'S',
        color: Color::Silver,
        description: "the Sword of Impending Doom".to_string(),
    }
}

fn ground_tags(bg: Color) -> Vec<Tag> {
    vec![Tag::Ground, Tag::Background(bg), Tag::Terrain]
}

fn shallow_water_tags() -> Vec<Tag> {
    vec![
        Tag::Liquid {
            liquid: Liquid::Water,
            deep: false,
        },
        Tag::Background(Color::LightBlue),
        Tag::Terrain,
    ]
}

fn deep_water_tags() -> Vec<Tag> {
    vec![
        Tag::Liquid {
            liquid: Liquid::Water,
            deep: true,
        },
        Tag::Background(Color::LightBlue),
        Tag::Terrain,
    ]
}
fn vitr_tags() -> Vec<Tag> {
    vec![
        Tag::Liquid {
            liquid: Liquid::Vitr,
            deep: true,
        },
        Tag::Background(Color::Black),
        Tag::Terrain,
    ]
}

fn wall_tags(bg: Color, material: Material) -> Vec<Tag> {
    let durability = 5 * to_durability(material); // walls are quite a bit tougher than something like a door
    vec![
        Tag::Durability {
            current: durability,
            max: durability,
        },
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
        Tag::Durability {
            current: durability,
            max: durability,
        },
        Tag::Material(material),
        if open { Tag::OpenDoor } else { Tag::ClosedDoor },
        Tag::Background(bg),
        Tag::Terrain,
    ]
}

fn player_tags() -> Vec<Tag> {
    vec![
        Tag::Character,
        Tag::Durability {
            current: 100,
            max: 100,
        },
        Tag::Name(String::from("yourself")),
        Tag::Player,
    ]
}

fn weak_sword_tags() -> Vec<Tag> {
    vec![Tag::Name(String::from("long sword")), Tag::Portable]
}

fn mighty_sword_tags() -> Vec<Tag> {
    vec![
        Tag::Name(String::from("Sword of Impending Doom")),
        Tag::Portable,
    ]
}

fn sign_tags() -> Vec<Tag> {
    vec![Tag::Sign]
}

fn to_durability(material: Material) -> i32 {
    match material {
        Material::Wood => 100,
        Material::Stone => 1000,
        Material::Metal => 10000,
    }
}
