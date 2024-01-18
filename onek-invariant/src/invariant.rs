#[cfg(test)]
use onek_types::*;

// TODO: probably should check the entire map
// expensive but this is in its own process and we don't have to
// process every update
// but that could blow the message queue!
#[cfg(test)]
pub fn invariant(state: &StateIO) {
    state.begin_read_transaction("invariant".to_string()); // TODO: should probably use RAII here

    let view = state.get_player_view();
    assert!(!view.cells.is_empty(), "need at least the cell the player is at");

    let loc = state.get_player_loc();
    let cell = view.cells.get(&loc);
    assert!(cell.is_some(), "expected a cell for the player at {loc}");

    let cell = cell.unwrap();
    assert!(
        cell.len() >= 2,
        "expected cell {cell:?} to have at least terrain and the player"
    );

    let object = cell.last().unwrap();
    let id = object.get("id").unwrap().to_id();
    assert!(id.0 == "player", "expected a player but found {object:?}");

    let oid = object.get("oid").unwrap().to_oid();
    assert!(*oid == PLAYER_ID, "expected player at {loc} not {oid}");

    state.end_read_transaction("invariant".to_string());
}
