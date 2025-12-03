//! Region key - spatial identifier for organisms.
//!
//! A region key wraps a Vec<usize> with efficient cloning and comparison.

use std::sync::Arc;

/// A region key that wraps a Vec<usize> with efficient cloning and comparison.
///
/// Uses an Arc to make cloning cheap and maintains a precomputed hash for O(1)
/// equality checks and ordering.
#[derive(Debug, Clone)]
pub struct RegionKey {
    /// The interval indices for each dimension
    values: Arc<Vec<usize>>,
    /// Precomputed hash for fast comparison
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

    /// Returns a reference to the underlying values.
    pub fn values(&self) -> &[usize] {
        &self.values
    }

    /// Returns the precomputed hash.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Computes the full hash of all values.
    fn compute_full_hash(values: &[usize]) -> u64 {
        values
            .iter()
            .enumerate()
            .map(|(pos, &val)| Self::position_hash(pos, val))
            .fold(0u64, |acc, h| acc ^ h)
    }

    /// Computes a position-dependent hash for a single value.
    fn position_hash(position: usize, value: usize) -> u64 {
        let combined = ((position as u128) << 64) | (value as u128);
        let low = combined as u64;
        let high = (combined >> 64) as u64;

        const FNV_PRIME: u64 = 0x100000001b3;
        let mut hash = 0xcbf29ce484222325;

        hash ^= low;
        hash = hash.wrapping_mul(FNV_PRIME);
        hash ^= high;
        hash = hash.wrapping_mul(FNV_PRIME);

        hash
    }

    /// Creates a new RegionKey with a single position updated.
    ///
    /// Uses incremental hash update for efficiency.
    pub fn with_updated_position(&self, position: usize, new_value: usize) -> Self {
        let old_value = self.values[position];
        if old_value == new_value {
            return self.clone();
        }

        let mut new_values = (*self.values).clone();
        new_values[position] = new_value;

        // Incremental hash update: XOR out old, XOR in new
        let old_pos_hash = Self::position_hash(position, old_value);
        let new_pos_hash = Self::position_hash(position, new_value);
        let new_hash = self.hash ^ old_pos_hash ^ new_pos_hash;

        Self {
            values: Arc::new(new_values),
            hash: new_hash,
        }
    }
}

impl PartialEq for RegionKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for RegionKey {}

impl std::hash::Hash for RegionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialOrd for RegionKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RegionKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_values_when_new_then_region_key_created() {
        let key = RegionKey::new(vec![1, 2, 3]);
        assert_eq!(key.values(), &[1, 2, 3]);
    }

    #[test]
    fn given_same_values_when_compared_then_equal() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = RegionKey::new(vec![1, 2, 3]);
        assert_eq!(key1, key2);
    }

    #[test]
    fn given_different_values_when_compared_then_not_equal() {
        let key1 = RegionKey::new(vec![1, 2, 3]);
        let key2 = RegionKey::new(vec![1, 2, 4]);
        assert_ne!(key1, key2);
    }

    #[test]
    fn given_key_when_with_updated_position_then_new_key_has_updated_value() {
        let key = RegionKey::new(vec![1, 2, 3]);
        let updated = key.with_updated_position(1, 5);
        assert_eq!(updated.values(), &[1, 5, 3]);
    }

    #[test]
    fn given_key_when_with_same_value_then_returns_clone() {
        let key = RegionKey::new(vec![1, 2, 3]);
        let updated = key.with_updated_position(1, 2);
        assert_eq!(key, updated);
    }

    #[test]
    fn given_key_when_clone_then_cheap_arc_clone() {
        let key = RegionKey::new(vec![1, 2, 3]);
        let cloned = key.clone();
        assert_eq!(key.hash(), cloned.hash());
        // Arc pointer should be the same
        assert!(Arc::ptr_eq(&key.values, &cloned.values));
    }
}
