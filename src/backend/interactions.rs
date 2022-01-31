//! This is where the bulk of the logic exists to handle interactions between
//! Characters and between items. It's structured as a lookup table of
//! (tag1, tag2) => handler. For example (Player, Sign) => function_to_print_sign.
use super::make;
use super::{Cell, Event, Game, Material, Message, Point, State, Tag, Topic};
use fnv::FnvHashMap;
use rand::prelude::*;

// ---- Helpers ------------------------------------------------------------
fn impassible_terrain_tag(tag: &Tag) -> Option<Message> {
    match tag {
        Tag::DeepWater => Some(Message::new(Topic::Failed, "The water is too deep.")),
        Tag::Tree => Some(Message::new(
            Topic::Failed,
            "The tree's are too thick to travel through.",
        )),
        Tag::Vitr => Some(Message::new(Topic::Failed, "Do you have a death wish?")),
        Tag::Wall => Some(Message::new(Topic::Failed, "You bump into the wall.")),
        _ => None,
    }
}

fn impassible_terrain(cell: &Cell) -> Option<Message> {
    let obj = cell.terrain();
    for tag in obj.iter() {
        let mesg = impassible_terrain_tag(tag);
        if mesg.is_some() {
            return mesg;
        }
    }
    None
}

fn find_empty_cell(game: &Game, loc: &Point) -> Option<Point> {
    let mut deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
    deltas.shuffle(&mut *game.rng());
    for delta in deltas {
        let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
        let cell = &game.level.get(&new_loc);
        if !cell.contains(&Tag::Character) && impassible_terrain(cell).is_none() {
            return Some(new_loc);
        }
    }
    None
}

fn damage_wall(game: &Game, loc: &Point, scaled_damage: i32, events: &mut Vec<Event>) {
    assert!(scaled_damage > 0);
    let cell = game.level.get(loc);
    let obj = cell.get(&Tag::Wall);
    let (current, max) = obj.durability().unwrap();
    let damage = max / scaled_damage;

    if damage < current {
        let mesg = Message::new(
            Topic::Normal,
            "You chip away at the wall with your pick-axe.", // TODO: probably should have slightly different text for wooden walls (if we ever add them)
        );
        events.push(Event::AddMessage(mesg));

        let mut obj = obj.clone();
        obj.replace(Tag::Durability {
            current: current - damage,
            max,
        });
        events.push(Event::ChangeObject(*loc, Tag::Wall, obj));
    } else {
        let mesg = Message::new(Topic::Important, "You destroy the wall!");
        events.push(Event::AddMessage(mesg));

        events.push(Event::DestroyObject(*loc, Tag::Wall));
    }
}

// ---- Interaction handlers -----------------------------------------------
fn emp_sword_vs_vitr(game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) -> bool {
    if !matches!(game.state, State::WonGame) {
        let mesg = Message::new(
            Topic::Important,
            "You carefully place the Emperor's sword into the vitr and watch it dissolve.",
        );
        events.push(Event::AddMessage(mesg));

        let mesg = Message::new(Topic::Important, "You have won the game!!");
        events.push(Event::AddMessage(mesg));
        events.push(Event::StateChanged(State::WonGame));
        true
    } else {
        false
    }
}

fn pick_axe_vs_wall(game: &Game, _player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let cell = game.level.get(new_loc);
    let obj = cell.get(&Tag::Wall);
    match obj.material() {
        // Some(Material::Wood) => damage_wall(game, new_loc, 3, events),
        Some(Material::Stone) => damage_wall(game, new_loc, 6, events),
        Some(Material::Metal) => {
            let mesg = Message::new(
                Topic::Normal,
                "Your pick-axe bounces off the metal wall doing no damage.",
            );
            events.push(Event::AddMessage(mesg));
        }
        None => panic!("Walls should always have a Material"),
    }
    true
}

fn player_vs_closed_door(_game: &Game, loc: &Point, events: &mut Vec<Event>) {
    events.push(Event::ChangeObject(*loc, Tag::ClosedDoor, make::open_door()));
}

fn player_vs_doorman(game: &Game, _player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let cell = game.level.get(&game.level.player());
    let obj = cell.get(&Tag::Character);
    match obj.inventory() {
        Some(items) if items.iter().any(|obj| obj.description.contains("Doom")) => {
            let mesg = Message::new(Topic::NPCSpeaks, "Ahh, a new champion for the Emperor!");
            events.push(Event::AddMessage(mesg));

            if let Some(to_loc) = find_empty_cell(game, new_loc) {
                events.push(Event::NPCMoved(*new_loc, to_loc));
            }
        }
        _ => {
            let mesg = Message::new(Topic::NPCSpeaks, "You are not worthy.");
            events.push(Event::AddMessage(mesg));
        }
    }
    true
}

fn player_vs_deep_water(_game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let mesg = impassible_terrain_tag(&Tag::DeepWater).unwrap();
    events.push(Event::AddMessage(mesg));
    true
}

fn player_vs_portable(_game: &Game, loc: &Point, events: &mut Vec<Event>) {
    events.push(Event::AddToInventory(*loc));
}

fn player_vs_rhulad(_game: &Game, player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let mesg = Message::new(Topic::Important, "After an epic battle you kill the Emperor!");
    events.push(Event::AddMessage(mesg));

    events.push(Event::DestroyObject(*new_loc, Tag::Character));
    events.push(Event::AddObject(*player_loc, super::make::emp_sword()));
    events.push(Event::AddToInventory(*player_loc));
    events.push(Event::StateChanged(State::KilledRhulad));
    true
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

fn player_vs_spectator(game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) -> bool {
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
    true
}

fn player_vs_tree(_game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let mesg = impassible_terrain_tag(&Tag::Tree).unwrap();
    events.push(Event::AddMessage(mesg));
    true
}

fn player_vs_vitr(_game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let mesg = impassible_terrain_tag(&Tag::Vitr).unwrap();
    events.push(Event::AddMessage(mesg));
    true
}

fn player_vs_wall(_game: &Game, _player_loc: &Point, _new_loc: &Point, events: &mut Vec<Event>) -> bool {
    let mesg = impassible_terrain_tag(&Tag::Wall).unwrap();
    events.push(Event::AddMessage(mesg));
    true
}

// ---- struct Interaction -------------------------------------------------
type PreHandler = fn(&Game, &Point, &Point, &mut Vec<Event>) -> bool;
type PostHandler = fn(&Game, &Point, &mut Vec<Event>);

// TODO:
// add support for pre-move handlers
// do we need any other handlers? or maybe just comment missing ones?
pub struct Interactions {
    pre_table: FnvHashMap<(i32, i32), PreHandler>,
    post_table: FnvHashMap<i32, PostHandler>,
}

impl Interactions {
    pub fn new() -> Interactions {
        let mut i = Interactions {
            pre_table: FnvHashMap::default(),
            post_table: FnvHashMap::default(),
        };

        i.pre_ins(Tag::EmpSword, Tag::Vitr, emp_sword_vs_vitr);
        i.pre_ins(Tag::PickAxe, Tag::Wall, pick_axe_vs_wall);
        i.pre_ins(Tag::Player, Tag::DeepWater, player_vs_deep_water);
        i.pre_ins(Tag::Player, Tag::Doorman, player_vs_doorman);
        i.pre_ins(Tag::Player, Tag::Rhulad, player_vs_rhulad);
        i.pre_ins(Tag::Player, Tag::Spectator, player_vs_spectator);
        i.pre_ins(Tag::Player, Tag::Tree, player_vs_tree);
        i.pre_ins(Tag::Player, Tag::Vitr, player_vs_vitr);
        i.pre_ins(Tag::Player, Tag::Wall, player_vs_wall);

        i.post_ins(Tag::ClosedDoor, player_vs_closed_door); // post moves are always (Player, X)
        i.post_ins(Tag::Portable, player_vs_portable);
        i.post_ins(Tag::ShallowWater, player_vs_shallow_water);
        i.post_ins(Tag::Sign, player_vs_sign);

        i
    }

    /// A Character is attempting to move to a new square and may need to do
    /// an interaction instead of a move (e.g. attack another Character or
    /// unlock a door). Typically only the topmost interactible object is
    /// interacted with.
    pub fn pre_move(
        &self,
        tag0: &Tag,
        tag1: &Tag,
        game: &Game,
        char_loc: &Point,
        new_loc: &Point,
        events: &mut Vec<Event>,
    ) -> bool {
        if let Some(handler) = self.pre_table.get(&(tag0.to_index(), tag1.to_index())) {
            handler(game, char_loc, new_loc, events)
        } else {
            false
        }
    }

    /// Player has moved into a new cell and now may need to interact with
    /// what is there. This could be the terrain itself (e.g. ShallowWater)
    /// or an object (e.g. a Sign). Typically all interactible objects in
    /// the new cell are interacted with.
    pub fn post_move(&self, tag1: &Tag, game: &Game, loc: &Point, events: &mut Vec<Event>) {
        if let Some(handler) = self.post_table.get(&tag1.to_index()) {
            handler(game, loc, events);
        }
    }

    fn pre_ins(&mut self, tag0: Tag, tag1: Tag, handler: PreHandler) {
        self.pre_table.insert((tag0.to_index(), tag1.to_index()), handler);
    }

    fn post_ins(&mut self, tag1: Tag, handler: PostHandler) {
        self.post_table.insert(tag1.to_index(), handler);
    }
}
