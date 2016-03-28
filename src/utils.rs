use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash};

use fnv::FnvHasher;

/// Construct a hash set with the specified capacity. The hashing algorithm is faster than the default
/// (but is less robust against security attacks on key collisions).
pub fn fnv_hashset<T: Hash + Eq>(capacity: usize) -> HashSet<T, BuildHasherDefault<FnvHasher>> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashSet::<T, _>::with_capacity_and_hasher(capacity, fnv)
}

/// Construct a hash map with the specified capacity. The hashing algorithm is faster than the default
/// (but is less robust against security attacks on key collisions).
pub fn fnv_hashmap<K: Hash + Eq, V>(capacity: usize) -> HashMap<K, V, BuildHasherDefault<FnvHasher>> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashMap::<K, V, _>::with_capacity_and_hasher(capacity, fnv)
}
