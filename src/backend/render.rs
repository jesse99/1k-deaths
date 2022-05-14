//! These are the functions that UIs use when rendering.
use super::*;

pub fn recent_messages(state: &State, limit: usize) -> impl Iterator<Item = &Message> {
    let iter = state.messages.iter();
    if limit < state.messages.len() {
        iter.skip(state.messages.len() - limit)
    } else {
        iter.skip(0)
    }
}

pub fn player_loc(state: &State) -> Point {
    player::loc(state)
}

pub fn player_hps(_state: &State) -> (i32, i32) {
    (100, 100)
}
