//! Actions that the player can perform.
use super::*;
// use arraystring::{typenum::U16, ArrayString};
// use serde::{Deserialize, Serialize};
// use std::borrow::Cow;
// use std::fmt;

impl Store {
    pub fn bump_player(&mut self, dx: i32, dy: i32) {
        let old_loc = self.expect_location(ObjectId::Player);
        let new_loc = Point::new(old_loc.x + dx, old_loc.y + dy);
        let terrain = self.expect_terrain(ObjectId::Cell(new_loc));
        match player_can_traverse(terrain) {
            None => self.update(ObjectId::Player, Relation::Location(new_loc)),
            Some(_) if terrain == Terrain::ClosedDoor => {
                self.update(ObjectId::Cell(new_loc), Relation::Terrain(Terrain::OpenDoor));
                self.update(ObjectId::Player, Relation::Location(new_loc));
            }
            Some(mesg) => self.process_messages(|messages| {
                if messages.len() > 1500 {
                    messages.drain(0..1000);
                }
                messages.push(Message {
                    kind: MessageKind::Normal,
                    text: mesg.to_string(),
                });
            }),
        }
    }
}

// TODO: should return either an option or some sort of error
fn player_can_traverse(terrain: Terrain) -> Option<&'static str> {
    match terrain {
        Terrain::ClosedDoor => Some(""),
        Terrain::DeepWater => Some("The water is too deep."),
        Terrain::Dirt => None,
        Terrain::OpenDoor => None,
        Terrain::Rubble => None,
        Terrain::ShallowWater => None,
        Terrain::Tree => Some("The tree is too big."),
        Terrain::Vitr => Some("Why would you want to do that?"),
        Terrain::Wall => Some("You bump the wall."),
    }
}
