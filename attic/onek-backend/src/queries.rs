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
            let default = game.objects.get(&DEFAULT_CELL_OID).unwrap();
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

// pub fn handle_player_view(game: &Game) -> StateResponse {
//     let mut view = View::new();

//     let start_loc = Point::new(
//         game.player_loc.x - super::pov::RADIUS,
//         game.player_loc.y - super::pov::RADIUS,
//     );
//     for dy in 0..2 * super::pov::RADIUS {
//         for dx in 0..2 * super::pov::RADIUS {
//             let loc = Point::new(start_loc.x + dx, start_loc.y + dy);
//             let cell = visible_cell(game, loc);
//             view.insert(loc, cell);
//         }
//     }

//     StateResponse::Map(view)
// }

fn cell_to_terminal(cell: &Cell) -> (char, Color, Color) {
    let bg = cell[0].get("back_color").unwrap().to_color();
    if cell.len() == 1 {
        let symbol = cell[0].get("symbol").unwrap().to_char();
        let fg = cell[0].get("color").unwrap().to_color();
        (symbol, fg, bg)
    } else {
        // TODO: would be nice to do something extra when there are multiple objects
        let symbol = cell.last().unwrap().get("symbol").unwrap().to_char();
        let fg = cell.last().unwrap().get("color").unwrap().to_color();
        (symbol, fg, bg)
    }
}

fn get_tcell(game: &Game, loc: Point) -> TerminalCell {
    if game.pov.visible(game, &loc) {
        if let Some(oids) = game.level.get(&loc) {
            // Cell is visible and there's something there. TODO: could optimize this a
            // bit: really need only the first and last object
            let cell = oids.iter().map(|oid| game.objects.get(&oid).unwrap().clone()).collect();
            let (symbol, fg, bg) = cell_to_terminal(&cell);
            TerminalCell::Seen {
                symbol: symbol,
                color: fg,
                back_color: bg,
            }
        } else {
            // Cell is visible but there's nothing at that loc.
            let object = game.objects.get(&DEFAULT_CELL_OID).unwrap();
            let symbol = object.get("symbol").unwrap().to_char();
            let fg = object.get("color").unwrap().to_color();
            let bg = object.get("back_color").unwrap().to_color();
            TerminalCell::Seen {
                symbol: symbol,
                color: fg,
                back_color: bg,
            }
        }
    } else {
        match game.old_pov.get(&loc) {
            // Cell was visible but now its state is stale.
            Some(cell) => {
                let (symbol, _, bg) = cell_to_terminal(cell);
                TerminalCell::Stale {
                    symbol: symbol,
                    back_color: bg,
                }
            }

            // Cell was never visible.
            None => TerminalCell::Unseen,
        }
    }
}

// This is  bottle neck for the terminal so we return a special compact type
// (TerminalCell instead of Object) and also run length encode the result.
pub fn handle_terminal_row(game: &Game, start: Point, len: i32) -> StateResponse {
    let mut row = Vec::new();

    if len > 0 {
        let mut prev_cell = get_tcell(game, start);
        let mut prev_count = 1;
        for dx in 1..len {
            let loc = Point::new(start.x + dx, start.y);
            let cell = get_tcell(game, loc);
            if cell == prev_cell {
                prev_count += 1;
            } else {
                row.push((prev_cell, prev_count));
                prev_cell = cell;
                prev_count = 1;
            }
        }
        row.push((prev_cell, prev_count));
    }

    StateResponse::TerminalRow(TerminalRow { row })
}

pub fn handle_query(channel_name: ChannelName, game: &Game, mesg: StateQueries) {
    use StateQueries::*;
    let response = match mesg {
        CellAt(loc) => handle_cell_at(game, loc),
        Notes(count) => handle_notes(game, count),
        PlayerLoc => handle_player_loc(game),
        TerminalRow { start, len } => handle_terminal_row(game, start, len),
    };
    game.send_response(channel_name, response);
}
