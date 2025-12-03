//! Fitness evaluation implementation for organisms.
//!
//! Evaluates the organism's fitness by passing its expressed phenotype values
//! through the world function.

use std::sync::Arc;

use crate::{
    phenotype::Phenotype,
    world::{WorldFunction, regions::region_key::RegionKey},
};

use super::EvaluateFitnessResult;

/// Evaluates the organism's fitness using the world function.
///
/// # Arguments
///
/// * `phenotype` - The organism's genetic material
/// * `world_function` - The fitness evaluation function
/// * `region_key` - The organism's current region key
/// * `age` - The organism's current age
/// * `training_data_index` - Index into shared training data (ignored for function optimization)
///
/// # Returns
///
/// Tuple of (EvaluateFitnessResult, calculated score for caching).
///
/// # Algorithm
///
/// 1. Extract expressed values from phenotype
/// 2. Call world_function.run() with expressed values
/// 3. Calculate fitness score as Euclidean distance from outputs to known_outputs (floor)
/// 4. Return result with score, age, and region key
///
/// # Panics
///
/// Panics if:
/// - The world function output is empty
/// - Any output contains non-finite values
/// - Any output is below the function floor
/// - The calculated score is non-finite
pub fn evaluate_fitness(
    phenotype: &Arc<Phenotype>,
    world_function: &Arc<dyn WorldFunction + Send + Sync>,
    region_key: &RegionKey,
    age: usize,
    _training_data_index: usize,
) -> (EvaluateFitnessResult, f64) {
    let phenotype_expressed_values = phenotype.expression_problem_values();

    // For SingleValuedFunction optimization, inputs are empty
    // and known_outputs is the function floor (single value)
    let inputs: &[f64] = &[];
    let outputs = world_function.run(phenotype_expressed_values, inputs);

    // Get the function floor for score calculation
    let floor = world_function.function_floor();
    let known_outputs = [floor];

    // Validate outputs
    assert!(
        !outputs.is_empty(),
        "World function must return at least one output"
    );
    assert_eq!(
        outputs.len(),
        known_outputs.len(),
        "Number of outputs ({}) must match number of known outputs ({})",
        outputs.len(),
        known_outputs.len()
    );

    for (i, (&output, &floor_val)) in outputs.iter().zip(known_outputs.iter()).enumerate() {
        assert!(
            output.is_finite(),
            "Output[{i}] = {output} is not finite (NaN or Infinity)"
        );
        assert!(
            output >= floor_val,
            "Output[{i}] = {output} is below the function floor {floor_val}"
        );
    }

    // Calculate fitness as Euclidean distance: sqrt(Σ(output_i - known_output_i)²)
    let sum_of_squares: f64 = outputs
        .iter()
        .zip(known_outputs.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum();

    let score = sum_of_squares.sqrt();

    assert!(
        score.is_finite(),
        "Fitness score must be finite, got: {score}"
    );

    let result = EvaluateFitnessResult {
        score,
        age,
        region_key: region_key.clone(),
    };

    (result, score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NUM_SYSTEM_PARAMETERS, world::single_valued_function::SingleValuedFunction};

    /// A simple test function: f(x) = x^2 (minimum at x=0, floor=0)
    #[derive(Debug)]
    struct SquareFunction;

    impl SingleValuedFunction for SquareFunction {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    /// A function with a non-zero floor
    #[derive(Debug)]
    struct ShiftedSquareFunction {
        floor_value: f64,
    }

    impl SingleValuedFunction for ShiftedSquareFunction {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum::<f64>() + self.floor_value
        }

        fn function_floor(&self) -> f64 {
            self.floor_value
        }
    }

    fn create_test_phenotype(problem_values: Vec<f64>) -> Arc<Phenotype> {
        // System parameters (6 values) + problem values
        let mut expressed = vec![0.5; NUM_SYSTEM_PARAMETERS];
        expressed[5] = 100.0; // max_age
        expressed.extend(problem_values);
        Arc::new(Phenotype::new_for_test(expressed))
    }

    fn create_test_region_key() -> RegionKey {
        RegionKey::new(vec![0, 0])
    }

    #[test]
    fn given_organism_when_evaluate_fitness_then_returns_correct_score() {
        let phenotype = create_test_phenotype(vec![3.0]); // Only problem value, no system params in test
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SquareFunction);
        let region_key = create_test_region_key();

        let (result, cached_score) =
            evaluate_fitness(&phenotype, &world_function, &region_key, 5, 0);

        // f(3) = 9, floor = 0, score = |9 - 0| = 9
        assert_eq!(result.score, 9.0);
        assert_eq!(cached_score, 9.0);
    }

    #[test]
    fn given_organism_when_evaluate_fitness_then_result_contains_age() {
        let phenotype = create_test_phenotype(vec![0.0]);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SquareFunction);
        let region_key = create_test_region_key();

        let (result, _) = evaluate_fitness(&phenotype, &world_function, &region_key, 42, 0);

        assert_eq!(result.age, 42);
    }

    #[test]
    fn given_organism_when_evaluate_fitness_then_result_contains_region_key() {
        let phenotype = create_test_phenotype(vec![0.0]);
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SquareFunction);
        let region_key = RegionKey::new(vec![1, 2, 3]);

        let (result, _) = evaluate_fitness(&phenotype, &world_function, &region_key, 0, 0);

        assert_eq!(result.region_key.values(), &[1, 2, 3]);
    }

    #[test]
    fn given_function_with_floor_when_evaluate_then_score_is_distance_from_floor() {
        let phenotype = create_test_phenotype(vec![2.0]); // f(2) = 4
        let world_function: Arc<dyn WorldFunction + Send + Sync> =
            Arc::new(ShiftedSquareFunction { floor_value: 10.0 }); // floor = 10, output = 4 + 10 = 14
        let region_key = create_test_region_key();

        let (result, _) = evaluate_fitness(&phenotype, &world_function, &region_key, 0, 0);

        // Output = 14, floor = 10, score = |14 - 10| = 4
        assert_eq!(result.score, 4.0);
    }

    #[test]
    fn given_optimal_params_when_evaluate_then_score_is_zero() {
        let phenotype = create_test_phenotype(vec![0.0]); // f(0) = 0
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SquareFunction);
        let region_key = create_test_region_key();

        let (result, _) = evaluate_fitness(&phenotype, &world_function, &region_key, 0, 0);

        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn given_multiple_params_when_evaluate_then_score_computed_correctly() {
        // f(x, y) = x^2 + y^2
        let phenotype = create_test_phenotype(vec![3.0, 4.0]); // 9 + 16 = 25
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::new(SquareFunction);
        let region_key = create_test_region_key();

        let (result, _) = evaluate_fitness(&phenotype, &world_function, &region_key, 0, 0);

        assert_eq!(result.score, 25.0);
    }
}
