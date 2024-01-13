use onek_types::*;
use std::collections::{HashMap, VecDeque};

pub struct Game {
    pub terrain: HashMap<Point, Terrain>,
    pub default_terrain: Terrain,
    pub player_loc: Point,
    pub notes: VecDeque<Note>,
    pub reply_senders: HashMap<ChannelName, ipmpsc::Sender>,
    pub read_transactions: Vec<String>,
    pub queued_mutates: Vec<StateMutators>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            terrain: HashMap::new(),
            default_terrain: Terrain::Wall,
            player_loc: Point::new(0, 0),
            notes: VecDeque::new(),
            reply_senders: HashMap::new(),
            read_transactions: Vec::new(),
            queued_mutates: Vec::new(),
        }
    }

    pub fn send_response(&self, name: ChannelName, response: StateResponse) {
        match self.reply_senders.get(&name) {
            Some(tx) => {
                debug!("sending {response} to {name}");
                let result = tx.send(&response);
                assert!(!result.is_err(), "error sending reply: {result:?}");
            }
            None => panic!("failed to find {name} reply sender"),
        }
    }
}
