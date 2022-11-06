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

type Relations = FnvHashMap<RelationTag, Relation>;

pub(super) struct Store {
    tuples: FnvHashMap<ObjectId, Relations>,
    counter: u32,
}

impl Store {
    pub(super) fn new() -> Store {
        Store {
            tuples: FnvHashMap::default(),
            counter: 0,
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn new_object(&mut self, tag: &'static str) -> ObjectId {
        let new = ObjectId::Obj(Counter {
            tag,
            value: self.counter,
        });
        self.counter += 1;
        new
    }

    #[cfg(not(debug_assertions))]
    pub(super) fn new_object(&mut self, _tag: &'static str) -> ObjectId {
        let new = ObjectId::Obj(Counter { value: self.counter });
        self.counter += 1;
        new
    }
}

// The basic CRUD functions (there are also some generated functions built on top of these
// that offer better ease of use).
impl Store {
    pub(super) fn create(&mut self, oid: ObjectId, relation: Relation) {
        let tag = relation.tag();
        let relations = self.tuples.entry(oid).or_insert_with(|| Relations::default());
        let old = relations.insert(tag, relation);
        assert!(old.is_none(), "{oid} already has {tag}");
    }

    /// Typically one of the generated functions (eg expect_location or find_location)
    /// would be used instead.
    pub(super) fn find(&self, oid: ObjectId, tag: RelationTag) -> Option<&Relation> {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));

        if let Some(relations) = self.tuples.get(&oid) {
            relations.get(&tag)
        } else {
            if matches!(oid, ObjectId::Cell(_)) {
                let relations = self.tuples.get(&ObjectId::DefaultCell).unwrap();
                relations.get(&tag)
            } else {
                None
            }
        }
    }

    pub(super) fn update(&mut self, oid: ObjectId, relation: Relation) {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));

        let tag = relation.tag();
        let relations = self.tuples.entry(oid).or_insert_with(|| Relations::default());
        let old = relations.insert(tag, relation);
        assert!(old.is_some(), "{oid} is missing {tag}"); // should we return the old value?
    }

    /// Note that it is not an error to remove a missing tuple.
    pub(super) fn remove(&mut self, oid: ObjectId, tag: RelationTag) {
        debug_assert!(!matches!(oid, ObjectId::DefaultCell));

        if let Some(relations) = self.tuples.get_mut(&oid) {
            relations.remove(&tag);
        }
    }
}

// TODO: these should be generated
impl Store {
    pub(super) fn expect_location(&self, oid: ObjectId) -> &Point {
        match self.find(oid, RelationTag::Location) {
            Some(Relation::Location(value)) => value,
            _ => panic!("{oid} is missing the Location tag"),
        }
    }

    pub(super) fn find_location(&self, oid: ObjectId) -> Option<&Point> {
        match self.find(oid, RelationTag::Location) {
            Some(Relation::Location(value)) => Some(value),
            _ => None,
        }
    }
}
