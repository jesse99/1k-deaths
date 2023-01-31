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
                self.pov.dirty();
            }
            Some(_) if terrain == Terrain::ClosedDoor => {
                self.set_terrain(new_loc, Terrain::OpenDoor);
                self.move_char(PLAYER_ID, new_loc);
                self.pov.dirty();
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

// TODO: Depending on how much code coverage we have with unit tests we can just rely
// on the snapshots to catch problems rather than adding complex invariant checks.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let mut level = Level::from(
            "###\n\
             #@#\n\
             ###",
        );
        PoV::refresh(&mut level);
        insta::assert_display_snapshot!(level);
    }

    #[test]
    fn move_into_wall() {
        let mut level = Level::from(
            "####\n\
             #@ #\n\
             ####",
        );
        level.bump_player(1, 0);
        level.bump_player(1, 0);
        PoV::refresh(&mut level);
        insta::assert_display_snapshot!(level);
    }

    #[test]
    fn move_into_water() {
        let mut level = Level::from(
            "####\n\
             #@~W\n\
             ####",
        );
        level.bump_player(1, 0);
        level.bump_player(1, 0);
        PoV::refresh(&mut level);
        insta::assert_display_snapshot!(level);
    }

    // #[test]
    // fn move_into_door() {
    //     let mut level = Level::from(
    //         "####\n\
    //          #@+#\n\
    //          ####",
    //     );
    //     level.bump_player(1, 0);
    //     insta::assert_display_snapshot!(level);
    // }

    // #[test]
    // fn initial() {
    //     let level = Level::from(
    //         "####\n\
    //          #@+#\n\
    //          ####",
    //     );
    //     insta::assert_display_snapshot!(level);
    // }

    // #[test]
    // fn test_round_trip() {
    //     let old_level = Level::from(
    //         "###\n\
    //          #@#\n\
    //          ###",
    //     );
    //     let bytes: Vec<u8> = postcard::to_allocvec(&old_level).unwrap();
    //     let level: Store3 = from_bytes(bytes.deref()).unwrap();
    //     insta::assert_display_snapshot!(level);
    // }
}
