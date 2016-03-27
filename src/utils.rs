use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hash};

use fnv::FnvHasher;

/// Construct a hash set with the specified capacity. The hashing algorithm is faster than the default
/// (but is less robust against security attacks on key collisions).
pub fn fnv_hashset<T: Hash + Eq>(capacity: usize) -> HashSet<T, BuildHasherDefault<FnvHasher>> {
    let fnv = BuildHasherDefault::<FnvHasher>::default();
    HashSet::<T, _>::with_capacity_and_hasher(capacity, fnv)
}
