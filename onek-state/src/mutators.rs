use super::*;
use std::mem;

fn char_to_terrain(ch: char) -> Option<Terrain> {
    match ch {
        '#' => Some(Terrain::Wall),
        ' ' => Some(Terrain::Dirt),
        _ => None,
    }
}

fn handle_begin_transaction(game: &mut Game, id: String) {
    info!("begin read transaction with {id}");
    game.read_transactions.push(id);
    debug_assert!(game.read_transactions.len() < 100); // sanity check
}

fn handle_end_transaction(game: &mut Game, id: String) {
    info!("end read transaction with {id}");

    // Read transactions can overlap instead of always being strictly nested so we need
    // to search for the transacvtion id.
    match game.read_transactions.iter().position(|value| *value == id) {
        Some(index) => game.read_transactions.remove(index),
        None => panic!("failed to find read transaction {id}"),
    };

    if game.read_transactions.is_empty() {
        let mut mesgs = Vec::new();
        mem::swap(&mut mesgs, &mut game.queued_mutates);

        for mesg in mesgs {
            handle_mutate(game, mesg);
        }
    }
}

fn handle_move_player(game: &mut Game, loc: Point) {
    info!("moving player to {loc}");
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
    use StateMutators::*;
    match mesg {
        BeginReadTransaction(_) => (),
        EndReadTransaction(_) => (),
        _ => {
            if !game.read_transactions.is_empty() {
                game.queued_mutates.push(mesg);
                debug_assert!(game.read_transactions.len() < 5000); // sanity check
                return;
            }
        }
    }
    match mesg {
        BeginReadTransaction(id) => handle_begin_transaction(game, id),
        EndReadTransaction(id) => handle_end_transaction(game, id),
        MovePlayer(loc) => handle_move_player(game, loc),
        Reset(map) => handle_reset(game, &map),
    }
}
