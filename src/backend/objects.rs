use super::{Color, Liquid, Material, Object, Tag};

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

pub fn door() -> Object {
    Object {
        dname: String::from("closed door"),
        tags: door_tags(Color::Black, Material::Stone, false),
        symbol: '+',
        color: Color::Yellow,
        description: String::from("a closed door"),
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

pub fn player() -> Object {
    Object {
        dname: String::from("player"),
        tags: player_tags(),
        symbol: '@',
        color: Color::Blue,
        description: String::from("yourself"),
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

fn to_durability(material: Material) -> i32 {
    match material {
        Material::Wood => 100,
        Material::Stone => 1000,
        Material::Metal => 10000,
    }
}
