//! Region key calculation implementation for organisms.
//!
//! Calculates which spatial region an organism belongs to based on its
//! expressed phenotype values and the current dimension bounds.

use std::sync::Arc;

use crate::{
    phenotype::Phenotype,
    world::{
        dimensions::{
            Dimensions,
            calculate_dimensions_key::{CalculateDimensionsKeyResult, calculate_dimensions_key},
        },
        regions::region_key::RegionKey,
    },
};

use super::CalculateRegionKeyResult;

/// Calculates the organism's region key based on its phenotype and dimensions.
///
/// # Arguments
///
/// * `phenotype` - The organism's genetic material
/// * `dimensions` - Current dimension bounds
/// * `cached_region_key` - Previously calculated region key (for incremental updates)
/// * `cached_dimension_version` - Version of dimensions when key was last calculated
/// * `request_dimension_version` - Version of dimensions in the request
/// * `changed_dimensions` - Indices of dimensions that changed since last calculation
///
/// # Returns
///
/// Tuple of (CalculateRegionKeyResult, new_dimension_version_to_cache).
///
/// * `Ok(RegionKey)` - Contains the calculated region key
/// * `OutOfBounds(Vec<usize>)` - Contains indices of dimensions where the organism exceeds bounds
///
/// # Algorithm
///
/// For simplicity in lib2, we always perform a full recalculation using the
/// `calculate_dimensions_key` function. The incremental update logic from lib1
/// is not needed because:
/// 1. We use message passing, so each organism handles its own state
/// 2. The overhead of full calculation is minimal compared to message passing
pub fn calculate_region_key(
    phenotype: &Arc<Phenotype>,
    dimensions: &Arc<Dimensions>,
    _cached_region_key: Option<&RegionKey>,
    _cached_dimension_version: u64,
    request_dimension_version: u64,
    _changed_dimensions: &[usize],
) -> (CalculateRegionKeyResult, u64) {
    let expressed_values = phenotype.expression_problem_values();

    match calculate_dimensions_key(expressed_values, dimensions) {
        CalculateDimensionsKeyResult::Ok(region_key) => (
            CalculateRegionKeyResult::Ok(region_key),
            request_dimension_version,
        ),
        CalculateDimensionsKeyResult::OutOfBounds {
            dimensions_exceeded,
        } => (
            CalculateRegionKeyResult::OutOfBounds(dimensions_exceeded),
            request_dimension_version,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::world::dimensions::Dimension;

    fn create_test_phenotype(problem_values: Vec<f64>) -> Arc<Phenotype> {
        // System parameters (6 values) + problem values
        let mut expressed = vec![0.5; NUM_SYSTEM_PARAMETERS];
        expressed[5] = 100.0; // max_age
        expressed.extend(problem_values);
        Arc::new(Phenotype::new_for_test(expressed))
    }

    fn create_test_dimensions(ranges: Vec<std::ops::RangeInclusive<f64>>) -> Arc<Dimensions> {
        let dimensions: Vec<Dimension> = ranges.into_iter().map(Dimension::new).collect();
        Arc::new(Dimensions::new_for_test(dimensions))
    }

    #[test]
    fn given_organism_within_bounds_when_calculate_region_key_then_returns_ok() {
        // 2D space: x in [0, 10], y in [-5, 5]
        let dimensions = create_test_dimensions(vec![0.0..=10.0, -5.0..=5.0]);
        // Phenotype with problem values at (5.0, 0.0)
        let phenotype = create_test_phenotype(vec![5.0, 0.0]);

        let (result, new_version) = calculate_region_key(&phenotype, &dimensions, None, 0, 1, &[]);

        assert!(matches!(result, CalculateRegionKeyResult::Ok(_)));
        assert_eq!(new_version, 1);
    }

    #[test]
    fn given_organism_outside_bounds_when_calculate_region_key_then_returns_out_of_bounds() {
        // 2D space: x in [0, 10], y in [-5, 5]
        let dimensions = create_test_dimensions(vec![0.0..=10.0, -5.0..=5.0]);
        // Phenotype with problem values at (15.0, 0.0) - x is out of bounds
        let phenotype = create_test_phenotype(vec![15.0, 0.0]);

        let (result, _) = calculate_region_key(&phenotype, &dimensions, None, 0, 1, &[]);

        match result {
            CalculateRegionKeyResult::OutOfBounds(dims) => {
                assert_eq!(dims, vec![0]); // First dimension out of bounds
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_organism_outside_multiple_bounds_when_calculate_then_returns_all_exceeded() {
        // 2D space: x in [0, 10], y in [-5, 5]
        let dimensions = create_test_dimensions(vec![0.0..=10.0, -5.0..=5.0]);
        // Phenotype with problem values at (-1.0, 10.0) - both out of bounds
        let phenotype = create_test_phenotype(vec![-1.0, 10.0]);

        let (result, _) = calculate_region_key(&phenotype, &dimensions, None, 0, 1, &[]);

        match result {
            CalculateRegionKeyResult::OutOfBounds(dims) => {
                assert_eq!(dims, vec![0, 1]); // Both dimensions out of bounds
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_dimension_version_when_calculate_then_returns_request_version() {
        let dimensions = create_test_dimensions(vec![0.0..=10.0]);
        let phenotype = create_test_phenotype(vec![5.0]);

        let (_, new_version) = calculate_region_key(&phenotype, &dimensions, None, 0, 42, &[]);

        assert_eq!(new_version, 42);
    }

    #[test]
    fn given_dimensions_with_doublings_when_calculate_then_returns_correct_region_key() {
        // Create dimension with 1 doubling (2 intervals)
        let mut dimension = Dimension::new(0.0..=10.0);
        dimension.set_number_of_doublings(1); // Intervals: [0,5), [5,10]
        let dimensions = Arc::new(Dimensions::new_for_test(vec![dimension]));

        // Value 7.5 should be in interval 1
        let phenotype = create_test_phenotype(vec![7.5]);

        let (result, _) = calculate_region_key(&phenotype, &dimensions, None, 0, 1, &[]);

        match result {
            CalculateRegionKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[1]);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_cached_region_key_when_calculate_then_still_recalculates() {
        // This test verifies that we always recalculate (simplified implementation)
        let dimensions = create_test_dimensions(vec![0.0..=10.0]);
        let phenotype = create_test_phenotype(vec![5.0]);
        let cached_key = RegionKey::new(vec![99]); // Wrong value

        let (result, _) =
            calculate_region_key(&phenotype, &dimensions, Some(&cached_key), 0, 1, &[]);

        // Should return correct value, not cached
        match result {
            CalculateRegionKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[0]); // Correct value, not 99
            }
            _ => panic!("Expected Ok result"),
        }
    }
}
