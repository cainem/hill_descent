//! Combined epoch processing implementation for organisms.
//!
//! Combines region key calculation, fitness evaluation, and age increment
//! into a single operation to reduce synchronization barriers.

use std::sync::Arc;

use crate::{
    phenotype::Phenotype,
    world::{
        WorldFunction,
        dimensions::{
            Dimensions,
            calculate_dimensions_key::{CalculateDimensionsKeyResult, calculate_dimensions_key},
        },
        regions::region_key::RegionKey,
    },
};

use super::ProcessEpochResult;

/// Processes an organism's epoch: calculates region key, evaluates fitness, and increments age.
///
/// # Arguments
///
/// * `phenotype` - The organism's genetic material
/// * `dimensions` - Current dimension bounds
/// * `world_function` - The fitness evaluation function
/// * `current_age` - The organism's current age
/// * `training_data_index` - Index into shared training data (ignored for function optimization)
/// * `cached_region_key` - Previously calculated region key (for incremental updates)
/// * `changed_dimensions` - Indices of dimensions that changed since last calculation
///
/// # Returns
///
/// `ProcessEpochResult` containing:
/// - Region key calculation result (Ok or OutOfBounds)
/// - Fitness score (if region key was Ok)
/// - Age-related info (new age, should_remove)
///
/// # Algorithm
///
/// 1. Calculate region key from phenotype and dimensions (using incremental update if possible)
/// 2. If in bounds, evaluate fitness using world function
/// 3. Increment age and check against max_age
/// 4. Return combined result
pub fn process_epoch(
    phenotype: &Arc<Phenotype>,
    dimensions: &Arc<Dimensions>,
    world_function: &Arc<dyn WorldFunction + Send + Sync>,
    current_age: usize,
    _training_data_index: usize,
    cached_region_key: Option<RegionKey>,
    changed_dimensions: &[usize],
) -> ProcessEpochResult {
    let expressed_values = phenotype.expression_problem_values();

    // Step 1: Calculate region key
    let region_key_result = 'calc: {
        // Try incremental update if possible
        if let Some(mut key) = cached_region_key {
            if changed_dimensions.is_empty() {
                break 'calc CalculateDimensionsKeyResult::Ok(key);
            }

            // Handle incremental updates for any number of changed dimensions
            let mut oob_dims = Vec::new();
            let mut update_success = true;

            for &dim_idx in changed_dimensions {
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
                    break 'calc CalculateDimensionsKeyResult::OutOfBounds {
                        dimensions_exceeded: oob_dims,
                    };
                }
                break 'calc CalculateDimensionsKeyResult::Ok(key);
            }
            // If update failed (e.g. invalid index), fall through to full recalculation
        }

        calculate_dimensions_key(expressed_values, dimensions)
    };

    match region_key_result {
        CalculateDimensionsKeyResult::OutOfBounds {
            dimensions_exceeded,
        } => {
            // Can't evaluate fitness if out of bounds
            ProcessEpochResult::OutOfBounds {
                dimensions_exceeded,
            }
        }
        CalculateDimensionsKeyResult::Ok(region_key) => {
            // Step 2: Evaluate fitness
            let inputs: &[f64] = &[];
            let outputs = world_function.run(expressed_values, inputs);
            let floor = world_function.function_floor();

            // Calculate fitness as distance from floor
            let score = (outputs[0] - floor).abs();

            // Step 3: Increment age and check death
            let new_age = current_age + 1;
            let max_age = phenotype.system_parameters().max_age();
            let should_remove = (new_age as f64) > max_age;

            ProcessEpochResult::Ok {
                region_key,
                score,
                new_age,
                should_remove,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::world::dimensions::Dimension;
    use crate::world::single_valued_function::SingleValuedFunction;

    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    fn create_test_phenotype(problem_values: Vec<f64>, max_age: f64) -> Arc<Phenotype> {
        let mut expressed = vec![0.5; NUM_SYSTEM_PARAMETERS];
        expressed[5] = max_age; // max_age is at index 5
        expressed.extend(problem_values);
        Arc::new(Phenotype::new_for_test(expressed))
    }

    fn create_test_dimensions(ranges: Vec<std::ops::RangeInclusive<f64>>) -> Arc<Dimensions> {
        let dimensions: Vec<Dimension> = ranges.into_iter().map(Dimension::new).collect();
        Arc::new(Dimensions::new_for_test(dimensions))
    }

    #[test]
    fn given_organism_in_bounds_when_process_epoch_then_returns_ok_with_score() {
        let phenotype = create_test_phenotype(vec![3.0, 4.0], 10.0);
        let dimensions = create_test_dimensions(vec![-10.0..=10.0, -10.0..=10.0]);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        let result = process_epoch(&phenotype, &dimensions, &world_function, 0, 0, None, &[]);

        match result {
            ProcessEpochResult::Ok { score, new_age, .. } => {
                assert_eq!(score, 25.0); // 3² + 4² = 25
                assert_eq!(new_age, 1);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_organism_out_of_bounds_when_process_epoch_then_returns_out_of_bounds() {
        let phenotype = create_test_phenotype(vec![15.0, 0.0], 10.0);
        let dimensions = create_test_dimensions(vec![-10.0..=10.0, -10.0..=10.0]);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        let result = process_epoch(&phenotype, &dimensions, &world_function, 0, 0, None, &[]);

        match result {
            ProcessEpochResult::OutOfBounds {
                dimensions_exceeded,
            } => {
                assert!(dimensions_exceeded.contains(&0));
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_organism_at_max_age_when_process_epoch_then_should_remove_true() {
        let phenotype = create_test_phenotype(vec![0.0, 0.0], 5.0);
        let dimensions = create_test_dimensions(vec![-10.0..=10.0, -10.0..=10.0]);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        let result = process_epoch(&phenotype, &dimensions, &world_function, 5, 0, None, &[]);

        match result {
            ProcessEpochResult::Ok {
                should_remove,
                new_age,
                ..
            } => {
                assert_eq!(new_age, 6);
                assert!(should_remove); // 6 > 5.0
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_young_organism_when_process_epoch_then_should_remove_false() {
        let phenotype = create_test_phenotype(vec![0.0, 0.0], 10.0);
        let dimensions = create_test_dimensions(vec![-10.0..=10.0, -10.0..=10.0]);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        let result = process_epoch(&phenotype, &dimensions, &world_function, 0, 0, None, &[]);

        match result {
            ProcessEpochResult::Ok {
                should_remove,
                new_age,
                ..
            } => {
                assert_eq!(new_age, 1);
                assert!(!should_remove); // 1 <= 10.0
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_cached_key_and_single_change_when_process_epoch_then_incremental_update() {
        let mut dim0 = Dimension::new(0.0..=10.0);
        dim0.set_number_of_doublings(1);
        let dimensions = Arc::new(Dimensions::new_for_test(vec![dim0]));
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        // Phenotype: [2.5] -> Region [0]
        let phenotype = create_test_phenotype(vec![2.5], 10.0);
        let cached_key = RegionKey::new(vec![1]); // Incorrect

        let result = process_epoch(
            &phenotype,
            &dimensions,
            &world_function,
            0,
            0,
            Some(cached_key),
            &[0],
        );

        match result {
            ProcessEpochResult::Ok { region_key, .. } => {
                assert_eq!(region_key.values(), &[0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_cached_key_and_single_change_out_of_bounds_when_process_epoch_then_out_of_bounds() {
        let mut dim0 = Dimension::new(0.0..=10.0);
        dim0.set_number_of_doublings(1);
        let dimensions = Arc::new(Dimensions::new_for_test(vec![dim0]));
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        let phenotype = create_test_phenotype(vec![15.0], 10.0);
        let cached_key = RegionKey::new(vec![0]);

        let result = process_epoch(
            &phenotype,
            &dimensions,
            &world_function,
            0,
            0,
            Some(cached_key),
            &[0],
        );

        match result {
            ProcessEpochResult::OutOfBounds {
                dimensions_exceeded,
            } => {
                assert_eq!(dimensions_exceeded, vec![0]);
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_cached_key_and_multiple_changes_when_process_epoch_then_incremental_update() {
        // Setup: 3 dimensions
        let dimensions = create_test_dimensions(vec![0.0..=10.0, 0.0..=10.0, 0.0..=10.0]);
        let phenotype = create_test_phenotype(vec![5.0, 5.0, 5.0], 10.0);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SumOfSquares);

        // Cached key has wrong values for all dimensions
        // Dim 0: 99 (unchanged, should be preserved by incremental update)
        // Dim 1: 99 (changed, should be updated)
        // Dim 2: 99 (changed, should be updated)
        let cached_key = RegionKey::new(vec![99, 99, 99]);

        // Two dimensions changed (1 and 2)
        let result = process_epoch(
            &phenotype,
            &dimensions,
            &world_function,
            0,
            0,
            Some(cached_key),
            &[1, 2],
        );

        match result {
            ProcessEpochResult::Ok { region_key, .. } => {
                // If incremental: [99, 0, 0] (Dim 0 preserved, Dims 1&2 updated)
                // If full recalc: [0, 0, 0] (All calculated from scratch)
                assert_eq!(region_key.values(), &[99, 0, 0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }
}
