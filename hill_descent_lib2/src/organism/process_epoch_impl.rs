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
/// 1. Calculate region key from phenotype and dimensions
/// 2. If in bounds, evaluate fitness using world function
/// 3. Increment age and check against max_age
/// 4. Return combined result
pub fn process_epoch(
    phenotype: &Arc<Phenotype>,
    dimensions: &Arc<Dimensions>,
    world_function: &Arc<dyn WorldFunction + Send + Sync>,
    current_age: usize,
    _training_data_index: usize,
) -> ProcessEpochResult {
    let expressed_values = phenotype.expression_problem_values();

    // Step 1: Calculate region key
    match calculate_dimensions_key(expressed_values, dimensions) {
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

        let result = process_epoch(&phenotype, &dimensions, &world_function, 0, 0);

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

        let result = process_epoch(&phenotype, &dimensions, &world_function, 0, 0);

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

        let result = process_epoch(&phenotype, &dimensions, &world_function, 5, 0);

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

        let result = process_epoch(&phenotype, &dimensions, &world_function, 0, 0);

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
}
