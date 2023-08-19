use super::*;

fn send_response(game: &Game, name: ChannelName, response: StateResponse) {
    match game.reply_senders.get(&name) {
        Some(tx) => {
            debug!("sending {response}");
            let result = tx.send(&response);
            assert!(!result.is_err(), "error sending reply: {result:?}");
        }
        None => panic!("failed to find {name} reply sender"),
    }
}

fn handle_player_loc(game: &Game, name: ChannelName) {
    let response = StateResponse::Location(game.player_loc);
    send_response(game, name, response);
}

fn handle_player_view(game: &Game, name: ChannelName) {
    let mut view = View::new();

    for (&loc, &terrain) in game.terrain.iter() {
        // TODO: need to handle LOS
        let cell = if loc == game.player_loc {
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
        };
        view.insert(loc, cell);
    }

    let response = StateResponse::Map(view);
    send_response(game, name, response);
}

pub fn handle_query(game: &Game, mesg: StateQueries) {
    use StateQueries::*;
    match mesg {
        PlayerLoc(channel_name) => handle_player_loc(game, channel_name),
        PlayerView(channel_name) => handle_player_view(game, channel_name),
    }
}
