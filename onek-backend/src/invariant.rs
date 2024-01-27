use super::*;

// TODO: probably want fast and slow invariants
// TODO: do fast invariants always in debug, check limited number of cells
// TODO: do slow invariants if an option is set, check all cells
#[cfg(debug_assertions)]
pub fn invariant(game: &Game) {
    for (loc, oids) in game.level.iter() {
        assert!(!oids.is_empty(), "cell at {loc} is empty");
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
        }

        // player should be at player_loc
        let oid = oids.last().unwrap();
        if *loc == game.player_loc {
            assert!(
                *oid == PLAYER_ID,
                "last object at player_loc should be the player not {oid}"
            );
        } else {
            assert!(
                *oid != PLAYER_ID,
                "last object at {loc} is the player but player_loc is {}",
                game.player_loc
            );
        }
    }
}
