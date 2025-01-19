use arraystring::{typenum::U16, ArrayString};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

pub static NULL_OID: Oid = Oid::without_tag(0);
pub static PLAYER_OID: Oid = Oid::without_tag(1);
pub static DEFAULT_CELL_OID: Oid = Oid::without_tag(2);

pub type TagStr = ArrayString<U16>;

/// Used to uniquely identify objects.
#[derive(Clone, Copy, Eq, Serialize, Deserialize)]
pub struct Oid {
    // Used by Display so that we get more informative logging.
    #[cfg(debug_assertions)]
    pub tag: Option<TagStr>, // Option to allow us to use stuff like PLAYER_ID, annoying but it is debug only and just for Display

    pub value: u32,
}

impl Oid {
    #[cfg(debug_assertions)]
    pub fn new(tag: &str, value: u32) -> Oid {
        Oid {
            tag: Some(TagStr::from_str_truncate(tag)),
            value: value,
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(_tag: &str, value: u32) -> Oid {
        Oid { value: value }
    }

    #[cfg(debug_assertions)]
    pub const fn without_tag(value: u32) -> Oid {
        Oid {
            tag: None,
            value: value,
        }
    }

    #[cfg(not(debug_assertions))]
    pub const fn without_tag(value: u32) -> Oid {
        Oid { value: value }
    }
}

impl PartialEq for Oid {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Hash for Oid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl fmt::Display for Oid {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(tag) = self.tag {
            write!(f, "{}#{}", tag, self.value)
        } else {
            match self.value {
                0 => write!(f, "null#{}", self.value),
                1 => write!(f, "player#{}", self.value),
                2 => {
                    write!(f, "default obj#{}", self.value)
                }
                _ => panic!("excpected a tag"),
            }
        }
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.value)
    }
}

impl fmt::Debug for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
