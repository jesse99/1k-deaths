extern crate derive_more;
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
use serde::ser::{SerializeSeq, Serializer};

mod backend;
mod terminal;

use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;

use backend::Game;

fn main() {
    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        config,
        File::create("1k-deaths.log").unwrap(),
    )])
    .unwrap();
    let local = chrono::Local::now();
    info!(
        "started up on {} with version {}",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let (mut game, mut events) = Game::new();
    if events.is_empty() {
        game.new_game(&mut events);
    }
    game.post(events);
    let mut terminal = terminal::Terminal::new(game);
    terminal.run();
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Animal {
    Cat(String),
    Turtle,
    // Dog(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let animals = vec![Animal::Cat("meow".to_string()), Animal::Turtle];

        let bytes: Vec<u8> = rmp_serde::to_vec(&animals).unwrap();
        println!("encoded: {bytes:?}");

        let decoded: Vec<Animal> = rmp_serde::from_read_ref(&bytes).unwrap();

        assert_eq!(animals.len(), decoded.len());
        assert_eq!(animals[0], decoded[0]);
        assert_eq!(animals[1], decoded[1]);
    }

    // #[test]
    // fn test_backward_compatibility() {
    //     // Idea here is to add a Dog variant and see if we can still deserialize.
    //     // With bincode we can, but only when Dog is added to the end.
    //     let bytes: Vec<u8> = vec![
    //         146, 129, 163, 67, 97, 116, 164, 109, 101, 111, 119, 166, 84, 117, 114, 116, 108, 101,
    //     ];
    //     let decoded: Vec<Animal> = rmp_serde::from_read_ref(&bytes).unwrap();

    //     assert_eq!(2, decoded.len());
    //     assert_eq!(Animal::Cat("meow".to_string()), decoded[0]);
    //     assert_eq!(Animal::Turtle, decoded[1]);
    // }

    // https://github.com/serde-rs/json/issues/345
    #[test]
    fn test_append() {
        let mut bytes = Vec::new();
        let mut serializer = rmp_serde::encode::Serializer::new(&mut bytes);
        let mut seq = serializer.serialize_seq(None).unwrap();

        let mut animals = vec![Animal::Cat("meow".to_string()), Animal::Turtle];
        for e in animals.iter() {
            seq.serialize_element(e).unwrap();
        }

        let mut animals2 = vec![Animal::Turtle, Animal::Cat("hiss".to_string())];
        for e in animals2.iter() {
            seq.serialize_element(e).unwrap();
        }
        seq.end().unwrap();

        animals.append(&mut animals2);

        let decoded: Vec<Animal> = rmp_serde::from_read_ref(&bytes).unwrap();
        assert_eq!(animals.len(), decoded.len());
        assert_eq!(animals[0], decoded[0]);
        assert_eq!(animals[1], decoded[1]);
        assert_eq!(animals[2], decoded[2]);
        assert_eq!(animals[3], decoded[3]);
    }
}
