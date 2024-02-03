use super::invariant::*;
use super::*;
use fnv::FnvHashSet;

const MAX_NOTES: usize = 100;

fn player_can_move_in(object: &Object) -> Option<Note> {
    if let Some(value) = object.get("blocks_move") {
        Some(Note::new(NoteKind::Error, value.to_str().to_owned()))
    } else {
        None
    }
}

fn player_move_mesg(object: &Object) -> Option<Note> {
    if let Some(value) = object.get("move_mesg") {
        Some(Note::new(NoteKind::Environmental, value.to_str().to_owned()))
    } else {
        None
    }
}

fn player_can_move(game: &Game, to: Point) -> Option<Note> {
    if let Some(cell) = logical_cell(game, to) {
        for object in cell.iter() {
            if let Some(note) = player_can_move_in(&object) {
                return Some(note);
            }
        }
        for object in cell {
            if let Some(note) = player_move_mesg(&object) {
                return Some(note);
            }
        }
        None
    } else {
        let object = game.objects.get(&DEFAULT_CELL_OID).unwrap();
        if let Some(note) = player_can_move_in(object) {
            Some(note)
        } else {
            player_move_mesg(object)
        }
    }
}

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
    game.remove_oid(game.player_loc, PLAYER_OID);
    game.append_oid(loc, PLAYER_OID);
    game.player_loc = loc;
    game.pov.dirty();

    OldPoV::update(game); // TODO: this should happen when time advances
    PoV::refresh(game);
}

fn find_closed_door(game: &mut Game, loc: Point) -> Option<Oid> {
    if let Some(oids) = game.level.get(&loc) {
        for oid in oids {
            let object = game.objects.get(&oid).unwrap();
            if let Some(value) = object.get("tag") {
                let tag = value.to_tag();
                if tag.0 == "closed door" {
                    return Some(*oid);
                }
            }
        }
    }
    None
}

fn bumped_object(game: &mut Game, loc: Point) -> bool {
    if let Some(old_oid) = find_closed_door(game, loc) {
        let new_oid = game.new_object("open door");
        game.replace_oid(loc, old_oid, new_oid);
        OldPoV::update(game); // TODO: this should happen when time advances
        PoV::refresh(game);
        return true;
    }
    false
}

fn attempt_player_move(game: &mut Game, loc: Point) {
    // Can the player move? If not we'll get an error note. If so we may get an
    // environmental note.
    let note = player_can_move(game, loc);
    match &note {
        None => (),
        Some(note) => handle_add_note(game, note.clone()),
    }

    // Do the move as long as the note isn't an error note.
    match note {
        Some(Note {
            kind: NoteKind::Error,
            text: _,
        }) => (),
        _ => handle_move_player(game, loc),
    }
    PoV::refresh(game)
}

// TODO: should we rule out bumps more than one square from oid?
// TODO: what about stuff like hopping? maybe those are restricted to moves?
fn handle_player_bump(game: &mut Game, loc: Point) {
    info!("player bump to {loc}");

    // At some point may want a dispatch table, e.g.
    // (Actor, Action, Obj) -> Handler(actor, obj)
    if !bumped_object(game, loc) {
        attempt_player_move(game, loc);
    }
}

fn handle_examine(game: &mut Game, loc: Point, wizard: bool) {
    info!("examine {loc}");

    let notes = if game.pov.visible(&game, &loc) {
        let oids = game.level.get(&loc).unwrap();
        oids.iter()
            .map(|oid| {
                let object = game.objects.get(oid).unwrap();
                let mut desc = object.get("description").unwrap().to_str().to_owned();
                if wizard {
                    desc.push_str(&format!(" loc: {loc}"));
                }
                desc
            })
            .collect()
    } else {
        vec!["You can't see there.".to_owned()]
    };
    for note in notes {
        let note = Note {
            kind: NoteKind::Info,
            text: note,
        };
        handle_add_note(game, note);
    }
}

static STARTING_LEVEL: &'static str = include_str!("../data/start.txt");

fn handle_new_level(game: &mut Game, name: String) {
    if name == "start" {
        let reason = format!("new level {name}");
        handle_reset(game, &reason, STARTING_LEVEL);
    } else {
        panic!("'{name}' isn't a known level");
    }
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
    let mut bad_chars = FnvHashSet::default();
    for ch in map.chars() {
        match ch {
            '@' => {
                game.level.insert(loc, vec![dirt, PLAYER_OID]);
                game.player_loc = loc;
                loc.x += 1;
            }
            '+' => {
                let closed_door = game.new_object("closed door");
                game.level.insert(loc, vec![dirt, closed_door]);
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
            _ => {
                if !bad_chars.contains(&ch) {
                    handle_add_note(
                        game,
                        Note {
                            text: format!("bad char in map: {ch}"),
                            kind: NoteKind::Error,
                        },
                    );
                    bad_chars.insert(ch);
                }
            }
        }
    }
    assert!(game.player_loc.x >= 0, "map is missing @");

    OldPoV::update(game);
    PoV::refresh(game)
}

pub fn handle_mutate(game: &mut Game, mesg: StateMutators) {
    use StateMutators::*;
    match mesg {
        Bump(loc) => handle_player_bump(game, loc),
        Examine { loc, wizard } => handle_examine(game, loc, wizard),
        NewLevel(name) => handle_new_level(game, name),
        Reset { reason, map } => handle_reset(game, &reason, &map),
    }

    #[cfg(debug_assertions)]
    invariant(&game);
}
