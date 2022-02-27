use derive_more::Display;

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Topic {
    /// An operation could not be completed.
    Error,

    /// An un-important message. Typically something that doesn't affect
    /// game play, e.g. bumping into a wall.
    Normal,

    /// Player tried to do something but was stopped, e.g. move into deep water.
    Failed,

    /// A significant announcement.
    Important,

    NPCSpeaks,

    /// NPC was damaged (but not by the player).
    NpcIsDamaged, // TODO: might want to have a separate Topic for player allies

    /// NPC was attacked but not damaged (but not by the player).
    NpcIsNotDamaged,

    /// The player has caused damage.
    PlayerDidDamage,

    /// The player attacked but did no damage.
    PlayerDidNoDamage,

    /// The player has taken damage.
    PlayerIsDamaged,

    /// The player was attacked but took no damage.
    PlayerIsNotDamaged,

    // /// The player will operate less well.
    // PlayerIsImpaired, // TODO: probably also want a PlayerEnchanced

    // /// The player is at risk of taking damage.
    // PlayerIsThreatened,
    /// An operation was not completely successful.
    Warning,
}

#[derive(Clone, Debug, Display, Eq, PartialEq)]
#[display(fmt = "{} {}", topic, text)]
pub struct Message {
    pub topic: Topic,
    pub text: String,
}

impl Message {
    pub fn new(topic: Topic, msg: &str) -> Message {
        Message {
            topic,
            text: String::from(msg),
        }
    }
}
