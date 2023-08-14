use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

/// Name of a channel to use to communicate between services. Typically a service will
/// create a receiver and then send the channel name to other services so that they can
/// create senders.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ChannelName {
    name: String,
}

impl ChannelName {
    pub fn new(name: &str) -> ChannelName {
        ChannelName { name: name.to_string() }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ChannelName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
