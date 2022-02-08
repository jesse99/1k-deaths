//! This is where the bulk of the logic exists to handle interactions between
//! Characters and between items. It's structured as a lookup table of
//! (tag1, tag2) => handler. For example (Player, Sign) => function_to_print_sign.
use super::object::TagValue;
use super::tag::*;
use super::{Event, Game, Material, Message, Object, Oid, Point, ScheduledAction, State, Tag, Topic};
use fnv::FnvHashMap;
use rand::prelude::*;

// ---- struct Interaction -------------------------------------------------
type PreHandler = fn(&Game, &Point, &Point, &mut Vec<Event>);
type PostHandler = fn(&Game, &Point, &mut Vec<Event>);

// TODO:
// do we need any other handlers? or maybe just comment missing ones?
pub struct Interactions {
    pre_table: FnvHashMap<(Tid, Tid), PreHandler>,
    post_table: FnvHashMap<Tid, PostHandler>,
}

impl Interactions {
    pub fn new() -> Interactions {
        let mut i = Interactions {
            pre_table: FnvHashMap::default(),
            post_table: FnvHashMap::default(),
        };

        i.pre_ins(EMP_SWORD_ID, VITR_ID, emp_sword_vs_vitr);
        i.pre_ins(PICK_AXE_ID, WALL_ID, pick_axe_vs_wall);
        i.pre_ins(PLAYER_ID, DEEP_WATER_ID, player_vs_deep_water);
        i.pre_ins(PLAYER_ID, DOORMAN_ID, player_vs_doorman);
        i.pre_ins(PLAYER_ID, RHULAD_ID, player_vs_rhulad);
        i.pre_ins(PLAYER_ID, SPECTATOR_ID, player_vs_spectator);
        i.pre_ins(PLAYER_ID, TREE_ID, player_vs_tree);
        i.pre_ins(PLAYER_ID, VITR_ID, player_vs_vitr);
        i.pre_ins(PLAYER_ID, WALL_ID, player_vs_wall);
        i.pre_ins(PLAYER_ID, CLOSED_DOOR_ID, player_vs_closed_door);

        i.post_ins(PORTABLE_ID, player_vs_portable); // post moves are always (Player, X)
        i.post_ins(SHALLOW_WATER_ID, player_vs_shallow_water);
        i.post_ins(SIGN_ID, player_vs_sign);

        i
    }

    /// A Character is attempting to move to a new square and may need to do
    /// an interaction instead of a move (e.g. attack another Character or
    /// unlock a door). Typically only the topmost interactible object is
    /// interacted with.
    pub fn scheduled_interaction(
        &self,
        tag0: &Tag,
        tag1: &Tag,
        game: &Game,
        char_loc: &Point,
        new_loc: &Point,
        events: &mut Vec<Event>,
    ) -> bool {
        if let Some(handler) = self.pre_table.get(&(tag0.to_id(), tag1.to_id())) {
            handler(game, char_loc, new_loc, events);
            true
        } else {
            false
        }
    }

    /// Player has moved into a new cell and now may need to interact with
    /// what is there. This could be the terrain itself (e.g. ShallowWater)
    /// or an object (e.g. a Sign). Typically all interactible objects in
    /// the new cell are interacted with.
    pub fn post_move(&self, tag1: &Tag, game: &Game, loc: &Point, events: &mut Vec<Event>) {
        if let Some(handler) = self.post_table.get(&tag1.to_id()) {
            handler(game, loc, events);
        }
    }

    fn pre_ins(&mut self, id0: Tid, id1: Tid, handler: PreHandler) {
        self.pre_table.insert((id0, id1), handler);
    }

    fn post_ins(&mut self, id1: Tid, handler: PostHandler) {
        self.post_table.insert(id1, handler);
    }
}

// ---- Helpers ------------------------------------------------------------
fn impassible_terrain_tag(ch: &Object, tag: &Tag) -> Option<Message> {
    match tag {
        Tag::DeepWater => Some(Message::new(Topic::Failed, "The water is too deep.")),
        Tag::Tree => Some(Message::new(
            Topic::Failed,
            "The tree's are too thick to travel through.",
        )),
        Tag::Vitr => Some(Message::new(Topic::Failed, "Do you have a death wish?")),
        Tag::Wall => Some(Message::new(Topic::Failed, "You bump into the wall.")),
        Tag::ClosedDoor if !ch.has(CAN_OPEN_DOOR_ID) => Some(Message::new(Topic::Failed, "You fail to open the door.")),
        _ => None,
    }
}

pub fn impassible_terrain(ch: &Object, terrain: &Object) -> Option<Message> {
    for tag in terrain.iter() {
        let mesg = impassible_terrain_tag(ch, tag);
        if mesg.is_some() {
            return mesg;
        }
    }
    None
}

fn find_empty_cell(game: &Game, ch: &Object, loc: &Point) -> Option<Point> {
    let mut deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
    deltas.shuffle(&mut *game.rng());
    for delta in deltas {
        let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
        let character = &game.get(&new_loc, CHARACTER_ID);
        if character.is_none() {
            let (_, terrain) = game.get_bottom(&new_loc);
            if impassible_terrain(ch, terrain).is_none() {
                return Some(new_loc);
            }
        }
    }
    None
}

// ---- Interaction handlers -----------------------------------------------
fn emp_sword_vs_vitr(game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) {
    if !matches!(game.state, State::WonGame) {
        let mesg = Message::new(
            Topic::Important,
            "You carefully place the Emperor's sword into the vitr and watch it dissolve.",
        );
        events.push(Event::AddMessage(mesg));

        let mesg = Message::new(Topic::Important, "You have won the game!!");
        events.push(Event::AddMessage(mesg));
        events.push(Event::StateChanged(State::WonGame));
    }
}

fn pick_axe_vs_wall(game: &Game, _player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) {
    let (oid, obj) = game.get(new_loc, WALL_ID).unwrap();
    let material: Option<Material> = obj.value(MATERIAL_ID);
    match material {
        Some(Material::Stone) => {
            let saction = ScheduledAction::DamageWall(*new_loc, oid);
            events.push(Event::ScheduledAction(Oid(0), saction));
        }
        Some(Material::Metal) => {
            let mesg = Message::new(
                Topic::Normal,
                "Your pick-axe bounces off the metal wall doing no damage.",
            );
            events.push(Event::AddMessage(mesg));
        }
        None => panic!("Walls should always have a Material"),
    }
}

fn player_vs_doorman(game: &Game, _player_loc: &Point, doorman_loc: &Point, events: &mut Vec<Event>) {
    if game
        .player_inv_iter()
        .any(|(_, obj)| obj.description().contains("Doom"))
    {
        let (oid, doorman) = game.get(doorman_loc, DOORMAN_ID).unwrap();
        if let Some(to_loc) = find_empty_cell(game, doorman, doorman_loc) {
            let saction = ScheduledAction::ShoveDoorman(*doorman_loc, oid, to_loc);
            events.push(Event::ScheduledAction(Oid(0), saction));
        }
    } else {
        let mesg = Message::new(Topic::NPCSpeaks, "You are not worthy.");
        events.push(Event::AddMessage(mesg));
    }
}

fn player_vs_deep_water(game: &Game, player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = impassible_terrain_tag(player, &Tag::DeepWater).unwrap();
    events.push(Event::AddMessage(mesg));
}

fn player_vs_portable(game: &Game, loc: &Point, events: &mut Vec<Event>) {
    let oid = game.get(loc, PORTABLE_ID).unwrap().0;
    let saction = ScheduledAction::PickUp(*loc, oid);
    events.push(Event::ScheduledAction(Oid(0), saction));
}

fn player_vs_rhulad(game: &Game, _player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) {
    let oid = game.get(new_loc, CHARACTER_ID).unwrap().0;
    let saction = ScheduledAction::FightRhulad(*new_loc, oid);
    events.push(Event::ScheduledAction(Oid(0), saction));
}

fn player_vs_shallow_water(_game: &Game, _loc: &Point, events: &mut Vec<Event>) {
    let mesg = Message::new(Topic::Normal, "You splash through the water.");
    events.push(Event::AddMessage(mesg));
}

fn player_vs_sign(game: &Game, loc: &Point, events: &mut Vec<Event>) {
    let (_, obj) = game.get(loc, SIGN_ID).unwrap();
    let mesg = Message {
        topic: Topic::Normal,
        text: format!("You see a sign {}.", obj.description()),
    };
    events.push(Event::AddMessage(mesg));
}

fn player_vs_spectator(game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) {
    let messages = if matches!(game.state, State::Adventuring) {
        vec![
            "I hope you're prepared to die!",
            "The last champion only lasted thirty seconds.",
            "How can you defeat a man who will not stay dead?",
            "I have 10 gold on you lasting over two minutes!",
            "You're just another dead man walking.",
        ]
    } else {
        vec![
            "I can't believe that the Emperor is dead.",
            "You're my hero!",
            "You've done the impossible!",
        ]
    };
    let text = messages.iter().choose(&mut *game.rng()).unwrap();

    let mesg = Message::new(Topic::NPCSpeaks, text);
    events.push(Event::AddMessage(mesg));
}

fn player_vs_tree(game: &Game, player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = impassible_terrain_tag(player, &Tag::Tree).unwrap();
    events.push(Event::AddMessage(mesg));
}

fn player_vs_vitr(game: &Game, player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = impassible_terrain_tag(player, &Tag::Vitr).unwrap();
    events.push(Event::AddMessage(mesg));
}

fn player_vs_wall(game: &Game, player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = impassible_terrain_tag(player, &Tag::Wall).unwrap();
    events.push(Event::AddMessage(mesg));
}

fn player_vs_closed_door(game: &Game, player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) {
    let oid = game.get(new_loc, CLOSED_DOOR_ID).unwrap().0;
    let saction = ScheduledAction::OpenDoor(*player_loc, *new_loc, oid);
    events.push(Event::ScheduledAction(Oid(0), saction));
}
