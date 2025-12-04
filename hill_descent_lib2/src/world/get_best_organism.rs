//! Get best organism from the world.

use super::World;

impl World {
    /// Returns the ID of the best organism.
    ///
    /// # Returns
    ///
    /// The ID of the organism with the best fitness score, or `None` if
    /// no evaluations have occurred yet.
    pub fn get_best_organism_id(&self) -> Option<u64> {
        self.best_organism_id
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
    fn given_world_without_evaluations_when_get_best_organism_id_then_returns_none() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let world = World::new(&bounds, constants, Box::new(SumOfSquares));

        assert!(world.get_best_organism_id().is_none());
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_organism_id_then_returns_id() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Run training to evaluate organisms
        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // Should have a best organism ID
        assert!(world.get_best_organism_id().is_some());
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_organism_id_then_id_is_valid() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        let best_id = world.get_best_organism_id().unwrap();
        // The best organism ID should be in the current population
        assert!(world.organism_ids.contains(&best_id));
    }
}
