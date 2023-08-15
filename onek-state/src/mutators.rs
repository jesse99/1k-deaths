use super::*;

fn char_to_terrain(ch: char) -> Option<Terrain> {
    match ch {
        '#' => Some(Terrain::Wall),
        ' ' => Some(Terrain::Dirt),
        _ => None,
    }
}

fn handle_move_player(game: &mut Game, loc: Point) {
    info!("moving plater to {loc}");
    game.player_loc = loc;
}

fn handle_reset(game: &mut Game, map: &str) {
    info!("resetting");
    game.terrain.clear();
    game.reply_senders.clear();

    let mut loc = Point::new(0, 0);
    for ch in map.chars() {
        match ch {
            '@' => {
                game.terrain.insert(loc, Terrain::Dirt);
                game.player_loc = loc;
                loc.x += 1;
            }
            '\n' => {
                loc.x = 0;
                loc.y += 1;
            }
            _ => match char_to_terrain(ch) {
                Some(terrain) => {
                    game.terrain.insert(loc, terrain);
                    loc.x += 1;
                }
                None => panic!("bad char in map: {ch}"),
            },
        }
    }
}

pub fn handle_mutate(game: &mut Game, mesg: StateMutators) {
    match mesg {
        StateMutators::MovePlayer(loc) => handle_move_player(game, loc),
        StateMutators::Reset(map) => handle_reset(game, &map),
    }
}
