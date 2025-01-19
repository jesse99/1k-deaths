use super::*;

// TODO: do fast invariants always in debug, check limited number of cells
// TODO: do slow invariants if an option is set, check all cells
#[cfg(debug_assertions)]
pub fn invariant(game: &Game) {
    if game.invariants {
        for (loc, oids) in game.level.iter() {
            check_cell(game, *loc, oids);
        }
    } else {
        // Check up to an 8x8 grid around the player, ignoring LOS.
        const DELTA: i32 = 4;
        for dy in -DELTA..=DELTA {
            for dx in -DELTA..=DELTA {
                let loc = Point::new(game.player_loc.x + dx, game.player_loc.y + dy);
                if let Some(oids) = game.level.get(&loc) {
                    check_cell(game, loc, oids);
                }
            }
        }
    }
}

#[cfg(debug_assertions)]
pub fn check_cell(game: &Game, loc: Point, oids: &Vec<Oid>) {
    for (i, oid) in oids.iter().enumerate() {
        let object = game.objects.get(oid).expect(&format!("No object for {oid} at {loc}"));
        if i == 0 {
            // first object in a cell should be terrain
            assert!(
                object.contains_key("blocks_los"),
                "first object at {loc} is missing blocks_Los"
            );
            assert!(object.contains_key("color"), "first object at {loc} is missing color");
            assert!(
                object.contains_key("back_color"),
                "first object at {loc} is missing back_color"
            );
        }

        // all objects should have tag, oid, description, symbol
        assert!(object.contains_key("tag"), "first object at {loc} is missing tag");
        assert!(object.contains_key("oid"), "first object at {loc} is missing oid");
        assert!(
            object.contains_key("description"),
            "first object at {loc} is missing description"
        );
        assert!(object.contains_key("symbol"), "first object at {loc} is missing symbol");

        // oids should be consistent
        let value = object.get("oid").unwrap().to_oid();
        assert!(value == oid, "object has oid {value} but level has {oid}");

        // player should only be at player_loc
        if *oid == PLAYER_OID {
            assert!(
                loc == game.player_loc,
                "player should be at {} not {loc}",
                game.player_loc,
            );
        }
    }
}
