//! Actions that the player can perform.
use super::*;

impl Game {
    pub fn bump_player(&mut self, dx: i32, dy: i32) {
        let old_loc = self.level.expect_location(PLAYER_ID);
        let new_loc = Point::new(old_loc.x + dx, old_loc.y + dy);
        let terrain = self.level.get_terrain(new_loc);
        match player_can_traverse(terrain) {
            None => {
                self.do_move(PLAYER_ID, old_loc, new_loc);
            }
            Some(_) if terrain == Terrain::ClosedDoor => {
                self.do_open_door(PLAYER_ID, old_loc, new_loc);
            }
            Some(mesg) => self.add_message(Message {
                kind: MessageKind::Normal,
                text: mesg.to_string(),
            }),
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
        let mut game = Game::test_game(
            "###\n\
             #@#\n\
             ###",
        );
        PoV::refresh(&mut game.level);
        insta::assert_display_snapshot!(game);
    }

    #[test]
    fn move_into_wall() {
        let mut game = Game::test_game(
            "####\n\
             #@ #\n\
             ####",
        );
        game.bump_player(1, 0);
        game.bump_player(1, 0);
        PoV::refresh(&mut game.level);
        insta::assert_display_snapshot!(game);
    }

    #[test]
    fn move_into_water() {
        let mut game = Game::test_game(
            "####\n\
             #@~W\n\
             ####",
        );
        game.bump_player(1, 0);
        game.bump_player(1, 0);
        PoV::refresh(&mut game.level);
        insta::assert_display_snapshot!(game);
    }

    // #[test]
    // fn move_into_door() {
    //     let mut game = Game::test_game(
    //         "####\n\
    //          #@+#\n\
    //          ####",
    //     );
    // PoV::refresh(&mut game.level);
    // insta::assert_display_snapshot!(game);
    // }

    // #[test]
    // fn initial() {
    //     let game = Game::test_game(
    //         "####\n\
    //          #@+#\n\
    //          ####",
    //     );
    //     insta::assert_display_snapshot!(game);
    // }

    // #[test]
    // fn test_round_trip() {
    //     let old_game = Game::test_game(
    //         "###\n\
    //          #@#\n\
    //          ###",
    //     );
    //     let bytes: Vec<u8> = postcard::to_allocvec(&game).unwrap();
    //     let game: Game = from_bytes(bytes.deref()).unwrap();
    //     insta::assert_display_snapshot!(game);
    // }
}
