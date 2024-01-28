#![cfg(test)]
use std::collections::HashMap;

use super::*;

fn cell_to_char(cell: &Cell) -> char {
    for obj in cell.iter().rev() {
        if let Some(value) = obj.get("symbol") {
            return value.to_char();
        }
    }
    '?'
}

struct View {
    cells: HashMap<Point, Cell>,
    top_left: Point,
    bottom_right: Point,
}

impl View {
    pub fn size(&self) -> Size {
        self.bottom_right - self.top_left
    }
}

struct GameInfo {
    player_loc: Point,
    view: View,
    notes: StateResponse,
}

impl GameInfo {
    fn new(game: &Game) -> GameInfo {
        const NUM_NOTES: usize = 8;

        let player_loc = game.player_loc;
        let view = GameInfo::find_view(game);
        let notes = handle_notes(game, NUM_NOTES);
        GameInfo {
            player_loc,
            view,
            notes,
        }
    }

    fn find_view(game: &Game) -> View {
        let mut cells = HashMap::new();
        let mut top_left = Point::new(i32::MAX, i32::MAX);
        let mut bottom_right = Point::new(i32::MIN, i32::MIN);
        for &loc in game.pov.locations() {
            let cell = visible_cell(game, loc);
            cells.insert(loc, cell);

            if loc.y < top_left.y || (loc.y == top_left.y && loc.x < top_left.x) {
                top_left = loc;
            }
            if loc.y > bottom_right.y || (loc.y == bottom_right.y && loc.x > bottom_right.x) {
                bottom_right = loc;
            }
        }
        View {
            cells,
            top_left,
            bottom_right,
        }
    }
}

trait ToSnapshot {
    fn to_snapshot(&self) -> String;
}

// impl ToSnapshot for View {
//     fn to_snapshot(&self) -> String {
//         let mut result = String::with_capacity(200);
//         for y in self.top_left.y..=self.top_left.y + self.size().height {
//             for x in self.top_left.x..=self.top_left.x + self.size().width {
//                 let loc = Point::new(x, y);
//                 match self.cells.get(&loc) {
//                     Some(cell) => result.push(cell_to_char(cell)),
//                     None => result.push(' '),
//                 }
//             }
//             result.push('\n');
//         }
//         // At some point will need to use tx to include details about objects.
//         result
//     }
// }

impl ToSnapshot for View {
    fn to_snapshot(&self) -> String {
        let mut result = String::with_capacity(200);
        for y in self.top_left.y..=self.top_left.y + self.size().height {
            for x in self.top_left.x..=self.top_left.x + self.size().width {
                let loc = Point::new(x, y);
                match self.cells.get(&loc) {
                    Some(cell) => result.push(cell_to_char(cell)),
                    None => result.push(' '),
                }
            }
            result.push('\n');
        }
        // At some point will need to use tx to include details about objects.
        result
    }
}

impl ToSnapshot for Note {
    fn to_snapshot(&self) -> String {
        let mut result = String::with_capacity(200);
        result.push_str(&format!("[{:?}] {}\n", self.kind, self.text));
        result
    }
}

impl ToSnapshot for Vec<Note> {
    fn to_snapshot(&self) -> String {
        let mut result = String::with_capacity(800);

        for (i, note) in self.iter().enumerate() {
            let s = note.to_snapshot();
            result.push_str(&format!("{i}) {s}"));
        }
        result
    }
}

impl ToSnapshot for StateResponse {
    fn to_snapshot(&self) -> String {
        match self {
            // StateResponse::Map(map) => map.to_snapshot(),
            StateResponse::Notes(notes) => notes.to_snapshot(),
            _ => panic!("snapshots are not supported for {self}"),
        }
    }
}

impl ToSnapshot for GameInfo {
    fn to_snapshot(&self) -> String {
        let mut result = String::with_capacity(800);

        result.push_str(&format!("player_loc: {}\n", self.player_loc));
        result.push_str(&format!("view:\n{}\n", self.view.to_snapshot()));
        result.push_str(&format!("notes:\n{}\n", self.notes.to_snapshot()));
        result
    }
}

#[test]
fn test_from_str() {
    let mut game = Game::new();
    let mesg = StateMutators::Reset {
        reason: "test_from_str".to_owned(),
        map: "###\n\
         #@#\n\
         ###"
        .to_owned(),
    };
    handle_mutate(&mut game, mesg);

    let info = GameInfo::new(&game);
    insta::assert_display_snapshot!(info.to_snapshot());
}

#[test]
fn test_bump_move() {
    let mut game = Game::new();
    let mesg = StateMutators::Reset {
        reason: "test_bump_move".to_owned(),
        map: "####\n\
         #@ #\n\
         ####"
            .to_owned(),
    };
    handle_mutate(&mut game, mesg);

    let mesg = StateMutators::Bump(PLAYER_OID, Point::new(2, 1));
    handle_mutate(&mut game, mesg);

    let info = GameInfo::new(&game);
    insta::assert_display_snapshot!(info.to_snapshot());
}

#[test]
fn test_bump_wall() {
    let mut game = Game::new();
    let mesg = StateMutators::Reset {
        reason: "test_bump_wall".to_owned(),
        map: "####\n\
         #@ #\n\
         ####"
            .to_owned(),
    };
    handle_mutate(&mut game, mesg);

    let mesg = StateMutators::Bump(PLAYER_OID, Point::new(0, 1));
    handle_mutate(&mut game, mesg);

    let info = GameInfo::new(&game);
    insta::assert_display_snapshot!(info.to_snapshot());
}

#[test]
fn test_bump_shallow() {
    let mut game = Game::new();
    let mesg = StateMutators::Reset {
        reason: "test_bump_shallow".to_owned(),
        map: "####\n\
         #@~#\n\
         ####"
            .to_owned(),
    };
    handle_mutate(&mut game, mesg);

    let mesg = StateMutators::Bump(PLAYER_OID, Point::new(2, 1));
    handle_mutate(&mut game, mesg);

    let info = GameInfo::new(&game);
    insta::assert_display_snapshot!(info.to_snapshot());
}

#[test]
fn test_bump_deep() {
    let mut game = Game::new();
    let mesg = StateMutators::Reset {
        reason: "test_bump_deep".to_owned(),
        map: "####\n\
         #@W#\n\
         ####"
            .to_owned(),
    };
    handle_mutate(&mut game, mesg);

    let mesg = StateMutators::Bump(PLAYER_OID, Point::new(2, 1));
    handle_mutate(&mut game, mesg);

    let info = GameInfo::new(&game);
    insta::assert_display_snapshot!(info.to_snapshot());
}

// There are LOS unit tests so we don't need a lot here.
#[test]
fn test_los() {
    let mut game = Game::new();
    let mesg = StateMutators::Reset {
        reason: "test_los".to_owned(),
        map: "############\n\
         #          #\n\
         #   @   #  #\n\
         #   #      #\n\
         ############"
            .to_owned(),
    };
    handle_mutate(&mut game, mesg);

    let info = GameInfo::new(&game);
    insta::assert_display_snapshot!(info.to_snapshot());
}
