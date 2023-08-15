use onek_types::*;
use std::collections::HashMap;

pub struct Game {
    pub terrain: HashMap<Point, Terrain>,
    pub player_loc: Point,
    pub reply_senders: HashMap<ChannelName, ipmpsc::Sender>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            terrain: HashMap::new(),
            player_loc: Point::new(0, 0),
            reply_senders: HashMap::new(),
        }
    }
}
