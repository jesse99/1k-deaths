#[cfg(test)]
use super::state::*;
#[cfg(test)]
use onek_types::PLAYER_ID;

// TODO: probably should check the entire map
// expensive but this is in its own process and we don't have to
// process every update
// but that could blow the message queue!
#[cfg(test)]
pub fn invariant(state: &State) {
    state.begin_read_transaction("invariant".to_string());

    let view = state.get_player_view();
    assert!(!view.cells.is_empty(), "need at least the cell the player is at");

    let loc = state.get_player_loc();
    let cell = view.cells.get(&loc);
    assert!(cell.is_some(), "expected a cell for the player at {loc}");

    let ch = cell.unwrap().character;
    assert!(ch.is_some(), "expected a character for the player at {loc}");

    let oid = ch.unwrap();
    assert!(oid == PLAYER_ID, "expected player at {loc} not {oid}");

    state.end_read_transaction("invariant".to_string());
}
