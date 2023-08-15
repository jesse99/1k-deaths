use ipmpsc::{Receiver, Sender, SharedRingBuffer};
use onek_types::*;

mod game;
mod mutators;
mod queries;

use game::*;
use mutators::*;
use queries::*;

fn create_sender(name: &ChannelName) -> ipmpsc::Sender {
    match SharedRingBuffer::open(name.as_str()) {
        Ok(buffer) => Sender::new(buffer),
        Err(err) => panic!("error opening sender: {err:?}"),
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
