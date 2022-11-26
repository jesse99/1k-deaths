//! This module is used to manage the game state. We want something general purpose but
//! still able to handle unusual cases (maybe the player can pickup a snake and throw it
//! at an NPC). So what we do is use a database of facts modeled after RDF triplets but
//! our triplets are less general than RDF ([`ObjectId`]'s and [`Relation`] instead of URIs)
//! and quite a bit more type safe/easier to use (predicates are encoded within an enum so
//! that the associated object type is fixed).
//!
//! In addition our nomenclature is a bit different:
//! object - In RDF terms this is the subject of the triplet. It's identified using an [`ObjectId`].
//! relation - In RDF this is the predicate. It's an enum with values like Background(Color).
//! value - In RDF this is the object. It's the value of the [`Relation`] enum, e.g. Color.
use super::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use core::ops::Deref;
#[cfg(test)]
use postcard::from_bytes;

type Relations = FnvHashMap<RelationTag, Relation>;

#[derive(Serialize, Deserialize)]
pub struct Store {
    tuples: FnvHashMap<ObjectId, Relations>,
    counter: u32,
}

impl Store {
    pub fn new() -> Store {
        Store {
            tuples: FnvHashMap::default(),
            counter: 0,
        }
    }

    #[cfg(debug_assertions)]
    pub fn new_object(&mut self, tag: &'static str) -> ObjectId {
        let new = ObjectId::Obj(Counter {
            tag: TagStr::from_str_truncate(tag),
            value: self.counter,
        });
        self.counter += 1;
        new
    }

    #[cfg(not(debug_assertions))]
    pub fn new_object(&mut self, _tag: &'static str) -> ObjectId {
        let new = ObjectId::Obj(Counter { value: self.counter });
        self.counter += 1;
        new
    }
}

// The basic CRUD functions (there are also some generated functions built on top of these
// that offer better ease of use).
impl Store {
    pub fn create(&mut self, oid: ObjectId, relation: Relation) {
        let tag = relation.tag();
        let relations = self.tuples.entry(oid).or_insert_with(|| Relations::default());
        let old = relations.insert(tag, relation);
        assert!(old.is_none(), "{oid} already has {tag}");
    }

    /// Typically one of the generated functions (eg expect_location or find_location)
    /// would be used instead.
    pub fn find(&self, oid: ObjectId, tag: RelationTag) -> Option<&Relation> {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));

        if let Some(relations) = self.tuples.get(&oid) {
            relations.get(&tag)
        } else {
            match oid {
                // Note that we cannot get the Location for a DefaultCell (and we can't
                // use a temporary because we need to return a reference). But this should
                // be OK: the only way to access DefaultCell is to use a Cell which means
                // that the caller already has the location.
                ObjectId::Cell(_) => {
                    let relations = self.tuples.get(&ObjectId::DefaultCell).unwrap();
                    relations.get(&tag)
                }
                _ => None,
            }
        }
    }

    pub fn update(&mut self, oid: ObjectId, relation: Relation) {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));

        let tag = relation.tag();
        let relations = self.tuples.entry(oid).or_insert_with(|| Relations::default());
        let old = relations.insert(tag, relation);
        assert!(old.is_some(), "{oid} is missing {tag}"); // should we return the old value?
    }

    /// Note that it is not an error to remove a missing tuple.
    pub fn remove(&mut self, oid: ObjectId, tag: RelationTag) {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));

        if let Some(relations) = self.tuples.get_mut(&oid) {
            relations.remove(&tag);
        }
    }
}

// Non-basic core functions.
impl Store {
    pub fn process<F>(&mut self, oid: ObjectId, tag: RelationTag, mut callback: F)
    where
        F: FnMut(&mut Relation),
    {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));
        if let Some(relations) = self.tuples.get_mut(&oid) {
            let relation = relations.get_mut(&tag).unwrap();
            callback(relation);
        } else {
            panic!("Couldn't find {oid}/{tag}");
        }
    }

    pub fn process_messages<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Vec<Message>),
    {
        self.process(ObjectId::Game, RelationTag::Messages, |relation| match relation {
            Relation::Messages(messages) => callback(messages),
            _ => panic!("Expected Relation::Messages"),
        });
    }
}

// TODO: these should be generated
impl Store {
    pub fn expect_location(&self, oid: ObjectId) -> Point {
        match self.find(oid, RelationTag::Location) {
            Some(Relation::Location(value)) => *value,
            _ => panic!("{oid} is missing the Location tag"),
        }
    }

    pub fn expect_terrain(&self, oid: ObjectId) -> Terrain {
        match self.find(oid, RelationTag::Terrain) {
            Some(Relation::Terrain(value)) => *value,
            _ => panic!("{oid} is missing the Terrain tag"),
        }
    }

    pub fn find_character(&self, oid: ObjectId) -> Option<Character> {
        match self.find(oid, RelationTag::Character) {
            Some(Relation::Character(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn find_location(&self, oid: ObjectId) -> Option<Point> {
        match self.find(oid, RelationTag::Location) {
            Some(Relation::Location(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn find_terrain(&self, oid: ObjectId) -> Option<Terrain> {
        match self.find(oid, RelationTag::Terrain) {
            Some(Relation::Terrain(value)) => Some(*value),
            _ => None,
        }
    }
}

// TODO: Depending on how much code coverage we have with unit tests we can just rely
// on the snapshots to catch problems rather than adding complex invariant checks.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let store = Store::from(
            "###\n\
             # #\n\
             ###",
        );
        insta::assert_yaml_snapshot!(store);
    }

    #[test]
    fn test_round_trip() {
        let old_store = Store::from(
            "###\n\
             # #\n\
             ###",
        );
        let bytes: Vec<u8> = postcard::to_allocvec(&old_store).unwrap();
        let store: Store = from_bytes(bytes.deref()).unwrap();
        insta::assert_yaml_snapshot!(store);
    }

    #[test]
    fn move_into_wall() {
        let mut store = Store::from(
            "####\n\
             #@ #\n\
             ####",
        );
        store.bump_player(1, 0);
        store.bump_player(1, 0);
        insta::assert_yaml_snapshot!(store);
    }

    #[test]
    fn move_into_door() {
        let mut store = Store::from(
            "####\n\
             #@+#\n\
             ####",
        );
        store.bump_player(1, 0);
        insta::assert_yaml_snapshot!(store);
    }

    #[test]
    fn move_into_shallows() {
        let mut store = Store::from(
            "####\n\
             #@~#\n\
             ####",
        );
        store.bump_player(1, 0);
        insta::assert_yaml_snapshot!(store);
    }

    #[test]
    fn initial() {
        let store = Store::from(
            "####\n\
             #@+#\n\
             ####",
        );
        insta::assert_yaml_snapshot!(store);
    }
}
