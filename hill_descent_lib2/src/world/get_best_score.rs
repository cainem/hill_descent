//! Get best score from the world.

use super::World;

impl World {
    /// Returns the best fitness score seen so far.
    ///
    /// Lower scores are better (minimization problem).
    ///
    /// # Returns
    ///
    /// The best (lowest) fitness score found, or `f64::MAX` if no evaluations
    /// have occurred yet.
    pub fn get_best_score(&self) -> f64 {
        self.best_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobalConstants, TrainingData, world::single_valued_function::SingleValuedFunction};
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_without_evaluations_when_get_best_score_then_returns_max() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let world = World::new(&bounds, constants, Box::new(SumOfSquares));

        assert_eq!(world.get_best_score(), f64::MAX);
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_score_then_returns_minimum() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Run training to evaluate organisms
        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // Best score should be less than MAX
        assert!(world.get_best_score() < f64::MAX);
        // For SumOfSquares, scores should be non-negative
        assert!(world.get_best_score() >= 0.0);
    }

    #[test]
    fn given_world_when_multiple_runs_then_best_score_improves_or_stays() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);
        let first_score = world.get_best_score();

        // Multiple runs should maintain or improve
        for _ in 0..5 {
            let training_data = TrainingData::None { floor_value: 0.0 };
            world.training_run(training_data);
        }

        assert!(world.get_best_score() <= first_score);
    }
}
