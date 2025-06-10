use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::Phenotype;

impl Phenotype {
    /// Computes a hash value from a slice of f64 values.
    /// Each f64 is converted to its u64 bit representation and then hashed.
    pub fn compute_expressed_hash(expressed_values: &[f64]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for &value in expressed_values {
            value.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_slice_when_compute_expressed_hash_then_returns_consistent_hash() {
        let values: [f64; 0] = [];
        let hash1 = Phenotype::compute_expressed_hash(&values);
        let hash2 = Phenotype::compute_expressed_hash(&values);
        assert_eq!(hash1, hash2, "Hash of empty slice should be consistent");
    }

    #[test]
    fn given_single_value_when_compute_expressed_hash_then_returns_consistent_hash() {
        let values = [1.23];
        let hash1 = Phenotype::compute_expressed_hash(&values);
        let hash2 = Phenotype::compute_expressed_hash(&values);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_multiple_values_when_compute_expressed_hash_then_returns_consistent_hash() {
        let values = [1.0, 2.5, -3.14, 0.0];
        let hash1 = Phenotype::compute_expressed_hash(&values);
        let hash2 = Phenotype::compute_expressed_hash(&values);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_different_values_when_compute_expressed_hash_then_returns_different_hashes() {
        let values1 = [1.0, 2.0];
        let values2 = [2.0, 1.0];
        let values3 = [1.0, 3.0];

        let hash1 = Phenotype::compute_expressed_hash(&values1);
        let hash2 = Phenotype::compute_expressed_hash(&values2);
        let hash3 = Phenotype::compute_expressed_hash(&values3);

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
    fn given_values_with_zero_and_negative_zero_when_compute_expressed_hash_then_hashes_differ_if_bits_differ()
     {
        let val_positive_zero = 0.0f64;
        let val_negative_zero = -0.0f64;

        assert_ne!(val_positive_zero.to_bits(), val_negative_zero.to_bits());

        let hash_positive_zero = Phenotype::compute_expressed_hash(&[val_positive_zero]);
        let hash_negative_zero = Phenotype::compute_expressed_hash(&[val_negative_zero]);

        assert_ne!(
            hash_positive_zero, hash_negative_zero,
            "Hashes for 0.0 and -0.0 should differ as their bit patterns differ."
        );
    }

    #[test]
    fn given_values_with_nan_when_compute_expressed_hash_then_consistent_for_same_nan_bits() {
        let nan1 = f64::NAN;
        let nan2 = f64::NAN;

        let hash_nan1 = Phenotype::compute_expressed_hash(&[nan1]);
        let hash_nan2 = Phenotype::compute_expressed_hash(&[nan2]);

        if nan1.to_bits() == nan2.to_bits() {
            assert_eq!(
                hash_nan1, hash_nan2,
                "Hashes for NaNs with identical bit patterns should be the same."
            );
        } else {
            assert_ne!(
                hash_nan1, hash_nan2,
                "Hashes for NaNs with different bit patterns should differ."
            );
        }

        let values_with_nan = [1.0, f64::NAN, 3.0];
        let hash_values_nan1 = Phenotype::compute_expressed_hash(&values_with_nan);
        let hash_values_nan2 = Phenotype::compute_expressed_hash(&values_with_nan);
        assert_eq!(
            hash_values_nan1, hash_values_nan2,
            "Hash of slice containing NaN should be consistent."
        );
    }
}
