//! Get best parameters from the world.

use super::World;

impl World {
    /// Returns the expressed parameter values of the best organism.
    ///
    /// This returns only the problem parameters (excluding the 7 system parameters
    /// like mutation rates and max_age).
    ///
    /// # Returns
    ///
    /// The parameter values of the organism with the best fitness score,
    /// or an empty vector if no evaluations have occurred yet.
    ///
    /// Note: These parameters are cached when the best score is updated,
    /// so they remain available even if the best organism later dies.
    pub fn get_best_params(&self) -> Vec<f64> {
        self.best_params.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GlobalConstants, TrainingData, world::single_valued_function::SingleValuedFunction,
    };
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_without_evaluations_when_get_best_params_then_returns_empty() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let params = world.get_best_params();
        assert!(params.is_empty());
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_params_then_returns_best_organism_params() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Run training to evaluate organisms
        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        let params = world.get_best_params();

        // Should have same number of params as dimensions
        assert_eq!(params.len(), 2);

        // Params should be within bounds
        for &p in &params {
            assert!(
                (-10.0..=10.0).contains(&p),
                "Param {} should be within bounds",
                p
            );
        }
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_params_then_params_match_score() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Run multiple training iterations
        for _ in 0..10 {
            let training_data = TrainingData::None { floor_value: 0.0 };
            world.training_run(training_data);
        }

        let params = world.get_best_params();
        let best_score = world.get_best_score();

        // Verify that params produce the recorded best score
        let computed_score: f64 = params.iter().map(|x| x * x).sum();
        assert!(
            (computed_score - best_score).abs() < 1e-10,
            "Computed score {} should match best score {}",
            computed_score,
            best_score
        );
    }

    #[test]
    fn given_single_dimension_when_get_best_params_then_returns_single_value() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 789);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        let params = world.get_best_params();
        assert_eq!(params.len(), 1);
    }
}
