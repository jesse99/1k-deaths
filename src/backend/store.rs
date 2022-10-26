//! This module is used to manage the game state. We want something general purpose but
//! still able to handle unusual cases (maybe the player can pickup a snake and throw it
//! at an NPC). So what we do is use a database of facts modeled after RDF triplets but
//! our triplets are less general than RDF (Oid's and enums instead of URIs) and quite
//! a bit more type safe/easier to use (predicates are encoded within an enum so that
//! the associated object type is fixed).
//!
//! In addition our nomenclature is a bit different:
//! object - In RDF terms this is the subject of the triplet. It's identified using an Oid.
//! relation - In RDF this is the predicate. It's an enum with values like Location.
//! value - In RDF this is the object. It's the value of the relation enum, e.g. Point.
use super::*;
use fnv::FnvHashMap;
use std::fmt;

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Used to identify a particular Relation for operations like Store::find.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum RelationTag {
    // TODO: need to generate RelationTag and Relation
    Location,
}

impl fmt::Display for RelationTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Used to associate a value with an object in the Store.
pub(super) enum Relation {
    /// The location of an object (NPC, item, terrain, etc) within a level.
    Location(Point),
}

// TODO: generate this
impl Relation {
    // Would be nicer to use the From trait but that consumes the input.
    fn tag(&self) -> RelationTag {
        match self {
            Relation::Location(_) => RelationTag::Location,
        }
    }
}

type Relations = FnvHashMap<RelationTag, Relation>;

pub(super) struct Store {
    tuples: FnvHashMap<Oid, Relations>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            tuples: FnvHashMap::default(),
        }
    }
}

// The basic CRUD functions (there are also some generated functions built on top of these
// that offer better ease of use).
impl Store {
    pub(super) fn create(&mut self, oid: Oid, relation: Relation) {
        let tag = relation.tag();
        let relations = self.tuples.entry(oid).or_insert_with(|| Relations::default());
        let old = relations.insert(tag, relation);
        assert!(old.is_none(), "Oid {oid} already has {tag}");
    }

    /// Typically one of the generated functions (eg expect_location or find_location)
    /// would be used instead.
    pub(super) fn find(&self, oid: Oid, tag: RelationTag) -> Option<&Relation> {
        let relations = self.tuples.get(&oid)?;
        relations.get(&tag)
    }

    pub(super) fn update(&mut self, oid: Oid, relation: Relation) {
        let tag = relation.tag();
        let relations = self.tuples.entry(oid).or_insert_with(|| Relations::default());
        let old = relations.insert(tag, relation);
        assert!(old.is_some(), "Oid {oid} is missing {tag}"); // should we return the old value?
    }

    /// Note that it is not an error to remove a missing tuple.
    pub(super) fn remove(&mut self, oid: Oid, tag: RelationTag) {
        if let Some(relations) = self.tuples.get_mut(&oid) {
            relations.remove(&tag);
        }
    }
}

// TODO: these should be generated
impl Store {
    pub(super) fn expect_location(&self, oid: Oid) -> &Point {
        match self.find(oid, RelationTag::Location) {
            Some(Relation::Location(value)) => value,
            _ => panic!("Oid {oid} is missing the Location tag"),
        }
    }

    pub(super) fn find_location(&self, oid: Oid) -> Option<&Point> {
        match self.find(oid, RelationTag::Location) {
            Some(Relation::Location(value)) => Some(value),
            _ => None,
        }
    }
}
