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
        if player_can_traverse(terrain) {
            self.update(ObjectId::Player, Relation::Location(new_loc));
        } else if terrain == Terrain::ClosedDoor {
            self.update(ObjectId::Cell(new_loc), Relation::Terrain(Terrain::OpenDoor));
            self.update(ObjectId::Player, Relation::Location(new_loc));
        } else {
            info!("can't move into {terrain}");
        }
    }
}

// TODO: should return either an option or some sort of error
fn player_can_traverse(terrain: Terrain) -> bool {
    match terrain {
        Terrain::ClosedDoor => false,
        Terrain::DeepWater => false,
        Terrain::Dirt => true,
        Terrain::OpenDoor => true,
        Terrain::Rubble => true,
        Terrain::ShallowWater => true,
        Terrain::Tree => false,
        Terrain::Vitr => false,
        Terrain::Wall => false,
    }
}
