use xxhash_rust::xxh3::xxh3_64;

/// Derives deterministic seed for region from world seed + region key.
/// Same world seed + region key = same RNG, different regions = independent streams.
pub fn derive_region_seed(world_seed: u64, region_key: &[usize]) -> u64 {
    let mut hasher_input = world_seed.to_le_bytes().to_vec();
    for &idx in region_key {
        hasher_input.extend_from_slice(&(idx as u64).to_le_bytes());
    }
    xxh3_64(&hasher_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_same_world_seed_and_key_when_derive_then_same_result() {
        assert_eq!(
            derive_region_seed(12345, &[0, 1, 2]),
            derive_region_seed(12345, &[0, 1, 2])
        );
    }

    #[test]
    fn given_different_world_seeds_when_derive_then_different_results() {
        assert_ne!(
            derive_region_seed(12345, &[0, 1, 2]),
            derive_region_seed(67890, &[0, 1, 2])
        );
    }

    #[test]
    fn given_different_region_keys_when_derive_then_different_results() {
        assert_ne!(
            derive_region_seed(12345, &[0, 1, 2]),
            derive_region_seed(12345, &[0, 1, 3])
        );
    }

    #[test]
    fn given_empty_region_key_when_derive_then_returns_valid_seed() {
        assert_ne!(derive_region_seed(12345, &[]), 0);
    }

    #[test]
    fn given_large_region_key_when_derive_then_returns_valid_seed() {
        let large_key: Vec<usize> = (0..100).collect();
        assert_ne!(derive_region_seed(12345, &large_key), 0);
    }
}
