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
    let player_loc = Point::new(10, 10);
    let terrain = default_map();
    let default = Tile {
        terrain: Terrain::RockWall,
        items: Vec::new(),
        character: None,
    };
    let messages = VecDeque::new();
    Box::new(Game {
        player_loc,
        terrain,
        default,
        messages,
    })
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
                if let (Some(err)) = self.can_move_to(loc) {
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
}

impl Game {
    fn can_move_to(&self, loc: Point) -> Option<String> {
        let terrain = self.terrain.get(&loc).unwrap_or(&self.default.terrain);
        match terrain {
            Terrain::Dirt => None, // TODO also check for characters
            Terrain::RockWall => Some(format!("A wall at {loc} is in the way.")),
        }
    }

    fn add_message(&mut self, kind: MessageKind, text: String) {
        self.messages.push_back(Message { kind, text });

        while self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
    }
}

// TODO maybe this should be named build_level and take a game ref
fn default_map() -> FnvHashMap<Point, Terrain> {
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
#                                                         #
#                                                         #
#                                                         #
#                                                         #
#                                                         #
#            A                                            #
#                                      A                  #
#                                                         #
###########################################################
";
    let mut map = FnvHashMap::<Point, Terrain>::default();
    let mut loc = Point::new(0, -1);
    for ch in level.chars() {
        if ch != '\n' {
            match ch {
                '\n' => (),
                '#' => (),
                '@' => (), // TODO handle player
                'A' => (), // TODO handle chars
                'a' => (), // TODO handle items
                's' => (), // TODO handle items
                ' ' => {
                    let _ = map.insert(loc, Terrain::Dirt);
                }
                _ => panic!("bad char: {}", ch),
            };
            loc = Point::new(loc.x + 1, loc.y);
        } else {
            loc = Point::new(0, loc.y + 1);
        }
    }
    map
}
