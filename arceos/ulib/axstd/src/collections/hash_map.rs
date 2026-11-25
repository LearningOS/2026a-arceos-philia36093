//! A hash map implemented with separate chaining.

use alloc::vec::Vec;
use core::fmt;
use core::hash::{Hash, Hasher};

/// The initial number of buckets of a [`HashMap`].
const INITIAL_BUCKETS: usize = 8;

/// The load factor of a [`HashMap`], i.e., `len / buckets` must stay below
/// `LOAD_FACTOR_NUM / LOAD_FACTOR_DEN`.
const LOAD_FACTOR_NUM: usize = 3;
const LOAD_FACTOR_DEN: usize = 4;

/// A no_std friendly hasher using the [FNV-1a] algorithm.
///
/// It is not cryptographically secure but is small, fast and deterministic,
/// which is enough for the kernel hash map.
///
/// [FNV-1a]: https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
#[derive(Clone, Copy, Debug)]
struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        for &byte in bytes {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }
}

/// Computes the hash value of `value` using [`FnvHasher`].
fn hash_value<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = FnvHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A hash map implemented with separate chaining.
///
/// It is a `no_std` replacement for [`std::collections::HashMap`] built on top
/// of `alloc` only. Keys are hashed by [`Hash`] and compared by [`Eq`].
///
/// # Examples
///
/// ```
/// use axstd::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("a", 1);
/// assert_eq!(map.get("a"), Some(&1));
/// ```
pub struct HashMap<K, V> {
    /// Buckets, each of which holds the entries whose hash falls into it.
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
}

impl<K, V> HashMap<K, V> {
    /// Creates an empty `HashMap`.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty `HashMap` with at least the given capacity for elements.
    pub fn with_capacity(capacity: usize) -> Self {
        let bucket_count = bucket_count_for(capacity);
        let mut buckets = Vec::with_capacity(bucket_count);
        buckets.resize_with(bucket_count, Vec::new);
        Self { buckets, len: 0 }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all elements from the map, keeping the allocated capacity.
    pub fn clear(&mut self) {
        for bucket in self.buckets.iter_mut() {
            bucket.clear();
        }
        self.len = 0;
    }

    /// Returns the number of buckets.
    fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.buckets
            .iter()
            .flat_map(|bucket| bucket.iter())
            .map(|(k, v)| (k, v))
    }

    /// An iterator visiting all key-value pairs in arbitrary order, with
    /// mutable references to the values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.buckets
            .iter_mut()
            .flat_map(|bucket| bucket.iter_mut())
            .map(|(k, v)| (&*k, v))
    }

    /// An iterator visiting all keys in arbitrary order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    /// An iterator visiting all values in arbitrary order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    /// Returns the bucket index of the given key.
    fn bucket_index(&self, key: &K) -> usize {
        hash_value(key) as usize & (self.bucket_count() - 1)
    }

    /// Resizes the bucket array to `new_bucket_count` buckets.
    fn resize(&mut self, new_bucket_count: usize) {
        let mut new_buckets = Vec::with_capacity(new_bucket_count);
        new_buckets.resize_with(new_bucket_count, Vec::new);
        for bucket in self.buckets.drain(..) {
            for (key, value) in bucket {
                let index = hash_value(&key) as usize & (new_bucket_count - 1);
                new_buckets[index].push((key, value));
            }
        }
        self.buckets = new_buckets;
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned. Otherwise
    /// the value is updated and the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if (self.len + 1) * LOAD_FACTOR_DEN > self.bucket_count() * LOAD_FACTOR_NUM {
            self.resize(self.bucket_count() * 2);
        }
        let index = self.bucket_index(&key);
        let bucket = &mut self.buckets[index];
        for (k, v) in bucket.iter_mut() {
            if *k == key {
                return Some(core::mem::replace(v, value));
            }
        }
        bucket.push((key, value));
        self.len += 1;
        None
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.bucket_index(key);
        self.buckets[index]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = self.bucket_index(key);
        self.buckets[index]
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Returns `true` if the map contains the given key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Removes a key from the map, returning the value if the key was present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.bucket_index(key);
        let bucket = &mut self.buckets[index];
        let pos = bucket.iter().position(|(k, _)| k == key)?;
        self.len -= 1;
        Some(bucket.swap_remove(pos).1)
    }
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for HashMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/// Returns the minimum bucket count that can hold `capacity` elements without
/// exceeding the load factor.
fn bucket_count_for(capacity: usize) -> usize {
    let mut count = INITIAL_BUCKETS;
    while count * LOAD_FACTOR_NUM / LOAD_FACTOR_DEN < capacity {
        count <<= 1;
    }
    count
}
