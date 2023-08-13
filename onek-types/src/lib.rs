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

pub enum Terrain {
    Dirt,
    Wall,
}
pub struct Cell {
    pub terrain: Terrain,
    pub identifiers: Vec<Oid>,
    pub character: Option<Oid>,
}

/// These take the name of a channel to send a [`StateResponse`] to.
pub enum StateQueries {
    // TODO: these should go into a module
    PlayerView(ChannelName),
}

/// these update internal state and then send a StateResponse.Updated message to services
/// that used RegisterForUpdate.
pub enum StateMutators {
    MovePlayer(Point),
}

/// Messages that the state service receives.
pub enum StateMessages {
    Mutate(StateMutators),
    Query(StateQueries),
    RegisterForQuery(ChannelName),
    RegisterForUpdate(ChannelName),
}

/// Messages that the state service sends to other services.
pub enum StateResponse {
    Map(HashMap<Point, Cell>),
    Updated(EditCount),
}
