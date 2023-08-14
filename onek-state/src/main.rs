use ipmpsc::{Receiver, Sender, SharedRingBuffer};
use onek_types::*;
use std::collections::HashMap;

struct Game {
    terrain: HashMap<Point, Terrain>,
    player_loc: Point,
    reply_senders: HashMap<ChannelName, ipmpsc::Sender>,
}

impl Game {
    fn new() -> Game {
        Game {
            terrain: HashMap::new(),
            player_loc: Point::new(0, 0),
            reply_senders: HashMap::new(),
        }
    }
}

fn char_to_terrain(ch: char) -> Option<Terrain> {
    match ch {
        '#' => Some(Terrain::Wall),
        ' ' => Some(Terrain::Dirt),
        _ => None,
    }
}

fn handle_reset(game: &mut Game, map: &str) {
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

fn create_sender(name: &ChannelName) -> ipmpsc::Sender {
    match SharedRingBuffer::open(name.as_str()) {
        Ok(buffer) => Sender::new(buffer),
        Err(err) => panic!("error opening sender: {err:?}"),
    }
}
fn handle_mutate(game: &mut Game, mesg: StateMutators) {
    match mesg {
        StateMutators::MovePlayer(loc) => game.player_loc = loc,
        StateMutators::Reset(map) => handle_reset(game, &map),
    }
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

    match game.reply_senders.get(&name) {
        Some(tx) => {
            let mesg = StateResponse::Map(view);
            let result = tx.send(&mesg);
            assert!(!result.is_err(), "error sending reply: {result:?}");
        }
        None => panic!("failed to find {name} reply sender"),
    }
}

// TODO: should have modules for queries and mutators
fn handle_query(game: &Game, mesg: StateQueries) {
    match mesg {
        StateQueries::PlayerView(channel_name) => handle_player_view(game, channel_name),
    }
}

fn handle_mesg(game: &mut Game, mesg: StateMessages) {
    match mesg {
        StateMessages::Mutate(mesg) => handle_mutate(game, mesg),
        StateMessages::Query(mesg) => handle_query(game, mesg),
        StateMessages::RegisterForQuery(channel_name) => {
            let sender = create_sender(&channel_name);
            game.reply_senders.insert(channel_name, sender);
        }
        StateMessages::RegisterForUpdate(_channel_name) => println!("RegisterForUpdate isn't implemented yet"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map_file = "/tmp/state-sink";
    let rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);

    let mut game = Game::new();

    loop {
        match rx.recv() {
            // TODO: do we want zero-copy?
            Ok(mesg) => handle_mesg(&mut game, mesg),
            Err(err) => {
                println!("rx error: {err}");
                return Result::Err(Box::new(err));
            }
        }
    }
}
