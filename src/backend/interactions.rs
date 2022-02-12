//! This is where the bulk of the logic exists to handle interactions between
//! Characters and between items. It's structured as a lookup table of
//! (tag1, tag2) => handler. For example (Player, Sign) => function_to_print_sign.
use super::object::TagValue;
use super::tag::*;
use super::*;
use fnv::FnvHashMap;
use rand::prelude::*;

// ---- struct Interaction -------------------------------------------------
pub type PreHandler = fn(&mut Game, &Point, &Point) -> Option<Time>;
pub type PostHandler = fn(&mut Game, &Point) -> Time;

// TODO:
// do we need any other handlers? or maybe just comment missing ones?
pub struct Interactions {
    pre_table: FnvHashMap<(Tid, Tid), PreHandler>,
    post_table: FnvHashMap<(Tid, Tid), PostHandler>,
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

        i.post_ins(PLAYER_ID, PORTABLE_ID, player_vs_portable);
        i.post_ins(PLAYER_ID, SHALLOW_WATER_ID, player_vs_shallow_water);
        i.post_ins(PLAYER_ID, SIGN_ID, player_vs_sign);

        i
    }

    /// Something may want to interact with something else in a neighboring cell, e.g. tag
    ///  == PLAYER_ID and tag1 == CLOSED_DOOR_ID is used when the player attempts to open
    /// a door. PreHandler a duration if an interaction happened.
    pub fn find_interact_handler(&self, tag0: &Tag, tag1: &Tag) -> Option<PreHandler> {
        // It'd be nicer to actually do the interaction here but the borrow checker makes
        // that difficult.
        self.pre_table.get(&(tag0.to_id(), tag1.to_id())).copied()
    }

    /// The player or an NPC has moved into a new cell and may need to interact with
    /// what's there, e.g. a trap. Returns a time if the player's move duration should
    /// be extended. Typically all interactible objects in the new cell are interacted with.
    pub fn find_post_handler(&self, tag0: &Tag, tag1: &Tag) -> Option<&PostHandler> {
        self.post_table.get(&(tag0.to_id(), tag1.to_id()))
    }

    fn pre_ins(&mut self, id0: Tid, id1: Tid, handler: PreHandler) {
        self.pre_table.insert((id0, id1), handler);
    }

    fn post_ins(&mut self, id0: Tid, id1: Tid, handler: PostHandler) {
        self.post_table.insert((id0, id1), handler);
    }
}

// ---- Pre-move handlers ----------------------------------------------------------------
fn emp_sword_vs_vitr(game: &mut Game, _player_loc: &Point, _new_loc: &Point) -> Option<Time> {
    if !matches!(game.state, State::WonGame) {
        let mesg = Message::new(
            Topic::Important,
            "You carefully place the Emperor's sword into the vitr and watch it dissolve.",
        );
        game.messages.push(mesg);

        let mesg = Message::new(Topic::Important, "You have won the game!!");
        game.messages.push(mesg);
        game.state = State::WonGame;
        Some(time::DESTROY_EMP_SWORD)
    } else {
        None
    }
}

fn pick_axe_vs_wall(game: &mut Game, _player_loc: &Point, new_loc: &Point) -> Option<Time> {
    let (oid, obj) = game.get(new_loc, WALL_ID).unwrap();
    let material: Option<Material> = obj.value(MATERIAL_ID);
    match material {
        Some(Material::Stone) => {
            let damage = 6;
            game.do_dig(Oid(0), new_loc, oid, damage);
            Some(time::DIG_STONE)
        }
        Some(Material::Metal) => {
            let mesg = Message::new(
                Topic::Normal,
                "Your pick-axe bounces off the metal wall doing no damage.",
            );
            game.messages.push(mesg);
            Some(time::SCRATCH_METAL)
        }
        None => panic!("Walls should always have a Material"),
    }
}

fn player_vs_deep_water(game: &mut Game, player_loc: &Point, _new_loc: &Point) -> Option<Time> {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = player.impassible_terrain_tag(&Tag::DeepWater).unwrap();
    game.messages.push(mesg);
    Some(Time::zero())
}

fn player_vs_doorman(game: &mut Game, _player_loc: &Point, doorman_loc: &Point) -> Option<Time> {
    if game
        .player_inv_iter()
        .any(|(_, obj)| obj.description().contains("Doom"))
    {
        let (oid, doorman) = game.get(doorman_loc, DOORMAN_ID).unwrap();
        if let Some(to_loc) = game.find_empty_cell(doorman, doorman_loc) {
            game.do_shove_doorman(Oid(0), doorman_loc, oid, &to_loc);
            Some(time::SHOVE_DOORMAN)
        } else {
            Some(Time::zero())
        }
    } else {
        let mesg = Message::new(Topic::NPCSpeaks, "You are not worthy.");
        game.messages.push(mesg);
        Some(Time::zero())
    }
}

fn player_vs_rhulad(game: &mut Game, _player_loc: &Point, new_loc: &Point) -> Option<Time> {
    let oid = game.get(new_loc, CHARACTER_ID).unwrap().0;
    game.do_fight_rhulad(Oid(0), new_loc, oid);
    Some(time::FIGHT_RHULAD)
}

fn player_vs_spectator(game: &mut Game, _player_loc: &Point, _new_loc: &Point) -> Option<Time> {
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
    game.messages.push(mesg);
    Some(time::SPEAK_TO_SPECTATOR)
}

fn player_vs_tree(game: &mut Game, player_loc: &Point, _new_loc: &Point) -> Option<Time> {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = player.impassible_terrain_tag(&Tag::Tree).unwrap();
    game.messages.push(mesg);
    Some(Time::zero())
}

fn player_vs_vitr(game: &mut Game, player_loc: &Point, _new_loc: &Point) -> Option<Time> {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = player.impassible_terrain_tag(&Tag::Vitr).unwrap();
    game.messages.push(mesg);
    Some(Time::zero())
}

fn player_vs_wall(game: &mut Game, player_loc: &Point, _new_loc: &Point) -> Option<Time> {
    let player = game.get(player_loc, PLAYER_ID).unwrap().1;
    let mesg = player.impassible_terrain_tag(&Tag::Wall).unwrap();
    game.messages.push(mesg);
    Some(Time::zero())
}

fn player_vs_closed_door(game: &mut Game, player_loc: &Point, new_loc: &Point) -> Option<Time> {
    let oid = game.get(new_loc, CLOSED_DOOR_ID).unwrap().0;
    game.do_open_door(Oid(0), player_loc, new_loc, oid);
    Some(time::OPEN_DOOR)
}

// ---- Post-move handlers ---------------------------------------------------------------
fn player_vs_portable(game: &mut Game, loc: &Point) -> Time {
    let oid = game.get(loc, PORTABLE_ID).unwrap().0;
    game.do_pick_up(Oid(0), loc, oid);
    time::PICK_UP
}

fn player_vs_shallow_water(game: &mut Game, _loc: &Point) -> Time {
    let mesg = Message::new(Topic::Normal, "You splash through the water.");
    game.messages.push(mesg);

    // TODO: Some NPCs should not have a penalty (or maybe even be faster)
    // TODO: May change for the player as well (especially if we have any small races)
    time::MOVE_THRU_SHALLOW_WATER // just a little slower
}

fn player_vs_sign(game: &mut Game, loc: &Point) -> Time {
    let (_, obj) = game.get(loc, SIGN_ID).unwrap();
    let mesg = Message {
        topic: Topic::Normal,
        text: format!("You see a sign {}.", obj.description()),
    };
    game.messages.push(mesg);
    Time::zero()
}
