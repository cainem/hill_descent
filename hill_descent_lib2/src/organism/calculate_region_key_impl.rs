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
/// Uses incremental updates when possible to avoid full recalculation and allocation.
/// If only one dimension changed and we have a cached key, we update it in-place.
pub fn calculate_region_key(
    phenotype: &Arc<Phenotype>,
    dimensions: &Arc<Dimensions>,
    cached_region_key: Option<RegionKey>,
    _cached_dimension_version: u64,
    request_dimension_version: u64,
    changed_dimensions: &[usize],
) -> (CalculateRegionKeyResult, u64) {
    // Try incremental update if possible
    if let Some(mut key) = cached_region_key {
        if changed_dimensions.is_empty() {
            return (CalculateRegionKeyResult::Ok(key), request_dimension_version);
        }

        // Handle incremental updates for any number of changed dimensions
        let mut oob_dims = Vec::new();
        let mut update_success = true;

        for &dim_idx in changed_dimensions {
            let expressed_values = phenotype.expression_problem_values();

            // Ensure dim_idx is valid for expressed_values
            if dim_idx < expressed_values.len() {
                let value = expressed_values[dim_idx];
                let dimension = dimensions.get_dimension(dim_idx);

                if let Some(interval) = dimension.get_interval(value) {
                    key.update_position(dim_idx, interval);
                } else {
                    oob_dims.push(dim_idx);
                }
            } else {
                // Dimension index mismatch - fall back to full recalc
                update_success = false;
                break;
            }
        }

        if update_success {
            if !oob_dims.is_empty() {
                return (
                    CalculateRegionKeyResult::OutOfBounds(oob_dims),
                    request_dimension_version,
                );
            }
            return (CalculateRegionKeyResult::Ok(key), request_dimension_version);
        }
        // If update failed (e.g. invalid index), fall through to full recalculation
    }

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
    fn given_cached_region_key_when_calculate_then_returns_cached() {
        // This test verifies that we return the cached key if no dimensions changed
        let dimensions = create_test_dimensions(vec![0.0..=10.0]);
        let phenotype = create_test_phenotype(vec![5.0]);
        let cached_key = RegionKey::new(vec![99]); // "Wrong" value, but cached

        let (result, _) =
            calculate_region_key(&phenotype, &dimensions, Some(cached_key), 0, 1, &[]);

        // Should return cached value because changed_dimensions is empty
        match result {
            CalculateRegionKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[99]); // Returns cached key
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_cached_key_and_single_change_when_calculate_then_incremental_update() {
        // Setup: 2 dimensions.
        // Dim 0: [0, 10] (1 doubling -> [0,5), [5,10])
        // Dim 1: [0, 10] (0 doublings -> [0,10])
        let mut dim0 = Dimension::new(0.0..=10.0);
        dim0.set_number_of_doublings(1);
        let dim1 = Dimension::new(0.0..=10.0);

        let dimensions = Arc::new(Dimensions::new_for_test(vec![dim0, dim1]));

        // Phenotype: [2.5, 5.0] -> Region [0, 0]
        let phenotype = create_test_phenotype(vec![2.5, 5.0]);

        // Cached key says [1, 0] (incorrect for current value, but simulates old state)
        let cached_key = RegionKey::new(vec![1, 0]);

        // We say dimension 0 changed.
        // Value 2.5 is in interval 0 of dim 0.
        // So update should change key from [1, 0] to [0, 0].

        let (result, _) = calculate_region_key(
            &phenotype,
            &dimensions,
            Some(cached_key),
            0,
            1,
            &[0], // Only dim 0 changed
        );

        match result {
            CalculateRegionKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[0, 0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_cached_key_and_single_change_out_of_bounds_when_calculate_then_returns_out_of_bounds()
    {
        let mut dim0 = Dimension::new(0.0..=10.0);
        dim0.set_number_of_doublings(1);
        let dimensions = Arc::new(Dimensions::new_for_test(vec![dim0]));

        // Phenotype: [15.0] -> Out of bounds
        let phenotype = create_test_phenotype(vec![15.0]);
        let cached_key = RegionKey::new(vec![0]);

        let (result, _) =
            calculate_region_key(&phenotype, &dimensions, Some(cached_key), 0, 1, &[0]);

        match result {
            CalculateRegionKeyResult::OutOfBounds(dims) => {
                assert_eq!(dims, vec![0]);
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_cached_key_and_multiple_changes_when_calculate_then_incremental_update() {
        // Setup: 3 dimensions
        let dimensions = create_test_dimensions(vec![0.0..=10.0, 0.0..=10.0, 0.0..=10.0]);
        let phenotype = create_test_phenotype(vec![5.0, 5.0, 5.0]);

        // Cached key has wrong values for all dimensions
        // Dim 0: 99 (unchanged, should be preserved by incremental update)
        // Dim 1: 99 (changed, should be updated)
        // Dim 2: 99 (changed, should be updated)
        let cached_key = RegionKey::new(vec![99, 99, 99]);

        // Two dimensions changed (1 and 2)
        let (result, _) =
            calculate_region_key(&phenotype, &dimensions, Some(cached_key), 0, 1, &[1, 2]);

        match result {
            CalculateRegionKeyResult::Ok(key) => {
                // If incremental: [99, 0, 0] (Dim 0 preserved, Dims 1&2 updated)
                // If full recalc: [0, 0, 0] (All calculated from scratch)
                assert_eq!(key.values(), &[99, 0, 0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }
}
