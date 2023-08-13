use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

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
    // TODO: invaritant service will need a get all state (to ensure atomicity, or maybe use a transaction)
    PlayerView(ChannelName),
}

/// These update internal state and then send a StateResponse.Updated message to services
/// that used RegisterForUpdate.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateMutators {
    // TODO: might need transaction support (so invariant doesn't check at a bad time)
    MovePlayer(Point), // TODO: invariant (or maybe watchdog) could catch overly long transactions
    Reset(String),     // could include an arg to map weird chars to some sort of object enum
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

mod display_impl {
    use super::*;

    impl fmt::Display for StateMessages {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl fmt::Display for StateResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}
