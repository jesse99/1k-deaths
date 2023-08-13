use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::fmt::Display;

mod channel_name;
mod edit_count;
mod oid;
mod point;

pub use channel_name::ChannelName;
pub use edit_count::EditCount;
pub use oid::Oid;
pub use point::Point;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Terrain {
    Dirt,
    Wall,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub terrain: Terrain,
    pub identifiers: Vec<Oid>,
    pub character: Option<Oid>,
}

/// These take the name of a channel to send a [`StateResponse`] to.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateQueries {
    // TODO: these should go into a module, maybe under a messages module
    // TODO: invaritant service will need a get all state (to ensure atomicity)
    PlayerView(ChannelName),
}

/// These update internal state and then send a StateResponse.Updated message to services
/// that used RegisterForUpdate.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateMutators {
    MovePlayer(Point),
}

/// Messages that the state service receives.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateMessages {
    Mutate(StateMutators),
    Query(StateQueries),
    RegisterForQuery(ChannelName),
    RegisterForUpdate(ChannelName),
}

/// Messages that the state service sends to other services.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateResponse {
    // TODO: ready to move should include all that are ready
    Map(HashMap<Point, Cell>),
    Updated(EditCount),
}
