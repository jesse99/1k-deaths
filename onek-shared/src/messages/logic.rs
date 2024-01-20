use crate::*;
use serde::{Deserialize, Serialize};

/// Messages that the state service receives.
#[derive(Debug, Serialize, Deserialize)]
pub enum LogicMessages {
    /// Perform a default action to a nearby cell. Typically this will be something like
    /// a move, an attack, opening a door, etc. Most often the point will be adjacent to
    /// the character and it can be further away for something like Crawl's rampage ability.
    Bump(Oid, Point),
}
