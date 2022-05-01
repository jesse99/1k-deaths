mod message;
pub mod player;
mod primitives;
pub mod render;

pub use message::{Message, Topic};
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

/// Ecapsulates all the backend game state. All the fields and methods are private so
/// UIs must use the render and input sub-modules.
pub struct State {
    player_loc: Point,      // TODO: replace this with indexing into chars position
    messages: Vec<Message>, // messages shown to the player
}

impl State {
    pub fn new() -> State {
        let mut messages = Vec::new();
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Welcome to 1k-deaths!"),
        });
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Are you the hero who will destroy the Crippled God's sword?"),
        });
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Press the '?' key for help."),
        });

        State {
            player_loc: Point::new(20, 20),
            messages,
        }
    }
}
