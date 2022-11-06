use super::*;
use std::convert::From;

// TODO: can we make this debug_assertions only?
impl From<&str> for Store {
    fn from(map: &str) -> Self {
        let mut store = Store::new();

        store.create(ObjectId::DefaultCell, Relation::Objects(vec![]));
        store.create(ObjectId::DefaultCell, Relation::Terrain(Terrain::Wall));

        let mut loc = Point::origin();
        for ch in map.chars() {
            match ch {
                ' ' => add_terrain(&mut store, loc, Terrain::Dirt),
                'W' => add_terrain(&mut store, loc, Terrain::DeepWater),
                '@' => add_player(&mut store, loc),
                '~' => add_terrain(&mut store, loc, Terrain::ShallowWater),
                '#' => add_terrain(&mut store, loc, Terrain::Wall),
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

        store
    }
}

fn add_player(store: &mut Store, loc: Point) {
    store.create(ObjectId::Player, Relation::Location(loc));
    store.create(ObjectId::Player, Relation::Objects(vec![]));
}

fn add_terrain(store: &mut Store, loc: Point, terrain: Terrain) {
    let oid = ObjectId::Cell(loc);
    store.create(oid, Relation::Location(loc));
    store.create(oid, Relation::Terrain(terrain));
}
