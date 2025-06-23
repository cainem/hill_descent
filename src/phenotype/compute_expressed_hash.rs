use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::Phenotype;

impl Phenotype {
    /// Computes a hash value from the spatial part of a slice of f64 values.
    /// Skips the first `num_system_parameters_to_skip` elements.
    /// Each relevant f64 is converted to its u64 bit representation and then hashed.
    pub fn compute_expressed_hash(
        expressed_values: &[f64],
        num_system_parameters_to_skip: usize,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        if expressed_values.len() > num_system_parameters_to_skip {
            for &value in &expressed_values[num_system_parameters_to_skip..] {
                value.to_bits().hash(&mut hasher);
            }
        }
        // If there are no spatial parameters (or fewer values than system params),
        // hash an empty set. hasher.finish() on an empty sequence is consistent.
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::NUM_SYSTEM_PARAMETERS;

    use super::*;
    // NUM_SYSTEM_PARAMETERS is available via crate::NUM_SYSTEM_PARAMETERS in the main code

    #[test]
    fn given_empty_slice_when_compute_expressed_hash_then_returns_consistent_hash() {
        let values: [f64; 0] = [];
        let hash1 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);
        let hash2 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);
        assert_eq!(hash1, hash2, "Hash of empty slice should be consistent");
    }

    #[test]
    fn given_slice_shorter_than_system_params_when_compute_expressed_hash_then_consistent_hash() {
        let values = [1.0, 2.0]; // Shorter than NUM_SYSTEM_PARAMETERS (typically 7)
        let hash1 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);
        let hash2 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);
        assert_eq!(
            hash1, hash2,
            "Hash of slice shorter than system params should be consistent (effectively empty spatial part)"
        );

        let expected_empty_hash = Phenotype::compute_expressed_hash(&[], NUM_SYSTEM_PARAMETERS);
        assert_eq!(
            hash1, expected_empty_hash,
            "Hash should be same as for empty spatial part"
        );
    }

    #[test]
    fn given_slice_with_only_system_params_when_compute_expressed_hash_then_consistent_hash() {
        let values = vec![1.0; NUM_SYSTEM_PARAMETERS];
        let hash1 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);
        let hash2 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);
        assert_eq!(hash1, hash2);

        let expected_empty_hash = Phenotype::compute_expressed_hash(&[], NUM_SYSTEM_PARAMETERS);
        assert_eq!(
            hash1, expected_empty_hash,
            "Hash should be same as for empty spatial part"
        );
    }

    #[test]
    fn given_system_params_and_one_spatial_when_compute_expressed_hash_then_hashes_spatial() {
        let mut values = vec![1.0; NUM_SYSTEM_PARAMETERS];
        values.push(1.23); // Spatial part: [1.23]

        // Calculate hash of just the spatial part by telling compute_expressed_hash to skip 0 system params
        let hash_spatial_part_only = Phenotype::compute_expressed_hash(&[1.23], 0);
        let hash_with_system_params =
            Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);

        assert_eq!(hash_with_system_params, hash_spatial_part_only);
    }

    #[test]
    fn given_system_params_and_multiple_spatial_when_compute_expressed_hash_then_hashes_spatial_part_only()
     {
        let mut values = vec![0.0; NUM_SYSTEM_PARAMETERS];
        values.extend_from_slice(&[1.0, 2.5, -std::f64::consts::PI]);

        let hash1 = Phenotype::compute_expressed_hash(&values, NUM_SYSTEM_PARAMETERS);

        let spatial_part = [1.0, 2.5, -std::f64::consts::PI];
        let expected_hash = Phenotype::compute_expressed_hash(&spatial_part, 0);

        assert_eq!(hash1, expected_hash);
    }

    #[test]
    fn given_different_system_params_same_spatial_when_compute_expressed_hash_then_returns_same_hash()
     {
        let mut values1 = vec![1.0; NUM_SYSTEM_PARAMETERS];
        values1.extend_from_slice(&[10.0, 20.0]);

        let mut values2 = vec![2.0; NUM_SYSTEM_PARAMETERS];
        values2.extend_from_slice(&[10.0, 20.0]);

        let hash1 = Phenotype::compute_expressed_hash(&values1, NUM_SYSTEM_PARAMETERS);
        let hash2 = Phenotype::compute_expressed_hash(&values2, NUM_SYSTEM_PARAMETERS);
        assert_eq!(
            hash1, hash2,
            "Hashes should be the same if only system parameters differ"
        );
    }

    #[test]
    fn given_same_system_params_different_spatial_when_compute_expressed_hash_then_returns_different_hashes()
     {
        let mut values1 = vec![1.0; NUM_SYSTEM_PARAMETERS];
        values1.extend_from_slice(&[10.0, 20.0]);

        let mut values2 = vec![1.0; NUM_SYSTEM_PARAMETERS];
        values2.extend_from_slice(&[30.0, 40.0]);

        let hash1 = Phenotype::compute_expressed_hash(&values1, NUM_SYSTEM_PARAMETERS);
        let hash2 = Phenotype::compute_expressed_hash(&values2, NUM_SYSTEM_PARAMETERS);
        assert_ne!(hash1, hash2, "Hashes should differ if spatial parts differ");
    }

    #[test]
    fn given_different_spatial_values_when_compute_expressed_hash_then_returns_different_hashes() {
        let values1_spatial = [1.0, 2.0];
        let values2_spatial = [2.0, 1.0];
        let values3_spatial = [1.0, 3.0];

        // Test these as if they are purely spatial data (0 system params to skip)
        let hash1 = Phenotype::compute_expressed_hash(&values1_spatial, 0);
        let hash2 = Phenotype::compute_expressed_hash(&values2_spatial, 0);
        let hash3 = Phenotype::compute_expressed_hash(&values3_spatial, 0);

        assert_ne!(
            hash1, hash2,
            "Hashes for [1.0, 2.0] and [2.0, 1.0] should differ"
        );
        assert_ne!(
            hash1, hash3,
            "Hashes for [1.0, 2.0] and [1.0, 3.0] should differ"
        );
    }

    #[test]
    fn given_spatial_values_with_zero_and_negative_zero_when_compute_expressed_hash_then_hashes_differ_if_bits_differ()
     {
        let val_positive_zero = 0.0f64;
        let val_negative_zero = -0.0f64;

        assert_ne!(val_positive_zero.to_bits(), val_negative_zero.to_bits());

        let hash_positive_zero = Phenotype::compute_expressed_hash(&[val_positive_zero], 0);
        let hash_negative_zero = Phenotype::compute_expressed_hash(&[val_negative_zero], 0);

        assert_ne!(
            hash_positive_zero, hash_negative_zero,
            "Hashes for 0.0 and -0.0 should differ as their bit patterns differ when treated as spatial data."
        );
    }

    #[test]
    fn given_spatial_values_with_nan_when_compute_expressed_hash_then_consistent_for_same_nan_bits()
    {
        let nan1 = f64::NAN;
        let nan2 = f64::NAN;

        let hash_nan1 = Phenotype::compute_expressed_hash(&[nan1], 0);
        let hash_nan2 = Phenotype::compute_expressed_hash(&[nan2], 0);

        if nan1.to_bits() == nan2.to_bits() {
            assert_eq!(
                hash_nan1, hash_nan2,
                "Hashes for NaNs with identical bit patterns should be the same (spatial data)."
            );
        } else {
            // This case is less likely for f64::NAN but possible with custom NaNs
            assert_ne!(
                hash_nan1, hash_nan2,
                "Hashes for NaNs with different bit patterns should differ (spatial data)."
            );
        }

        let values_with_nan = [1.0, f64::NAN, 3.0];
        let hash_values_nan1 = Phenotype::compute_expressed_hash(&values_with_nan, 0);
        let hash_values_nan2 = Phenotype::compute_expressed_hash(&values_with_nan, 0);
        assert_eq!(
            hash_values_nan1, hash_values_nan2,
            "Hash of slice containing NaN should be consistent (spatial data)."
        );
    }
}
