//! Used to manage game state. The idea here is to use a very general value store to
//! record information about the world. It relies on a KEY type (something hashable)
//! to access arbitrary VALUE types. Note that the value types for a given key must
//! be unique which may require the use of structs to bundle together information or
//! the newtype idiom.
use super::TypeId;
use fnv::FnvHashMap;
use postcard::from_bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;

type Values = FnvHashMap<u16, Vec<u8>>; // u16 is the TypeId for a particular value type
type ListValue = FnvHashMap<u16, Vec<Vec<u8>>>; // like `Values` except that there is a list of values

/// Records all the game state.
#[derive(Serialize, Deserialize)]
pub struct Store<KEY>
where
    KEY: Hash + Eq + Display + Copy,
{
    primitives: FnvHashMap<KEY, Values>,
    lists: FnvHashMap<KEY, ListValue>, // may want to use a VecDeque<Vec<u8>> here

    #[cfg(debug_assertions)]
    ids: FnvHashMap<String, u16>, // used to verify that ids are unique
}

impl<KEY> Store<KEY>
where
    KEY: Hash + Eq + Display + Copy,
{
    #[must_use]
    pub fn new() -> Store<KEY> {
        Store {
            primitives: FnvHashMap::default(),
            lists: FnvHashMap::default(),

            #[cfg(debug_assertions)]
            ids: FnvHashMap::default(),
        }
    }
}

// Primitive values
impl<KEY> Store<KEY>
where
    KEY: Hash + Eq + Display + Copy,
{
    /// It's an error if there is already a type with VALUE for the key.
    pub fn create<VALUE>(&mut self, key: KEY, value: VALUE)
    where
        VALUE: Serialize + TypeId<VALUE> + Display,
    {
        debug!("creating {key} -> {value}");
        let had_old = self.replace(key, value);
        assert!(!had_old);
    }

    /// OK if the value's type isn't present. Returns true on replace.
    pub fn replace<VALUE>(&mut self, key: KEY, value: VALUE) -> bool
    where
        VALUE: Serialize + TypeId<VALUE> + Display,
    {
        let bytes: Vec<u8> = postcard::to_allocvec(&value).unwrap();
        let values = self.primitives.entry(key).or_default();
        let old = values.insert(VALUE::ID, bytes);
        debug!("replace {key} with {value}");
        assert!(self.good_id::<VALUE>());
        old.is_some()
    }

    /// Note that it is not an error to remove a missing value.
    #[allow(dead_code)] // TODO: remove this
    pub fn remove<VALUE>(&mut self, key: KEY)
    where
        VALUE: DeserializeOwned + TypeId<VALUE> + Display,
    {
        if let Some(values) = self.primitives.get_mut(&key) {
            if values.remove(&VALUE::ID).is_some() {
                debug!("remove {key}");
            }
        }
    }

    pub fn transfer<VALUE>(&mut self, from: KEY, to: KEY)
    where
        VALUE: DeserializeOwned + TypeId<VALUE> + Display,
    {
        let old_values = self.primitives.get_mut(&from).unwrap();
        let bytes = old_values.remove(&VALUE::ID).unwrap();

        let new_values = self.primitives.entry(to).or_default();
        let old = new_values.insert(VALUE::ID, bytes);
        assert!(old.is_none());
        debug!("move from {from} to {to}");
    }

    #[must_use]
    pub fn find<VALUE>(&self, key: KEY) -> Option<VALUE>
    where
        VALUE: DeserializeOwned + TypeId<VALUE> + Display,
    {
        self.primitives.get(&key).and_then(|values| {
            values.get(&VALUE::ID).map(|bytes| {
                let value: VALUE = from_bytes(bytes).unwrap();
                value
            })
        })
    }
}

// List values
impl<KEY> Store<KEY>
where
    KEY: Hash + Eq + Display + Copy,
{
    // /// Used for lists of VALUEs.
    // #[must_use]
    // pub fn len<VALUE>(&self, key: KEY) -> usize
    // where
    //     VALUE: Serialize + TypeId<VALUE> + Display,
    // {
    //     self.lists
    //         .get(&key)
    //         .map_or(0, |lists| lists.get(&VALUE::ID).map_or(0, |list| list.len()))
    // }

    // /// Used for lists of VALUEs.
    // #[must_use]
    // pub fn get_all<VALUE>(&self, key: KEY) -> Vec<VALUE>
    // where
    //     VALUE: DeserializeOwned + Serialize + TypeId<VALUE> + Display,
    // {
    //     self.lists.get(&key).map_or(vec![], |lists| {
    //         lists.get(&VALUE::ID).map_or(vec![], |list| {
    //             list.iter().map(|bytes| from_bytes(bytes).unwrap()).collect()
    //         })
    //     })
    // }

    // /// Used for lists of VALUEs.
    // #[must_use]
    // pub fn get_last<VALUE>(&self, key: KEY) -> Option<VALUE>
    // where
    //     VALUE: DeserializeOwned + Serialize + TypeId<VALUE> + Display,
    // {
    //     self.lists.get(&key).and_then(|lists| {
    //         lists
    //             .get(&VALUE::ID)
    //             .and_then(|list| list.last().map(|bytes| from_bytes(bytes).unwrap()))
    //     })
    // }

    // /// Used for lists of VALUEs.
    // #[must_use]
    // pub fn get_range<VALUE>(&self, key: KEY, range: Range<usize>) -> Vec<VALUE>
    // where
    //     VALUE: DeserializeOwned + Serialize + TypeId<VALUE> + Display,
    // {
    //     self.lists.get(&key).map_or(vec![], |lists| {
    //         lists.get(&VALUE::ID).map_or(vec![], |list| {
    //             list[range].iter().map(|bytes| from_bytes(bytes).unwrap()).collect()
    //         })
    //     })
    // }

    // /// Used for lists of VALUEs.
    // pub fn append<VALUE>(&mut self, key: KEY, value: VALUE)
    // where
    //     VALUE: Serialize + TypeId<VALUE> + Display,
    // {
    //     let lists = self.lists.entry(key).or_default();
    //     let list = lists.entry(VALUE::ID).or_default();

    //     let bytes: Vec<u8> = postcard::to_allocvec(&value).unwrap();
    //     list.push(bytes);
    //     assert!(self.good_id::<VALUE>());
    // }

    // /// Used for lists of VALUEs. Removes a value using an equality test.
    // pub fn remove_value<VALUE>(&mut self, key: KEY, value: VALUE)
    // where
    //     VALUE: DeserializeOwned + Serialize + TypeId<VALUE> + Display + Default + PartialEq,
    // {
    //     if let Some(lists) = self.lists.get_mut(&key) {
    //         if let Some(list) = lists.get_mut(&VALUE::ID) {
    //             if let Some(index) = list.iter().position(|bytes| {
    //                 let x: VALUE = from_bytes(bytes).unwrap();
    //                 x == value
    //             }) {
    //                 list.remove(index);
    //             }
    //         }
    //     }
    // }

    // /// Used for lists of VALUEs.
    // pub fn remove_range<VALUE>(&mut self, key: KEY, range: Range<usize>)
    // where
    //     VALUE: DeserializeOwned + Serialize + TypeId<VALUE> + Display + Default,
    // {
    //     if let Some(lists) = self.lists.get_mut(&key) {
    //         if let Some(list) = lists.get_mut(&VALUE::ID) {
    //             list.drain(range);
    //         }
    //     }
    // }
}

// Debug support
impl<KEY> Store<KEY>
where
    KEY: Hash + Eq + Display + Copy,
{
    #[cfg(debug_assertions)]
    fn good_id<VALUE>(&mut self) -> bool
    where
        VALUE: TypeId<VALUE>,
    {
        self.ids
            .insert(std::any::type_name::<VALUE>().to_string(), VALUE::ID)
            .is_none_or(|old_id| old_id == VALUE::ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{self};

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
    enum Key {
        Home,
        Work,
        History,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct Address {
        pub street: String,
    }

    impl Address {
        fn new(street: &str) -> Address {
            Address {
                street: street.to_string(),
            }
        }
    }

    impl<T> TypeId<T> for Address {
        const ID: u16 = 1000;
    }

    impl Display for Key {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl Display for Address {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{self:?}")
        }
    }

    #[test]
    fn test_find() {
        let mut store = Store::new();

        let value = store.find::<Address>(Key::Home);
        assert!(value.is_none());

        store.create(Key::Home, Address::new("park ave"));
        let value = store.find::<Address>(Key::Home);
        assert_eq!(value.unwrap().street, "park ave");

        let value = store.find::<Address>(Key::Work);
        assert!(value.is_none());

        store.remove::<Address>(Key::Home);
        let value = store.find::<Address>(Key::Home);
        assert!(value.is_none());
    }

    // #[test]
    // fn test_list() {
    //     let mut store = Store::new();

    //     let len = store.len::<Address>(Key::History);
    //     assert_eq!(len, 0);

    //     store.append(Key::History, Address::new("park ave"));
    //     store.append(Key::History, Address::new("main street"));
    //     store.append(Key::History, Address::new("downtown"));

    //     let len = store.len::<Address>(Key::History);
    //     assert_eq!(len, 3);

    //     let slice = store.get_range::<Address>(Key::History, 0..1);
    //     assert_eq!(slice.len(), 1);
    //     assert_eq!(slice[0].street, "park ave");

    //     let slice = store.get_all::<Address>(Key::History);
    //     assert_eq!(slice.len(), 3);
    //     assert_eq!(slice[0].street, "park ave");
    //     assert_eq!(slice[1].street, "main street");
    //     assert_eq!(slice[2].street, "downtown");

    //     store.remove_range::<Address>(Key::History, 0..2);
    //     let slice = store.get_all::<Address>(Key::History);
    //     assert_eq!(slice.len(), 1);
    //     assert_eq!(slice[0].street, "downtown");
    // }
}
