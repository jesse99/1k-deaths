//! This is where the bulk of the logic exists to handle interactions between
//! Characters and between items. It's structured as a lookup table of
//! (tag1, tag2) => handler. For example (Player, Sign) => function_to_print_sign.
#![allow(dead_code)] // TODO: remove this
use super::{Event, Game, Message, Point, Tag, Topic};
use fnv::FnvHashMap;

// ---- Interaction handlers -----------------------------------------------
fn player_vs_portable(_game: &Game, loc: &Point, events: &mut Vec<Event>) {
    events.push(Event::AddToInventory(*loc));
}

fn player_vs_shallow_water(_game: &Game, _loc: &Point, events: &mut Vec<Event>) {
    let mesg = Message::new(Topic::Normal, "You splash through the water.");
    events.push(Event::AddMessage(mesg));
}

fn player_vs_sign(game: &Game, loc: &Point, events: &mut Vec<Event>) {
    let cell = game.level.get(loc);
    let obj = cell.get(&Tag::Sign);
    let text = obj.sign().unwrap();
    let mesg = Message {
        topic: Topic::Normal,
        text: format!("You see a sign {text}."),
    };
    events.push(Event::AddMessage(mesg));
}

// ---- struct Interaction -------------------------------------------------
type PostHandler = fn(&Game, &Point, &mut Vec<Event>);

// TODO:
// add support for pre-move handlers
// do we need any other handlers? or maybe just comment missing ones?
pub struct Interactions {
    post_table: FnvHashMap<(i32, i32), PostHandler>,
}

impl Interactions {
    pub fn new() -> Interactions {
        let mut i = Interactions {
            post_table: FnvHashMap::default(),
        };

        i.post_ins(Tag::Player, Tag::Portable, player_vs_portable);
        i.post_ins(Tag::Player, Tag::ShallowWater, player_vs_shallow_water);
        i.post_ins(Tag::Player, Tag::Sign, player_vs_sign);

        i
    }

    /// Player has moved into a new cell and now may need to interact with
    /// what is there. This could be the terrain itself (e.g. ShallowWater)
    /// or an object (e.g. a Sign).
    pub fn post_move(
        &self,
        tag0: &Tag,
        tag1: &Tag,
        game: &Game,
        loc: &Point,
        events: &mut Vec<Event>,
    ) {
        if let Some(handler) = self.post_table.get(&(tag0.to_index(), tag1.to_index())) {
            handler(game, loc, events);
        }
    }

    fn post_ins(&mut self, tag0: Tag, tag1: Tag, handler: PostHandler) {
        self.post_table
            .insert((tag0.to_index(), tag1.to_index()), handler);
    }
}
