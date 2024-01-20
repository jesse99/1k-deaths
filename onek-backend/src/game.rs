use super::{OldPoV, PoV};
use onek_shared::*;
use std::collections::{HashMap, VecDeque};

pub struct Game {
    pub level: HashMap<Point, Vec<Oid>>,
    pub objects: HashMap<Oid, Object>,
    pub player_loc: Point,
    pub notes: VecDeque<Note>,
    pub pov: PoV,        // locations that the player can currently see
    pub old_pov: OldPoV, // locations that the user has seen in the past (this will often be stale data)
    pub reply_senders: HashMap<ChannelName, ipmpsc::Sender>,
    pub read_transactions: Vec<String>,
    pub queued_mutates: Vec<StateMutators>,
    pub next_id: u32, // 0 is null, 1 is the player, 2 is default terrain
    exemplars: HashMap<Id, Object>,
}

impl Game {
    pub fn new() -> Game {
        let mut game = Game {
            level: HashMap::new(),
            objects: HashMap::new(),
            exemplars: super::objects::load_objects(),
            next_id: 1,
            player_loc: Point::new(0, 0),
            notes: VecDeque::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
            reply_senders: HashMap::new(),
            read_transactions: Vec::new(),
            queued_mutates: Vec::new(),
        };
        game.new_object("player"); // player
        game.new_object("stone wall"); // default terrain
        game
    }

    pub fn send_response(&self, name: ChannelName, response: StateResponse) {
        match self.reply_senders.get(&name) {
            Some(tx) => {
                match response {
                    StateResponse::Map(_) => debug!("sending Map(...) to {name}"),
                    _ => debug!("sending {response} to {name}"),
                }
                let result = tx.send(&response);
                assert!(!result.is_err(), "error sending reply: {result:?}");
            }
            None => panic!("failed to find {name} reply sender"),
        }
    }

    pub fn new_object(&mut self, id: &str) -> Oid {
        let oid = Oid::new(&id, self.next_id);
        let mut object = self.exemplars.get(&Id(id.to_owned())).unwrap().clone();
        object.insert("oid".to_owned(), Value::Oid(oid));
        self.objects.insert(oid, object);
        assert!(self.next_id < std::u32::MAX);
        self.next_id += 1;
        oid
    }

    pub fn remove_oid(&mut self, from: Point, oid: Oid) {
        let oids = self.level.get_mut(&from).unwrap();
        let index = oids.iter().position(|&candidate| candidate == oid).unwrap();
        oids.remove(index);
    }

    pub fn append_oid(&mut self, to: Point, oid: Oid) {
        let oids = self.level.get_mut(&to).unwrap();
        oids.push(oid);
    }
}
