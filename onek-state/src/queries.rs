use super::*;

fn cell_at(game: &Game, loc: Point) -> Cell {
    let terrain = *game.terrain.get(&loc).unwrap_or(&game.default_terrain);
    if loc == game.player_loc {
        Cell {
            terrain,
            objects: Vec::new(),
            character: Some(PLAYER_ID),
        }
    } else {
        Cell {
            terrain,
            objects: Vec::new(),
            character: None,
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

    // TODO: need to handle LOS
    for (&loc, _) in game.terrain.iter() {
        let cell = cell_at(game, loc);
        view.insert(loc, cell);
    }

    let response = StateResponse::Map(view);
    game.send_response(name, response);
}

pub fn handle_query(game: &Game, mesg: StateQueries) {
    use StateQueries::*;
    match mesg {
        CellAt(channel_name, loc) => handle_cell_at(game, channel_name, loc),
        Notes(channel_name, count) => handle_notes(game, channel_name, count),
        PlayerLoc(channel_name) => handle_player_loc(game, channel_name),
        PlayerView(channel_name) => handle_player_view(game, channel_name),
    }
}
