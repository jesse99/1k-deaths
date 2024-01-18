use super::Oid;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Used to identify an Object exemplar.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Id(pub String);

/// The value of an [`Object`] property.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Char(char),
    Id(Id),
    Int(i32),
    Oid(Oid),
    String(String),
    Seq(Vec<Value>),
}

/// Objects are used to represent most things in the game: terrain, characters, items,
/// traps, etc. They are dynamically typed to keep them as flexible as possible, e.g.
/// it's easy for an object to have a one-off property. Objects are also duck typed:
/// instead of a property like is_weapon objects have properties like melee_damage so
/// that objects can be used in multiple roles.
///
/// Objects are constructed from config files (currently ron files). It'd be possible
/// to use ron::Map directly but mapping them onto our own Object type does have
/// benefits:
/// 1) We can define Value variants for custom types like Id and Oid.
/// 2) Usage is simpler, e.g. Object keys are String instead of a Value type.
/// 3) It's easiser to add methods onto our own type. TODO: is this true?
/// 4) There should be an efficiency win because we use FnvHashMap.
/// 5) It insulates us from the config language.
pub type Object = FnvHashMap<String, Value>;

impl Value {
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            _ => panic!("{self:?} isn't a Bool"),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Value::Char(v) => *v,
            _ => panic!("{self:?} isn't an Char"),
        }
    }

    pub fn to_id(&self) -> &Id {
        match self {
            Value::Id(v) => v,
            _ => panic!("{self:?} isn't an Id"),
        }
    }

    pub fn to_oid(&self) -> &Oid {
        match self {
            Value::Oid(v) => v,
            _ => panic!("{self:?} isn't an Oid"),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Value::String(v) => &v,
            _ => panic!("{self:?} isn't a String"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{v}"),
            Value::Char(v) => write!(f, "'{v}'"),
            Value::Id(v) => write!(f, "Id({})", v.0),
            Value::Int(v) => write!(f, "{v}"),
            Value::Oid(v) => write!(f, "{v:?}"),
            Value::String(v) => write!(f, "\"{v}\""),
            Value::Seq(v) => {
                write!(f, "[")?;
                for w in v.iter() {
                    write!(f, "{w:?}, ")?;
                }
                write!(f, "]")
            }
        }
    }
}
