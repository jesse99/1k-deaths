use super::*;

fn unseen_obj() -> Object {
    let mut object = Object::default();

    object.insert("tag".to_owned(), Value::Tag(Tag("unseen".to_owned())));
    object.insert(
        "description".to_owned(),
        Value::String("You can't see there.".to_owned()),
    );
    object.insert("symbol".to_owned(), Value::Char('?'));
    object.insert("color".to_owned(), Value::Color(Color::White));
    object.insert("back_color".to_owned(), Value::Color(Color::Black));

    object
}

/// Note that this returns the visible cell at loc and will always return something
/// though it may return an object with tag "unseen".
pub fn visible_cell(game: &Game, loc: Point) -> Cell {
    if game.pov.visible(game, &loc) {
        if let Some(oids) = game.level.get(&loc) {
            // Cell is visible and there's something there.
            oids.iter().map(|oid| game.objects.get(&oid).unwrap().clone()).collect()
        } else {
            // Cell is visible but there's nothing at that loc.
            let default = game.objects.get(&DEFAULT_CELL_ID).unwrap();
            vec![default.clone()]
        }
    } else {
        match game.old_pov.get(&loc) {
            // Cell was visible but now its state is stale.
            Some(cell) => cell.clone(),

            // Cell was never visible.
            None => vec![unseen_obj()],
        }
    }
}

/// Returns the actual cell at loc (if present). Note that this ignores PoV.
pub fn logical_cell(game: &Game, loc: Point) -> Option<Cell> {
    if let Some(oids) = game.level.get(&loc) {
        Some(oids.iter().map(|oid| game.objects.get(&oid).unwrap().clone()).collect())
    } else {
        None
    }
}

// These are public for testing.
pub fn handle_cell_at(game: &Game, loc: Point) -> StateResponse {
    let cell = visible_cell(game, loc);
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
            let cell = visible_cell(game, loc);
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
