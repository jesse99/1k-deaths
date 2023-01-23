//! Actions that the player can perform.
use super::*;

impl Level {
    pub fn bump_player(&mut self, dx: i32, dy: i32) {
        let old_loc = self.expect_location(PLAYER_ID);
        let new_loc = Point::new(old_loc.x + dx, old_loc.y + dy);
        let terrain = self.get_terrain(new_loc);
        match player_can_traverse(terrain) {
            None => {
                self.move_char(PLAYER_ID, new_loc);
                self.player_entered(terrain);
            }
            Some(_) if terrain == Terrain::ClosedDoor => {
                self.set_terrain(new_loc, Terrain::OpenDoor);
                self.move_char(PLAYER_ID, new_loc);
            }
            Some(mesg) => self.append_message(Message {
                kind: MessageKind::Normal,
                text: mesg.to_string(),
            }),
        }
    }

    fn player_entered(&mut self, terrain: Terrain) {
        let text = match terrain {
            Terrain::ClosedDoor => None,
            Terrain::DeepWater => None,
            Terrain::Dirt => None,
            Terrain::OpenDoor => None,
            Terrain::Rubble => Some("You pick your way through the rubble."),
            Terrain::ShallowWater => Some("You splash through the water."),
            Terrain::Tree => None,
            Terrain::Vitr => None,
            Terrain::Wall => None,
        };
        if let Some(text) = text {
            self.append_message(Message {
                kind: MessageKind::Normal,
                text: text.to_string(),
            });
        }
    }
}

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
