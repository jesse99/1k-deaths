use super::*;
// use std::mem;

const MAX_NOTES: usize = 100;

fn handle_add_note(game: &mut Game, note: Note) {
    info!("adding note {note:?}");

    game.notes.push_back(note);
    if game.notes.len() > MAX_NOTES {
        // Because notes is a VecDeque pop_front is constant time so there's no real harm
        // in popping after every add.
        game.notes.pop_front();
    }
    assert!(game.notes.len() <= MAX_NOTES);
}

fn handle_move_player(game: &mut Game, loc: Point) {
    info!("moving player to {loc}");
    game.remove_oid(game.player_loc, PLAYER_ID);
    game.append_oid(loc, PLAYER_ID);
    game.player_loc = loc;
    game.pov.dirty();

    OldPoV::update(game); // TODO: this should happen when time advances
    PoV::refresh(game);
}

fn handle_bump(game: &mut Game, oid: Oid, loc: Point) {
    info!("{oid} bump to {loc}");
    // TODO: implement this
}

fn handle_reset(game: &mut Game, reason: &str, map: &str) {
    // TODO: should have an arg for default_terrain
    info!("resetting for {reason}");
    game.player_loc = Point::new(-1, -1);
    game.level.clear();
    game.objects.clear();
    game.notes.clear();

    game.next_id = 1;
    game.new_object("player"); // player
    game.new_object("stone wall"); // default terrain
    game.pov.reset();

    // Note that terrain objects are reused. If their durability drops (because of something
    // like digging) a new object will be created.
    let dirt = game.new_object("dirt");
    let wall = game.new_object("stone wall");
    let deep_water = game.new_object("deep water");
    let shallow_water = game.new_object("shallow water");

    let mut loc = Point::new(0, 0);
    for ch in map.chars() {
        match ch {
            '@' => {
                game.level.insert(loc, vec![dirt, PLAYER_ID]);
                game.player_loc = loc;
                loc.x += 1;
            }
            '#' => {
                game.level.insert(loc, vec![wall]);
                loc.x += 1;
            }
            '~' => {
                game.level.insert(loc, vec![shallow_water]);
                loc.x += 1;
            }
            'W' => {
                game.level.insert(loc, vec![deep_water]);
                loc.x += 1;
            }
            ' ' => {
                game.level.insert(loc, vec![dirt]);
                loc.x += 1;
            }
            '\n' => {
                loc.x = 0;
                loc.y += 1;
            }
            _ => panic!("bad char in map: {ch}"),
        }
    }
    assert!(game.player_loc.x >= 0, "map is missing @");

    OldPoV::update(game);
    PoV::refresh(game);
}

pub fn handle_mutate(game: &mut Game, mesg: StateMutators) {
    use StateMutators::*;
    match mesg {
        Bump(oid, loc) => handle_bump(game, oid, loc),
        Reset(reason, map) => handle_reset(game, &reason, &map),
    }
}
