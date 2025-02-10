use super::{ActiveTime, Oid, Scheduler, Store};
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
    pub cell_ids: FnvHashMap<Point, Oid>,
    pub store: Store<Oid>,
    pub scheduler: Scheduler,
    pub rng: RefCell<SmallRng>,
    next_oid: u32,
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
    });
    build_level(&mut game, level);
    game
}

impl crate::shared::Game for Game {
    fn player_loc(&self) -> Point {
        self.store.find(PLAYER_ID).unwrap()
    }

    fn execute(&mut self, command: Command) {
        debug!("executing {command:?}");
        match command {
            Command::Move(delta) => {
                let old_loc = self.player_loc();
                let new_loc = Point::new(old_loc.x + delta.x, old_loc.y + delta.y);
                if let Some(err) = self.can_move_to(new_loc) {
                    self.add_message(MessageKind::PlayerFailed, err);
                } else {
                    self.store.replace(PLAYER_ID, new_loc);
                }
            }
        }
    }

    // TODO maybe this should return a &Tile, that would allow us to cache this and
    // also get rid of the default method
    fn tile(&self, loc: Point) -> Option<Tile> {
        let player_loc = self.player_loc();
        let oid = self.cell_ids.get(&loc).unwrap_or(&DEFAULT_CELL_ID);
        let terrain = self.store.find(*oid).unwrap();
        if loc == player_loc {
            Some(Tile {
                terrain,
                items: Vec::new(),
                character: Some(Species::Human),
            })
        } else {
            Some(Tile {
                terrain,
                items: Vec::new(),
                character: None,
            })
        }
    }

    fn default(&self) -> Tile {
        Tile {
            terrain: self.store.find(DEFAULT_CELL_ID).unwrap(),
            items: Vec::new(),
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

    fn snapshot(&self, args: SnapshotArgs) -> String {
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
                        if loc == player_loc {
                            '@'
                        } else {
                            terrain_to_char(game.get_terrain(loc))
                        }
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

        let mut result = String::with_capacity(1024);
        if args.radius > 0 {
            snapshot_map(&mut result, self, args.radius);
        }
        if args.num_messages > 0 {
            snapshot_messages(&mut result, self, args.num_messages as usize);
        }
        result
    }
}

impl Game {
    // TODO: If find_cell turns out to be a bottle-neck then could add a get_cell
    // function with methods like get_terrain, get_portables, and get_char.
    pub fn get_terrain(&self, loc: Point) -> Terrain {
        if let Some(&cell_oid) = self.cell_ids.get(&loc) {
            let terrain = self.store.find::<Terrain>(cell_oid);
            terrain.expect(&format!("expected a terrain for {cell_oid}"))
        } else {
            let terrain = self.store.find::<Terrain>(DEFAULT_CELL_ID);
            terrain.expect(&format!("expected a terrain for the default cell"))
        }
    }

    // The RNG doesn't directly affect the game state so we use interior mutability for it.
    pub fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
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
                    let oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, oid);
                    game.store.create(oid, Terrain::Dirt);
                    game.store.create(PLAYER_ID, loc);
                    game.store.create(PLAYER_ID, ActiveTime {});
                }
                'A' => (), // TODO handle chars
                'a' => (), // TODO handle items
                's' => (), // TODO handle items
                ' ' => {
                    let oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, oid);
                    game.store.create(oid, Terrain::Dirt);
                }
                '~' => {
                    let oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, oid);
                    game.store.create(oid, Terrain::ShallowWater);
                }
                '_' => {
                    let oid = game.new_oid(loc);
                    game.cell_ids.insert(loc, oid);
                    game.store.create(oid, Terrain::DeepWater);
                }
                _ => panic!("bad char: {}", ch),
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
        game.execute(Command::Move(Point::new(0, 1)));
        game.execute(Command::Move(Point::new(-1, 0)));

        let args = SnapshotArgs::new();
        insta::assert_snapshot!(game.snapshot(args));
    }
}
