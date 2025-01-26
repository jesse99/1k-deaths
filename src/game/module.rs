use crate::shared::*;
use fnv::FnvHashMap;
use std::collections::VecDeque;

const MAX_MESSAGES: usize = 10;

// It'd be more efficient to use some sort of 2D array for terrain but we use a hash map
// so that it is more dynamic, e.g. this way it's easy to support things like the player
// extending the map with something like tunneling.
struct Game {
    player_loc: Point,
    terrain: FnvHashMap<Point, Terrain>,
    default: Tile,
    messages: VecDeque<Message>,
}

pub fn new() -> Box<dyn crate::shared::Game> {
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
    with_level(level)
}

pub fn with_level(level: &str) -> Box<dyn crate::shared::Game> {
    let default = Tile {
        terrain: Terrain::RockWall,
        items: Vec::new(),
        character: None,
    };
    let mut game = Box::new(Game {
        player_loc: Point::new(0, 0),
        terrain: FnvHashMap::default(),
        default,
        messages: VecDeque::new(),
    });
    build_level(&mut game, level);
    game
}

impl crate::shared::Game for Game {
    fn player_loc(&self) -> Point {
        self.player_loc
    }

    fn execute(&mut self, command: Command) {
        debug!("executing {command:?}");
        match command {
            Command::Move(delta) => {
                let loc = Point::new(self.player_loc.x + delta.x, self.player_loc.y + delta.y);
                if let Some(err) = self.can_move_to(loc) {
                    self.add_message(MessageKind::PlayerFailed, err);
                } else {
                    self.player_loc = loc;
                }
            }
        }
    }

    // TODO maybe this should return a &Tile, that would allow us to cache this and
    // also get rid of the default method
    fn tile(&self, loc: Point) -> Option<Tile> {
        if let Some(&terrain) = self.terrain.get(&loc) {
            if loc == self.player_loc {
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
        } else {
            if loc == self.player_loc {
                Some(Tile {
                    terrain: Terrain::RockWall,
                    items: Vec::new(),
                    character: Some(Species::Human),
                })
            } else {
                None
            }
        }
    }

    fn default(&self) -> &Tile {
        &self.default
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
            let default = game.default.terrain;
            for dy in -radius..radius {
                for dx in -radius..radius {
                    let loc = Point::new(game.player_loc.x + dx, game.player_loc.y + dy);
                    let ch = if loc.distance2(game.player_loc) <= radius * radius {
                        if loc == game.player_loc {
                            '@'
                        } else if let Some(terrain) = game.terrain.get(&loc) {
                            terrain_to_char(*terrain)
                        } else {
                            terrain_to_char(default)
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
    fn can_move_to(&self, loc: Point) -> Option<String> {
        let terrain = self.terrain.get(&loc).unwrap_or(&self.default.terrain);
        match terrain {
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
}

fn build_level(game: &mut Game, level: &str) {
    let mut loc = Point::new(0, -1);
    for ch in level.chars() {
        if ch != '\n' {
            match ch {
                '\n' => (),
                '#' => (),
                '@' => {
                    let _ = game.terrain.insert(loc, Terrain::Dirt);
                    game.player_loc = loc;
                }
                'A' => (), // TODO handle chars
                'a' => (), // TODO handle items
                's' => (), // TODO handle items
                ' ' => {
                    let _ = game.terrain.insert(loc, Terrain::Dirt);
                }
                '~' => {
                    let _ = game.terrain.insert(loc, Terrain::ShallowWater);
                }
                '_' => {
                    let _ = game.terrain.insert(loc, Terrain::DeepWater);
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
        let mut game = with_level(level);
        game.execute(Command::Move(Point::new(0, 1)));
        game.execute(Command::Move(Point::new(-1, 0)));

        let args = SnapshotArgs::new();
        insta::assert_snapshot!(game.snapshot(args));
    }
}
