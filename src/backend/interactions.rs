//! This is where the bulk of the logic exists to handle interactions between
//! Characters and between items. It's structured as a lookup table of
//! (tag1, tag2) => handler. For example (Player, Sign) => function_to_print_sign.
use super::sound::*;
use super::tag::*;
use super::*;
use fnv::FnvHashMap;
use rand::prelude::*;

pub enum PreResult {
    /// The player has interacted with an object at the new cell and the scheduler should
    /// run again.
    Acted(Time, Sound),

    /// The player did something that didn't take any time so he should have a chance to
    /// act again.
    ZeroAction,

    /// Another handler should be attempted.
    DidntAct,
}

// ---- struct Interaction -------------------------------------------------
pub type PreHandler = fn(&mut Game, &Point, &Point) -> PreResult;
pub type PostHandler = fn(&mut Game, &Point) -> (Time, Sound);

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

        i.pre_ins(PLAYER_ID, DOORMAN_ID, player_vs_doorman);
        i.pre_ins(PLAYER_ID, SPECTATOR_ID, player_vs_spectator);
        i.pre_ins(PLAYER_ID, CHARACTER_ID, player_vs_character);
        i.pre_ins(PLAYER_ID, TERRAIN_ID, player_vs_terrain_pre);

        i.post_ins(PLAYER_ID, PORTABLE_ID, player_vs_portable);
        i.post_ins(PLAYER_ID, SIGN_ID, player_vs_sign);
        i.post_ins(PLAYER_ID, TERRAIN_ID, player_vs_terrain_post);

        i
    }

    /// Something may want to interact with something else in a neighboring cell, e.g. tag
    ///  == PLAYER_ID and tag1 == CLOSED_DOOR_ID is used when the player attempts to open
    /// a door. PreHandler returns a duration if an interaction happened.
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
fn player_vs_terrain_pre(game: &mut Game, player_loc: &Point, new_loc: &Point) -> PreResult {
    let (oid, obj) = game.level.get_bottom(new_loc);
    let player = game.level.get(player_loc, PLAYER_ID).unwrap().1;

    // A few terrain types are special cased.
    let terrain = obj.terrain_value().unwrap();
    match terrain {
        Terrain::ClosedDoor => {
            game.do_open_door(Oid(0), player_loc, new_loc, oid);
            return PreResult::Acted(time::OPEN_DOOR, sound::VERY_QUIET);
        }
        Terrain::Vitr => {
            if game.in_inv(player, EMP_SWORD_ID) {
                let mesg = Message::new(
                    Topic::Important,
                    "You carefully place the Emperor's sword into the vitr and watch it dissolve.",
                );
                game.messages.push(mesg);

                let mesg = Message::new(Topic::Important, "You have won the game!!");
                game.messages.push(mesg);
                game.state = State::WonGame;
                return PreResult::Acted(time::DESTROY_EMP_SWORD, sound::QUIET);
            }
        }
        Terrain::Wall => {
            if game.in_inv(player, PICK_AXE_ID) {
                let material = obj.material_value();
                let delay = {
                    let item = game.inv_item(player, PICK_AXE_ID).unwrap();
                    item.delay_value().unwrap()
                };
                match material {
                    Some(Material::Stone) => {
                        let damage = 6;
                        game.do_dig(Oid(0), new_loc, oid, damage);
                        return PreResult::Acted(delay, sound::LOUD);
                    }
                    Some(Material::Metal) => {
                        let mesg = Message::new(
                            Topic::Normal,
                            "Your pick-axe bounces off the metal wall doing no damage.",
                        );
                        game.messages.push(mesg);
                        return PreResult::Acted(delay / 4, sound::QUIET);
                    }
                    None => unreachable!("Walls should always have a Material"),
                }
            }
        }
        _ => (),
    }

    // But for most we just check to see if they are impassible or not.
    if let Some(mesg) = player.impassible_terrain_type(terrain) {
        game.messages.push(mesg);
        PreResult::ZeroAction
    } else {
        PreResult::DidntAct
    }
}

fn player_vs_character(game: &mut Game, player_loc: &Point, new_loc: &Point) -> PreResult {
    let obj = game.level.get(new_loc, CHARACTER_ID).unwrap().1;
    match obj.disposition_value() {
        Some(Disposition::Aggressive) => {
            // This is QUIET because normally both parties will be making combat noises so
            // the probability is twice as high as just QUIET alone.
            let delay = game.melee_delay(player_loc);
            game.do_melee_attack(player_loc, new_loc);
            PreResult::Acted(delay, sound::QUIET)
        }
        Some(Disposition::Friendly) => {
            let mesg = Message::new(Topic::Normal, "Why would you attack a friend?");
            game.messages.push(mesg);
            PreResult::ZeroAction
        }
        Some(Disposition::Neutral) => {
            let obj = game.level.get_mut(new_loc, CHARACTER_ID).unwrap().1;
            let disposition = Tag::Disposition(Disposition::Aggressive);
            obj.replace(disposition);
            let delay = game.melee_delay(player_loc);
            game.do_melee_attack(player_loc, new_loc);
            PreResult::Acted(delay, sound::QUIET)
        }
        None => unreachable!("{obj} didn't have a Disposition!"),
    }
}

fn is_worthy(game: &Game) -> bool {
    let player = game.level.get(&game.player_loc(), PLAYER_ID).unwrap().1;
    if let Some(obj) = game.find_main_hand(player) {
        return obj.description().contains("Doom");
    }
    false
}

fn player_vs_doorman(game: &mut Game, _player_loc: &Point, doorman_loc: &Point) -> PreResult {
    if is_worthy(game) {
        let (oid, doorman) = game.level.get(doorman_loc, DOORMAN_ID).unwrap();
        if let Some(to_loc) = game.find_empty_cell(doorman, doorman_loc) {
            game.do_shove_doorman(Oid(0), doorman_loc, oid, &to_loc);
            PreResult::Acted(time::SHOVE_DOORMAN, sound::QUIET)
        } else {
            PreResult::ZeroAction
        }
    } else {
        let mesg = Message::new(Topic::NPCSpeaks, "You are not worthy.");
        game.messages.push(mesg);
        PreResult::ZeroAction
    }
}

fn player_vs_spectator(game: &mut Game, _player_loc: &Point, _new_loc: &Point) -> PreResult {
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
    PreResult::Acted(time::SPEAK_TO_SPECTATOR, sound::QUIET)
}

// ---- Post-move handlers ---------------------------------------------------------------
fn player_vs_portable(game: &mut Game, loc: &Point) -> (Time, Sound) {
    let oid = game.level.get(loc, PORTABLE_ID).unwrap().0;

    let player = game.level.get_mut(loc, CHARACTER_ID).unwrap().1;
    let inv = player.inventory_value().unwrap();
    if inv.len() < MAX_INVENTORY {
        game.do_pick_up(Oid(0), loc, oid);
        (time::PICK_UP, sound::NONE)
    } else {
        let why = "You don't have enough inventory space to ";
        game.do_ignore(Oid(0), loc, oid, why);
        (Time::zero(), sound::NONE)
    }
}

fn player_vs_sign(game: &mut Game, loc: &Point) -> (Time, Sound) {
    let (_, obj) = game.level.get(loc, SIGN_ID).unwrap();
    let mesg = Message {
        topic: Topic::Normal,
        text: format!("You see a sign {}.", obj.description()),
    };
    game.messages.push(mesg);
    (Time::zero(), sound::NONE)
}

fn player_vs_terrain_post(game: &mut Game, loc: &Point) -> (Time, Sound) {
    let (_, obj) = game.level.get(loc, TERRAIN_ID).unwrap();
    match obj.terrain_value().unwrap() {
        Terrain::Rubble => {
            let mesg = Message::new(Topic::Normal, "You pick your way through the rubble.");
            game.messages.push(mesg);
            (time::MOVE_THRU_SHALLOW_WATER * 2, sound::QUIET)
        }
        Terrain::ShallowWater => {
            let mesg = Message::new(Topic::Normal, "You splash through the water.");
            game.messages.push(mesg);

            // TODO: Some NPCs should not have a penalty (or maybe even be faster)
            // TODO: May change for the player as well (especially if we have any small races)
            (time::MOVE_THRU_SHALLOW_WATER, sound::QUIET) // just a little slower and a little louder
        }
        _ => (Time::zero(), sound::NONE),
    }
}
