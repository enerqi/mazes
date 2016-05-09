use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash};

use fnv::FnvHasher;
use rand;

pub type FnvHashSet<T> = HashSet<T, BuildHasherDefault<FnvHasher>>;
pub type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

/// Construct a hash set with the specified capacity. The hashing algorithm is faster than the default
/// (but is less robust against security attacks on key collisions).
pub fn fnv_hashset<T: Hash + Eq>(capacity: usize) -> FnvHashSet<T> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashSet::<T, _>::with_capacity_and_hasher(capacity, fnv)
}

/// Construct a hash map with the specified capacity. The hashing algorithm is faster than the default
/// (but is less robust against security attacks on key collisions).
pub fn fnv_hashmap<K: Hash + Eq, V>(capacity: usize) -> FnvHashMap<K, V> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashMap::<K, V, _>::with_capacity_and_hasher(capacity, fnv)
}

/// Construct a randomly seeded XorShiftRng. This is a very fast Rng but non-cryptographically secure.
pub fn xor_shift_rng() -> rand::XorShiftRng {
    // The Rand trait used by `random` generates seeded generators aswell as values
    // so we don't need to manually call the SeedableRng::from_seed trait function.
    rand::random()
}
