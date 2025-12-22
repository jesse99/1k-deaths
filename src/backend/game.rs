use super::{Character, Oid, Scheduler, Store};
use super::{Time, time};
use crate::shared::*;
use fnv::FnvHashMap;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::cell::{RefCell, RefMut};
use std::{collections::VecDeque, fmt::Display};

const MAX_MESSAGES: usize = 10;

pub static PLAYER_ID: Oid = Oid::without_tag(0);
pub static DEFAULT_CELL_ID: Oid = Oid::without_tag(1);
pub static LAST_ID: u32 = 1;

pub struct Game {
    pub messages: VecDeque<Message>,
    pub cell_ids: FnvHashMap<Point, Oid>, // TODO rename this level_oids?
    pub store: Store<Oid>,
    pub scheduler: Scheduler,
    pub rng: RefCell<SmallRng>,
    next_oid: u32,
    players_move: bool,
}

pub fn new(seed: u64) -> Box<dyn crate::shared::Game> {
    let level = "
###########################################################
#                                                         #
#       #######                                           #
#       #a   s#                                           #
#       #     #                                           #
#       ### ###                            A              #
#                                                         #
#                                                         #
#                   @                                     #
#                                ~                        #
#                               ~_~                       #
#                                ~                        #
#                                                         #
#                                                         #
#            A                                            #
#                                      A                  #
#                                                         #
###########################################################";
    with_level(level, seed)
}

pub fn with_level(level: &str, seed: u64) -> Box<dyn crate::shared::Game> {
    let rng = RefCell::new(SmallRng::seed_from_u64(seed));
    let mut game = Box::new(Game {
        messages: VecDeque::new(),
        cell_ids: FnvHashMap::default(),
        store: Store::new(),
        next_oid: LAST_ID + 1,
        scheduler: Scheduler::new(),
        rng,
        players_move: false,
    });
    build_level(&mut game, level);
    game
}

impl crate::shared::Game for Game {
    fn player_loc(&self) -> Point {
        self.store.find(PLAYER_ID).unwrap()
    }

    fn players_turn(&self) -> bool {
        self.players_move || self.game_over()
    }

    fn player_action(&mut self, command: Command) {
        debug!("executing {command:?}");
        match command {
            Command::Move(delta) => {
                let old_loc: Point = self.store.find(PLAYER_ID).unwrap();
                let new_loc = Point::new(old_loc.x + delta.x, old_loc.y + delta.y);

                if let Some(err) = self.can_move_to(new_loc) {
                    self.add_message(MessageKind::PlayerFailed, err);
                } else {
                    let old_oid = self.cell_ids.get(&old_loc).unwrap();
                    let new_oid = self.cell_ids.get(&new_loc).unwrap();
                    self.store.transfer::<Character>(*old_oid, *new_oid);
                    // TODO characters need a Point value
                    self.store.replace(PLAYER_ID, new_loc);
                }
                if delta.x == 0 || delta.y == 0 {
                    // TODO moving into a wall should take less time? or no time?
                    self.scheduler.player_acted(time::CARDINAL_MOVE, &self.rng);
                } else {
                    self.scheduler.player_acted(time::DIAGNOL_MOVE, &self.rng);
                }
                self.players_move = false; // TODO do this only for commands that take time
            }
        }
    }

    fn other_actions(&mut self, _replay: bool) {
        match Scheduler::execute_action(self) {
            super::PlayersTurn::Yes => self.players_move = true,
            super::PlayersTurn::No => {
                // if !replay {
                //     self.stream.push(Action::Object);
                // }
                // OldPoV::update(self);
                // PoV::refresh(self);
            }
        }
    }

    // TODO maybe this should return a &Tile, that would allow us to cache this and
    // also get rid of the default method
    fn tile(&self, loc: Point) -> Option<Tile> {
        let oid = self.cell_ids.get(&loc).unwrap_or(&DEFAULT_CELL_ID);
        let terrain = self.store.find(*oid).unwrap();
        if let Some(ch) = self.store.find::<Character>(*oid) {
            let species = self.store.find(ch.oid).unwrap();
            Some(Tile {
                terrain,
                // items: Vec::new(),
                character: Some(species),
            })
        } else {
            Some(Tile {
                terrain,
                // items: Vec::new(),
                character: None,
            })
        }
    }

    fn default(&self) -> Tile {
        Tile {
            terrain: self.store.find(DEFAULT_CELL_ID).unwrap(),
            // items: Vec::new(),
            character: None,
        }
    }

    // Would be quite a bit better to return an iterator but that's very problematic:
    // `impl Iterator<Item = &Message>` isn't object safe so we can't use it with the Game trait
    // `Box<dyn Iterator<Item = &Message>>` doesn't have a good way to specify the Message lifetime
    // passing in a closure also isn't object safe
    // could use a fn but that's quite limiting
    fn messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
    }

    fn snapshot(&self, args: SnapshotArgs) -> String {
        fn species_to_char(species: Species) -> char {
            match species {
                Species::Ay => 'A',
                Species::Human => '@',
            }
        }

        fn terrain_to_char(terrain: Terrain) -> char {
            match terrain {
                Terrain::DeepWater => '_',
                Terrain::Dirt => '.',
                Terrain::RockWall => '#',
                Terrain::ShallowWater => '~',
            }
        }

        fn snapshot_map(result: &mut String, game: &Game, radius: i32) {
            for dy in -radius..radius {
                for dx in -radius..radius {
                    let player_loc = game.player_loc();
                    let loc = Point::new(player_loc.x + dx, player_loc.y + dy);
                    let ch = if loc.distance2(player_loc) <= radius * radius {
                        let oid = game.cell_ids.get(&loc).unwrap_or(&DEFAULT_CELL_ID);
                        game.store
                            .find::<Character>(*oid)
                            .map(|c| c.oid)
                            .and_then(|o| game.store.find::<Species>(o))
                            .map_or(terrain_to_char(game.get_terrain(loc)), species_to_char)
                    } else {
                        ' '
                    };
                    result.push(ch);
                }
                result.push('\n');
            }
            result.push('\n');
        }

        fn snapshot_messages(result: &mut String, game: &Game, count: usize) {
            let n = if game.messages.len() >= count {
                game.messages.len() - count
            } else {
                0
            };
            for m in game.messages.iter().skip(n) {
                result.push_str(&m.text);
                result.push('\n');
            }
            result.push('\n');
        }

        // TODO should print details for the player and nearby NPCs
        // probably would have to just print the standard store values
        let mut result = String::with_capacity(2 * 1024);
        if args.radius > 0 {
            result.push_str("map:\n");
            snapshot_map(&mut result, self, args.radius);
        }
        if args.num_messages > 0 {
            result.push_str("\nmessages:\n");
            snapshot_messages(&mut result, self, args.num_messages as usize);
        }
        result.push('\n');
        result.push_str(&self.scheduler.dump(self));
        result
    }
}

impl Game {
    fn game_over(&self) -> bool {
        // matches!(self.state, State::LostGame | State::WonGame)
        false
    }

    // TODO: If find_cell turns out to be a bottle-neck then could add a get_cell
    // function with methods like get_terrain, get_portables, and get_char.
    pub fn get_terrain(&self, loc: Point) -> Terrain {
        if let Some(&cell_oid) = self.cell_ids.get(&loc) {
            let terrain = self.store.find::<Terrain>(cell_oid);
            terrain.unwrap_or_else(|| panic!("expected a terrain for {cell_oid}"))
        } else {
            let terrain = self.store.find::<Terrain>(DEFAULT_CELL_ID);
            terrain.expect("expected a terrain for the default cell")
        }
    }

    // The RNG doesn't directly affect the game state so we use interior mutability for it.
    pub fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    pub fn find_neighbor<F>(&self, loc: &Point, predicate: F) -> Option<Point>
    where
        F: Fn(&Point) -> bool,
    {
        let mut deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        deltas.shuffle(&mut *self.rng());
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if predicate(&new_loc) {
                return Some(new_loc);
            }
        }
        None
    }

    fn can_move_to(&self, loc: Point) -> Option<String> {
        match self.get_terrain(loc) {
            Terrain::DeepWater => Some("The water is too deep.".to_owned()),
            Terrain::Dirt => None, // TODO also check for characters
            Terrain::RockWall => Some("A wall is in the way.".to_owned()),
            Terrain::ShallowWater => None, // TODO should take extra time (and include a message)
        }
    }

    fn add_message(&mut self, kind: MessageKind, text: String) {
        self.messages.push_back(Message { kind, text });

        while self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
    }

    pub fn summary_str(&self, oid: Oid) -> String {
        fn summary_player(store: &Store<Oid>, player_loc: Point, oid: Oid) -> Option<String> {
            store.find::<Point>(oid).and_then(|loc| {
                if loc == player_loc {
                    Some("player".to_string())
                } else {
                    None
                }
            })
        }

        fn summary_species(store: &Store<Oid>, oid: Oid) -> Option<String> {
            store.find::<Species>(oid).map(|s| match s {
                Species::Ay => "ay".to_string(),
                Species::Human => "human".to_string(),
            })
        }

        fn summary_terrain(store: &Store<Oid>, oid: Oid) -> Option<String> {
            match store.find(oid) {
                Some(terrain) => match terrain {
                    Terrain::DeepWater => Some("deep water".to_string()),
                    Terrain::Dirt => Some("dirt".to_string()),
                    Terrain::RockWall => Some("rock wall".to_string()),
                    Terrain::ShallowWater => Some("shallow water".to_string()),
                },
                None => None,
            }
        }

        use crate::shared::traits::Game;
        let player_loc = self.player_loc();
        let s = summary_player(&self.store, player_loc, oid)
            .or(summary_species(&self.store, oid))
            .or(summary_terrain(&self.store, oid))
            .unwrap_or("?".to_string());

        if let Some(loc) = self.store.find::<Point>(oid) {
            format!("{s} at {loc} #{}", oid.value)
        } else {
            format!("{s} #{}", oid.value)
        }
    }

    fn new_oid<T>(&mut self, obj: T) -> Oid
    where
        T: Display,
    {
        let oid = Oid::new(&format!("{obj}"), self.next_oid);
        self.next_oid += 1;
        oid
    }
}

fn build_level(game: &mut Game, level: &str) {
    game.store.create(DEFAULT_CELL_ID, Terrain::RockWall);
    let mut loc = Point::new(0, -1);
    for ch in level.chars() {
        if ch != '\n' {
            match ch {
                '\n' => (),
                '#' => (),
                '@' => {
                    let s = Species::Human;
                    let ch_oid = PLAYER_ID;
                    game.store.create(ch_oid, s);
                    game.scheduler.add(ch_oid, time::DIAGNOL_MOVE);

                    let cell_oid = game.new_oid(loc);
                    game.store.create(cell_oid, loc);
                    game.store.create(cell_oid, Terrain::Dirt);
                    game.store.create(cell_oid, Character { oid: ch_oid });
                    game.cell_ids.insert(loc, cell_oid);

                    game.store.replace(PLAYER_ID, loc);
                }
                'A' => {
                    let s = Species::Ay;
                    let ch_oid = game.new_oid(s);
                    game.store.create(ch_oid, s);
                    game.scheduler.add(ch_oid, Time::zero());

                    let cell_oid = game.new_oid(loc);
                    game.store.create(cell_oid, loc);
                    game.store.create(cell_oid, Terrain::Dirt);
                    game.store.create(cell_oid, Character { oid: ch_oid });
                    game.cell_ids.insert(loc, cell_oid);
                }
                'a' => (),
                's' => (), // TODO handle items
                ' ' => {
                    let cell_oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, cell_oid);

                    game.store.create(cell_oid, loc);
                    game.store.create(cell_oid, Terrain::Dirt);
                }
                '~' => {
                    let cell_oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, cell_oid);

                    game.store.create(cell_oid, loc);
                    game.store.create(cell_oid, Terrain::ShallowWater);
                    game.scheduler.add(cell_oid, -time::SHALLOW_FLOOD.fuzz(&game.rng));
                }
                '_' => {
                    let cell_oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, cell_oid);

                    game.store.create(cell_oid, loc);
                    game.store.create(cell_oid, Terrain::DeepWater);
                    game.scheduler.add(cell_oid, -time::DEEP_FLOOD.fuzz(&game.rng));
                }
                _ => panic!("bad char: {ch}"),
            };
            loc = Point::new(loc.x + 1, loc.y);
        } else {
            loc = Point::new(0, loc.y + 1);
        }
    }
}

// TODO: Depending on how much code coverage we have with unit tests we can just rely
// on the snapshots to catch problems rather than adding complex invariant checks.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_into_wall() {
        let level = "
#################
#               #
#@              #
#               #
#################";
        let mut game = with_level(level, 1);
        game.player_action(Command::Move(Point::new(0, 1)));
        game.player_action(Command::Move(Point::new(-1, 0)));

        let args = SnapshotArgs::new();
        insta::assert_snapshot!(game.snapshot(args));
    }
}
