use super::*;

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

    match game.reply_senders.get(&name) {
        Some(tx) => {
            let mesg = StateResponse::Map(view);
            let result = tx.send(&mesg);
            assert!(!result.is_err(), "error sending reply: {result:?}");
        }
        None => panic!("failed to find {name} reply sender"),
    }
}

pub fn handle_query(game: &Game, mesg: StateQueries) {
    match mesg {
        StateQueries::PlayerView(channel_name) => handle_player_view(game, channel_name),
    }
}
