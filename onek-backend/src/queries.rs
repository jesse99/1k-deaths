use super::*;

fn unseen_obj() -> Object {
    let mut object = Object::default();

    object.insert("id".to_owned(), Value::Id(Id("unseen".to_owned())));
    object.insert(
        "description".to_owned(),
        Value::String("You can't see there.".to_owned()),
    );
    object.insert("symbol".to_owned(), Value::Char('?'));
    object.insert("color".to_owned(), Value::Color(Color::White));
    object.insert("back_color".to_owned(), Value::Color(Color::Black));

    object
}

pub fn cell_at(game: &Game, loc: Point) -> Cell {
    if game.pov.visible(game, &loc) {
        let default = game.objects.get(&DEFAULT_CELL_ID).unwrap();
        let oids = game.level.get(&loc).unwrap();
        oids.iter()
            .map(|oid| game.objects.get(&oid).unwrap_or(default).clone())
            .collect()
    } else {
        match game.old_pov.get(&loc) {
            Some(cell) => cell.clone(),
            None => vec![unseen_obj()],
        }
    }
}

// These are public for testing.
pub fn handle_cell_at(game: &Game, loc: Point) -> StateResponse {
    let cell = cell_at(game, loc);
    StateResponse::Cell(cell)
}

pub fn handle_notes(game: &Game, count: usize) -> StateResponse {
    let start = if count < game.notes.len() {
        game.notes.len() - count
    } else {
        0
    };
    let notes = game.notes.range(start..).cloned().collect();
    StateResponse::Notes(notes)
}

pub fn handle_player_loc(game: &Game) -> StateResponse {
    StateResponse::Location(game.player_loc)
}

pub fn handle_player_view(game: &Game) -> StateResponse {
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

    StateResponse::Map(view)
}

pub fn handle_query(channel_name: ChannelName, game: &Game, mesg: StateQueries) {
    use StateQueries::*;
    let response = match mesg {
        CellAt(loc) => handle_cell_at(game, loc),
        Notes(count) => handle_notes(game, count),
        PlayerLoc() => handle_player_loc(game),
        PlayerView() => handle_player_view(game),
    };
    game.send_response(channel_name, response);
}
