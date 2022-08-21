//! Wrapper for `lru::LruCache`
//!
//! This is needed because `lru::LruCache` doesn't implement (de)serialization.
use std::cmp::Eq;
use std::fmt::{Formatter, Result as FmtResult};
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use lru::{DefaultHasher, LruCache as OrigLruCache};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Wrapper to enable (de)serialization
pub(super) struct LRUCache<K, V, S = DefaultHasher>(OrigLruCache<K, V, S>);

impl<K: Hash + Eq, V> LRUCache<K, V> {
    pub(super) fn new(capacity: usize) -> Self {
        LRUCache(OrigLruCache::new(capacity))
    }

    pub(super) fn unbounded() -> Self {
        LRUCache(OrigLruCache::unbounded())
    }
}

/// Avoid usage of self.0 with self
///
/// Makes life easier when using the wrapper struct.
impl<K, V, S> Deref for LRUCache<K, V, S> {
    type Target = OrigLruCache<K, V, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K, V, S> DerefMut for LRUCache<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Deserialization code heavily "inspired" by: https://serde.rs/deserialize-map.html

/// Helper struct to deserialize a map into an LRU cache
struct LRUVisitor<K, V> {
    // Use phantoms to "use" the generic params without actually using them.
    // They're needed to correspond to an LRUCache<K, V>.
    phantom_key: PhantomData<K>,
    phantom_value: PhantomData<V>,
}

impl<K, V> LRUVisitor<K, V> {
    fn new() -> Self {
        LRUVisitor {
            phantom_key: PhantomData,
            phantom_value: PhantomData,
        }
    }
}

/// Allow the LRU visitor to talk to the deserializer and deserialize a map into an LRU cache
impl<'de, K, V> Visitor<'de> for LRUVisitor<K, V>
where
    K: Deserialize<'de> + Hash + Eq,
    V: Deserialize<'de>,
{
    type Value = LRUCache<K, V>;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a map of String keys and String values")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        // If the size is unknown, use an unbounded LRU to be on the safe side
        let mut lru = match access.size_hint() {
            Some(size) => LRUCache::new(size),
            None => LRUCache::unbounded(),
        };

        // Add all map entries one-by-one
        while let Some((key, value)) = access.next_entry()? {
            lru.push(key, value);
        }
        Ok(lru)
    }
}

/// Make the LRU cache deserializable as a map
impl<'de, K, V> Deserialize<'de> for LRUCache<K, V>
where
    K: Deserialize<'de> + Hash + Eq,
    V: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(LRUVisitor::new())
    }
}

// Serialization code heavily "inspired" by:
// https://serde.rs/impl-serialize.html#serializing-a-sequence-or-map

/// Make the LRU cache serializable as a map
impl<K, V, H> Serialize for LRUCache<K, V, H>
where
    K: Serialize + Hash + Eq,
    V: Serialize,
    H: BuildHasher,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        // Serialize all LRU entries one-by-one
        for (k, v) in self.into_iter() {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}
