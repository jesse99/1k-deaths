use super::*;

fn cell_at(game: &Game, loc: Point) -> Cell {
    if game.pov.visible(game, &loc) {
        // info!("level: {:?}", game.level);
        // info!("objects: {:?}", game.objects);
        // info!("player_loc: {:?}", game.player_loc);
        // info!("next_id: {:?}", game.next_id);
        let default = game.objects.get(&DEFAULT_CELL_ID).unwrap();
        let oids = game.level.get(&loc).unwrap();
        oids.iter()
            .map(|oid| game.objects.get(&oid).unwrap_or(default).clone())
            .collect()
    } else {
        match game.old_pov.get(&loc) {
            Some(cell) => cell.clone(),
            None => vec![],
        }
    }
}

fn handle_cell_at(game: &Game, name: ChannelName, loc: Point) {
    let cell = cell_at(game, loc);
    let response = StateResponse::Cell(cell);
    game.send_response(name, response);
}

fn handle_notes(game: &Game, name: ChannelName, count: usize) {
    let start = if count < game.notes.len() {
        game.notes.len() - count
    } else {
        0
    };
    let notes = game.notes.range(start..).cloned().collect();
    let response = StateResponse::Notes(notes);
    game.send_response(name, response);
}

fn handle_player_loc(game: &Game, name: ChannelName) {
    let response = StateResponse::Location(game.player_loc);
    game.send_response(name, response);
}

fn handle_player_view(game: &Game, name: ChannelName) {
    let mut view = View::new();

    let start_loc = Point::new(
        game.player_loc.x - super::pov::RADIUS,
        game.player_loc.y - super::pov::RADIUS,
    );
    for dy in 0..2 * super::pov::RADIUS {
        for dx in 0..2 * super::pov::RADIUS {
            let loc = Point::new(start_loc.x + dx, start_loc.y + dy);
            let cell = cell_at(game, loc);
            view.insert(loc, cell);
        }
    }

    let response = StateResponse::Map(view);
    game.send_response(name, response);
}

pub fn handle_query(channel_name: ChannelName, game: &Game, mesg: StateQueries) {
    use StateQueries::*;
    match mesg {
        CellAt(loc) => handle_cell_at(game, channel_name, loc),
        Notes(count) => handle_notes(game, channel_name, count),
        PlayerLoc() => handle_player_loc(game, channel_name),
        PlayerView() => handle_player_view(game, channel_name),
    }
}
