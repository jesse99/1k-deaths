#[cfg(test)]
use super::invariant::*;
#[cfg(test)]
use super::state::*;
#[cfg(test)]
use onek_types::*;

#[cfg(test)]
trait ToSnapshot {
    fn to_snapshot(&self, state: &State) -> String;
}

#[cfg(test)]
fn terrain_to_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Dirt => ' ',
        Terrain::Wall => '#',
    }
}

#[cfg(test)]
impl ToSnapshot for StateResponse {
    fn to_snapshot(&self, state: &State) -> String {
        match self {
            StateResponse::Map(map) => map.to_snapshot(state),
            _ => panic!("snapshots are not supported for {self}"),
        }
    }
}

#[cfg(test)]
impl ToSnapshot for View {
    fn to_snapshot(&self, _test: &State) -> String {
        let mut result = String::with_capacity(200);
        for y in self.top_left.y..=self.top_left.y + self.size().height {
            for x in self.top_left.x..=self.top_left.x + self.size().width {
                let loc = Point::new(x, y);
                match self.cells.get(&loc) {
                    Some(cell) => {
                        if cell.character.unwrap_or(NULL_ID) == PLAYER_ID {
                            result.push('@');
                        } else {
                            result.push(terrain_to_char(cell.terrain));
                        }
                    }
                    None => result.push(' '),
                }
            }
            result.push('\n');
        }
        // At some point will need to use tx to include details about objects.
        result
    }
}

// Mutators
#[cfg(test)]
impl State {}

#[test]
fn test_from_str() {
    let state = State::new(
        "###\n\
             #@#\n\
             ###",
    );

    invariant(&state);

    let view = state.get_player_view();
    insta::assert_display_snapshot!(view.to_snapshot(&state));
}
