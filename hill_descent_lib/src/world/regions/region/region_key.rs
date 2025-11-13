use std::sync::Arc;

/// A region key that wraps a Vec<usize> with efficient cloning and comparison.
///
/// This struct uses an Arc to make cloning cheap (just incrementing a reference count)
/// and maintains a precomputed hash for O(1) equality checks in the common case.
/// The hash is computed using XOR-based position-dependent mixing, which allows
/// incremental updates when a single position changes.
///
/// # Performance characteristics
///
/// - Clone: O(1) - just increments Arc reference count
/// - Equality check: O(1) in common case (hash comparison), O(n) on hash collision
/// - Hash computation: O(1) - returns precomputed value
/// - Single position update: O(n) to clone Vec, O(1) to update hash incrementally
#[derive(Debug, Clone)]
pub struct RegionKey {
    values: Arc<Vec<usize>>,
    hash: u64,
}

impl RegionKey {
    /// Creates a new RegionKey from a vector of values.
    ///
    /// Computes the full hash of the values on creation.
    pub fn new(values: Vec<usize>) -> Self {
        let hash = Self::compute_full_hash(&values);
        Self {
            values: Arc::new(values),
            hash,
        }
    }

    /// Computes the full hash of all values.
    ///
    /// This is only needed at creation time. Updates can use incremental hashing.
    fn compute_full_hash(values: &[usize]) -> u64 {
        values
            .iter()
            .enumerate()
            .map(|(pos, &val)| Self::position_hash(pos, val))
            .fold(0u64, |acc, h| acc ^ h)
    }

    /// Computes a position-dependent hash for a single value.
    ///
    /// Uses multiplication by large primes to mix the position and value together.
    /// This ensures that the same value at different positions contributes differently
    /// to the final hash, preventing issues like [1,2,3] and [3,2,1] having the same hash.
    ///
    /// # Arguments
    ///
    /// * `position` - The index in the vector
    /// * `value` - The value at that position
    ///
    /// # Returns
    ///
    /// A 64-bit hash that uniquely represents this position-value pair
    fn position_hash(position: usize, value: usize) -> u64 {
        // Combine position and value into a single u128 to ensure uniqueness
        // Then use FNV-like mixing for good distribution
        let combined = ((position as u128) << 64) | (value as u128);
        let low = combined as u64;
        let high = (combined >> 64) as u64;
        
        // FNV-1a style mixing
        const FNV_PRIME: u64 = 0x100000001b3;
        let mut hash = 0xcbf29ce484222325; // FNV offset basis
        
        hash ^= low;
        hash = hash.wrapping_mul(FNV_PRIME);
        hash ^= high;
        hash = hash.wrapping_mul(FNV_PRIME);
        
        hash
    }

    /// Creates a new RegionKey with a single position updated.
    ///
    /// This operation clones the underlying Vec but updates the hash incrementally
    /// in O(1) time by XORing out the old value's contribution and XORing in the
    /// new value's contribution.
    ///
    /// # Arguments
    ///
    /// * `position` - The index to update
    /// * `new_value` - The new value for that position
    ///
    /// # Returns
    ///
    /// A new RegionKey with the updated value and correctly recomputed hash
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds
    pub fn with_updated_position(&self, position: usize, new_value: usize) -> Self {
        let mut new_values = (*self.values).clone();
        let old_value = new_values[position];
        new_values[position] = new_value;

        // Incrementally update hash: remove old contribution, add new contribution
        // This works because XOR is its own inverse: hash ^ old ^ old = hash
        let new_hash = self.hash
            ^ Self::position_hash(position, old_value)
            ^ Self::position_hash(position, new_value);

        Self {
            values: Arc::new(new_values),
            hash: new_hash,
        }
    }

    /// Returns a reference to the underlying values.
    pub fn values(&self) -> &[usize] {
        &self.values
    }

    /// Returns the precomputed hash value.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Returns the length of the region key.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the region key is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Efficient equality based on hash first, then values.
///
/// Fast path checks hash equality (O(1)). If hashes differ, the keys are definitely
/// not equal. If hashes match, falls back to comparing actual values to handle
/// the rare case of hash collisions.
impl PartialEq for RegionKey {
    fn eq(&self, other: &Self) -> bool {
        // Fast path: different hashes mean definitely not equal
        if self.hash != other.hash {
            return false;
        }
        // Same hash: check actual values (could be collision)
        self.values == other.values
    }
}

impl Eq for RegionKey {}

/// Uses the precomputed hash for HashMap/HashSet operations.
///
/// This makes RegionKey very efficient as a HashMap key since hashing is O(1).
impl std::hash::Hash for RegionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_values_when_new_then_creates_region_key() {
        let values = vec![1, 2, 3];
        let key = RegionKey::new(values.clone());

        assert_eq!(key.values(), &values[..]);
        assert_eq!(key.len(), 3);
        assert!(!key.is_empty());
    }

    #[test]
    fn given_empty_values_when_new_then_is_empty() {
        let key = RegionKey::new(vec![]);

        assert!(key.is_empty());
        assert_eq!(key.len(), 0);
    }

    #[test]
    fn given_same_values_when_compare_then_equal() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = RegionKey::new(vec![1, 2, 3]);

        assert_eq!(key1, key2);
        assert_eq!(key1.hash(), key2.hash());
    }

    #[test]
    fn given_different_values_when_compare_then_not_equal() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = RegionKey::new(vec![1, 2, 4]);

        assert_ne!(key1, key2);
    }

    #[test]
    fn given_permuted_values_when_compare_then_not_equal() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = RegionKey::new(vec![3, 2, 1]);

        assert_ne!(key1, key2);
        assert_ne!(key1.hash(), key2.hash());
    }

    #[test]
    fn given_region_key_when_update_position_then_creates_new_key_with_correct_values() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = key1.with_updated_position(1, 5);

        assert_eq!(key1.values(), &[1, 2, 3]);
        assert_eq!(key2.values(), &[1, 5, 3]);
    }

    #[test]
    fn given_region_key_when_update_position_then_hash_matches_fresh_computation() {
        let key1 = RegionKey::new(vec![10, 20, 30]);
        let key2 = key1.with_updated_position(1, 99);

        // Create a fresh key with the same values to verify hash is correct
        let key3 = RegionKey::new(vec![10, 99, 30]);

        assert_eq!(key2.values(), key3.values());
        assert_eq!(key2.hash(), key3.hash());
        assert_eq!(key2, key3);
    }

    #[test]
    fn given_region_key_when_update_multiple_positions_then_hash_correct() {
        let key1 = RegionKey::new(vec![1, 2, 3, 4, 5]);
        let key2 = key1.with_updated_position(0, 10);
        let key3 = key2.with_updated_position(4, 50);

        let expected = RegionKey::new(vec![10, 2, 3, 4, 50]);

        assert_eq!(key3, expected);
        assert_eq!(key3.hash(), expected.hash());
    }

    #[test]
    fn given_region_key_when_clone_then_shares_underlying_data() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = key1.clone();

        assert_eq!(key1, key2);
        // Verify they share the same Arc (same pointer)
        assert!(Arc::ptr_eq(&key1.values, &key2.values));
    }

    #[test]
    fn given_region_keys_when_used_as_hashmap_keys_then_works_correctly() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = RegionKey::new(vec![4, 5, 6]);
        let key3 = RegionKey::new(vec![1, 2, 3]);

        map.insert(key1.clone(), "first");
        map.insert(key2.clone(), "second");

        assert_eq!(map.get(&key1), Some(&"first"));
        assert_eq!(map.get(&key2), Some(&"second"));
        assert_eq!(map.get(&key3), Some(&"first")); // key3 equals key1
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn given_different_positions_same_value_when_hash_then_different() {
        // Ensure position matters in the hash
        let key1 = RegionKey::new(vec![5, 0, 0]);
        let key2 = RegionKey::new(vec![0, 5, 0]);
        let key3 = RegionKey::new(vec![0, 0, 5]);

        assert_ne!(key1.hash(), key2.hash());
        assert_ne!(key2.hash(), key3.hash());
        assert_ne!(key1.hash(), key3.hash());
    }

    #[test]
    #[should_panic]
    fn given_out_of_bounds_position_when_update_then_panics() {
        let key = RegionKey::new(vec![1, 2, 3]);
        let _ = key.with_updated_position(10, 99);
    }

    #[test]
    fn given_hash_collision_when_compare_then_values_checked() {
        // Create a mock RegionKey with same hash but different values
        // This tests the fallback path in PartialEq where hashes match but values differ
        let key1 = RegionKey::new(vec![1, 2, 3]);

        // Manually construct a key with the same hash but different values
        let key2 = RegionKey {
            values: Arc::new(vec![9, 9, 9]),
            hash: key1.hash, // Force same hash
        };

        // They should not be equal despite having the same hash
        assert_ne!(key1, key2);
        assert_eq!(key1.hash, key2.hash); // Verify hash collision scenario
    }
}
