use super::*;
use std::convert::From;

// TODO: can we make this debug_assertions only?
impl From<&str> for Level {
    fn from(map: &str) -> Self {
        let mut level = Level::new();

        let mut loc = Point::origin();
        for ch in map.chars() {
            match ch {
                ' ' => level.create_terrain(loc, Terrain::Dirt),
                's' => add_weak_sword(&mut level, loc),
                'M' => level.create_terrain(loc, Terrain::Wall), // TODO: should be metal
                'T' => level.create_terrain(loc, Terrain::Tree),
                'S' => add_mighty_sword(&mut level, loc),
                'V' => level.create_terrain(loc, Terrain::Vitr),
                'W' => level.create_terrain(loc, Terrain::DeepWater),
                '@' => add_player(&mut level, loc),
                '~' => level.create_terrain(loc, Terrain::ShallowWater),
                '#' => level.create_terrain(loc, Terrain::Wall),
                '+' => level.create_terrain(loc, Terrain::ClosedDoor),
                '-' => level.create_terrain(loc, Terrain::OpenDoor),
                '\n' => (),
                _ => error!("bad char '{ch}' in store_from_str"), // TODO: probably should make this an error message
            }
            if ch == '\n' {
                loc.x = 0;
                loc.y += 1;
            } else {
                loc.x += 1;
            }
        }

        level
    }
}

fn add_player(level: &mut Level, loc: Point) {
    level.create_terrain(loc, Terrain::Dirt);
    level.create_player(loc);
}

fn add_weak_sword(level: &mut Level, loc: Point) {
    level.create_terrain(loc, Terrain::Dirt);
    level.append_portable(loc, "weak sword", Portable::WeakSword);
}

fn add_mighty_sword(level: &mut Level, loc: Point) {
    level.create_terrain(loc, Terrain::Dirt);
    level.append_portable(loc, "mighty sword", Portable::MightySword);
}
