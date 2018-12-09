use fnv::FnvHasher;
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hash}
};

pub type FnvHashSet<T> = HashSet<T, BuildHasherDefault<FnvHasher>>;
pub type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

/// Construct a hash set with the specified capacity. The hashing algorithm is much faster than the default
/// on short keys such as integers and small strings.
/// On large keys it is actually slower.
/// Note it is less robust against security attacks on key collisions.
pub fn fnv_hashset<T: Hash + Eq>(capacity: usize) -> FnvHashSet<T> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashSet::<T, _>::with_capacity_and_hasher(capacity, fnv)
}

/// Construct a hash map with the specified capacity. The hashing algorithm is much faster than the default
/// on short keys such as integers and small strings.
/// On large keys it is actually slower.
/// Note it is less robust against security attacks on key collisions.
pub fn fnv_hashmap<K: Hash + Eq, V>(capacity: usize) -> FnvHashMap<K, V> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashMap::<K, V, _>::with_capacity_and_hasher(capacity, fnv)
}
