/*!
# Typed and Type-Erased API

An illustration of a type, a database index, having both a typed and type-erased API.

Sometimes an implementation needs access to a concrete type while providing a type-erased interface. In some situations,
a type-erased interface can provide the same functionality as a typed-API but at some cost, and so you want to provide
both a typed and type-erased interface.

In this context, by _typed_ I mean that a concrete type is known at compile time. Generally
a typed API will be _generic_ over the type but monomorphized at compile time. (Stop
worrying about how monomorphization increases the size of the final binary. And, while
you're at it, stop obsessing about string copies.) The concrete type is `Index<T>` (notice
the generic `T`). The _typed_ API is just the methods provided by `impl<T> Index<T>{...}`.

A type-erased API, on the other hand, will be monomorphic over a trait object. We define the type-erased API via the
trait `TypeErasedIndex` (notice the lack of a generic `T`) and provide a _blanket implementation_ for all `Index<T>`s.

Notice that it turns out to be convenient to implement many of the methods of the typed
API by deferring to the type-erased API, which is typical. [You will want to do this when
you can to keep your code DRY.](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)
Don't be a slave to the DRY principle; violate it whenever it makes
sense. These rules of thumb are not an excuse to turn off your brain.

*/

use std::{any::Any, collections::HashSet, hash::Hash};

use hashbrown::{HashTable, hash_table::OccupiedEntry};

use crate::hashing::one_shot_128;

/// A "boxed" `TypeErasedIndex`, use anywhere you need a type-erased `Index<T>`
pub type BxIndex = Box<dyn TypeErasedIndex>;

pub type EntityId = u64;
pub type HashValue = u128;

/// The typed `Index<T>`
#[derive(Default)]
pub struct Index<T: Hash + Eq + Clone + Any> {
    /// We store a copy of the value here so that we can iterate over it in the typed API, and so that the type-erased
    /// API can access some serialization of it.
    lookup: HashTable<(T, HashSet<EntityId>)>,
}

/// Contains the typed API
impl<T: Hash + Eq + Clone + Any> Index<T> {
    pub fn new() -> Self {
        Self {
            lookup: HashTable::default(),
        }
    }

    /// Inserts an entity into the set associated with `key`, creating a new set if one does
    /// not yet exist. Returns a `bool` according to whether the `entity_id` already existed
    /// in the set. Observe that several of these just defer to the untyped implementation.
    pub fn insert_entity(&mut self, key: &T, entity_id: EntityId) -> bool {
        let hash = one_shot_128(&key);

        // `hasher` is called if entries need to be moved or copied to a new table.
        // This must return the same hash value that each entry was inserted with.
        let hasher = |(stored_value, _stored_set): &_| one_shot_128(stored_value) as u64;

        // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
        // death of the universe.
        let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;

        self.lookup
            .entry(hash as u64, hash128_equality, hasher)
            .or_insert_with(|| (key.clone(), HashSet::new()))
            .get_mut()
            .1
            .insert(entity_id)
    }

    /// Inserting a new _value_ requires the value itself.
    pub fn insert_value(
        &mut self,
        key: T,
        set: HashSet<EntityId>,
    ) -> OccupiedEntry<'_, (T, HashSet<EntityId>)> {
        let hash = one_shot_128(&key);
        // `hasher` is called if entries need to be moved or copied to a new table.
        // This must return the same hash value that each entry was inserted with.
        let hasher = |(stored_value, _stored_set): &_| one_shot_128(stored_value) as u64;
        self.lookup.insert_unique(hash as u64, (key, set), hasher)
    }

    /// Gets an immutable reference to the set associated with the `key` if it exists. Observe that we just defer to
    /// the untyped implementation.
    pub fn get(&self, key: &T) -> Option<&HashSet<EntityId>> {
        let hash = one_shot_128(&key);
        self.get_with_hash(hash)
    }

    /// Gets a mutable reference to the set associated with the `key` if it exists. Observe that we just defer to
    //   /// the untyped implementation.
    pub fn get_mut(&mut self, key: &T) -> Option<&mut HashSet<EntityId>> {
        let hash = one_shot_128(&key);
        self.get_with_hash_mut(hash)
    }

    // Possibly other methods ...
}

/// This trait Encapsulates the type-erased API.
pub trait TypeErasedIndex {
    /// Inserting a new entity only requires the hash but requires the set associated with the hash to already exist.
    ///
    /// If the set corresponding to the hash exists, inserts the `entity_id` into the associated set, returning a `bool`
    /// according to whether the `entity_id` was already in the set.
    /// If the set does not exist, returns `Err(())`
    fn insert_entity_with_hash(&mut self, hash: HashValue, entity_id: EntityId)
    -> Result<bool, ()>;

    /// Fetching a set only requires the hash.
    fn get_with_hash(&self, hash: HashValue) -> Option<&HashSet<EntityId>>;

    /// Fetching a set only requires the hash.
    fn get_with_hash_mut(&mut self, hash: HashValue) -> Option<&mut HashSet<EntityId>>;

    /// Does the index contain the given hash?
    fn has_hash(&self, hash: HashValue) -> bool;
}

/// A blanket implementation of the type-erased API for all `Index<T>`s.
impl<T: Hash + Eq + Clone + Any> TypeErasedIndex for Index<T> {
    fn insert_entity_with_hash(
        &mut self,
        hash: HashValue,
        entity_id: EntityId,
    ) -> Result<bool, ()> {
        // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
        // death of the universe.
        let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;

        let entities = self
            .lookup
            .find_mut(hash as u64, hash128_equality)
            .map(|(_, set)| set)
            .ok_or(())?;
        Ok(entities.insert(entity_id))
    }

    fn get_with_hash(&self, hash: HashValue) -> Option<&HashSet<EntityId>> {
        // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
        // death of the universe.
        let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;
        self.lookup
            .find(hash as u64, hash128_equality)
            .map(|(_, set)| set)
    }

    fn get_with_hash_mut(&mut self, hash: HashValue) -> Option<&mut HashSet<EntityId>> {
        // Equality is determined by comparing the full 128-bit hashes. We do not expect any collisions before the heat
        // death of the universe.
        let hash128_equality = |(stored_value, _): &_| one_shot_128(stored_value) == hash;
        self.lookup
            .find_mut(hash as u64, hash128_equality)
            .map(|(_, set)| set)
    }

    fn has_hash(&self, hash: HashValue) -> bool {
        self.get_with_hash(hash).is_some()
    }
}
