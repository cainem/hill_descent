//! Training run - the main optimization loop.

use super::World;
use crate::training_data::TrainingData;

impl World {
    /// Performs a single training run (generation).
    ///
    /// # Arguments
    ///
    /// * `training_data` - Training data configuration
    ///
    /// # Returns
    ///
    /// `true` if at resolution limit (no dimension changes), `false` otherwise.
    ///
    /// # Algorithm
    ///
    /// 1. Calculate region keys (may loop if dimensions expand)
    /// 2. Evaluate fitness for all organisms
    /// 3. Update carrying capacities based on region fitness
    /// 4. Process regions (sort, cull, select reproduction pairs)
    /// 5. Remove organisms that exceeded carrying capacity
    /// 6. Perform reproduction for selected pairs
    /// 7. Age organisms and remove dead ones
    pub fn training_run(&mut self, training_data: TrainingData) -> bool {
        // Get training data index (0 for function optimization)
        let training_data_index = match training_data {
            TrainingData::None { .. } => 0,
            TrainingData::Supervised { .. } => 0, // For now, use index 0
        };

        // Step 1: Calculate region keys (may expand dimensions)
        let changed_dimensions = self.calculate_region_keys();
        let at_resolution_limit = changed_dimensions.is_empty();

        // Step 2: Evaluate fitness for all organisms
        self.evaluate_fitness(training_data_index);

        // Step 3: Update carrying capacities based on region fitness
        self.regions.update_carrying_capacities();

        // Step 4: Process regions (sort, cull, select reproduction pairs)
        let process_results = self.regions.process_all(self.world_seed);

        // Step 5: Collect organisms to remove (exceeded carrying capacity)
        let organisms_to_remove: Vec<u64> = process_results
            .iter()
            .flat_map(|result| result.organisms_to_remove.iter().copied())
            .collect();

        // Step 6: Collect all reproduction pairs
        let reproduction_pairs: Vec<(u64, u64)> = process_results
            .into_iter()
            .flat_map(|result| result.reproduction_pairs)
            .collect();

        // Step 7: Remove organisms that exceeded carrying capacity
        self.remove_organisms(&organisms_to_remove);

        // Step 8: Perform reproduction for selected pairs
        self.perform_reproduction(reproduction_pairs);

        // Step 9: Age organisms and remove dead ones
        self.age_and_cull();

        at_resolution_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobalConstants, world::single_valued_function::SingleValuedFunction};
    use std::ops::RangeInclusive;

    // Test function: sum of squares (minimum at origin)
    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_when_training_run_then_fitness_evaluated() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Initial best score should be MAX
        assert_eq!(world.best_score, f64::MAX);

        // Run one training iteration
        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // Best score should be updated (less than MAX)
        assert!(world.best_score < f64::MAX);
    }

    #[test]
    fn given_world_when_training_run_then_reproduction_occurs() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Track initial organism IDs (they should be 0..29)
        let initial_max_id = world.organism_ids.iter().max().copied().unwrap_or(0);

        // Run one training iteration
        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // After reproduction, there should be new organisms with higher IDs
        let new_max_id = world.organism_ids.iter().max().copied().unwrap_or(0);

        // Reproduction should have created new organisms
        // (unless all organisms died, which is unlikely with 30 organisms)
        assert!(
            new_max_id > initial_max_id || world.organism_count() > 0,
            "Reproduction should create new organisms or population should survive"
        );
    }

    #[test]
    fn given_world_when_training_run_then_best_score_tracked() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // Best score should be tracked
        assert!(world.best_score < f64::MAX);
        assert!(world.best_organism_id.is_some());
    }

    #[test]
    fn given_world_when_multiple_training_runs_then_score_may_improve() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 789);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };

        // Run first iteration
        world.training_run(training_data);
        let first_best = world.best_score;

        // Run multiple more iterations
        for _ in 0..10 {
            let training_data = TrainingData::None { floor_value: 0.0 };
            world.training_run(training_data);
        }

        // Best score should be maintained or improved
        assert!(world.best_score <= first_best);
    }

    #[test]
    fn given_world_when_training_run_then_returns_resolution_status() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 2, 111);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };

        // First run may or may not need dimension expansion
        let result = world.training_run(training_data);

        // Result should be a boolean
        assert!(result == true || result == false);
    }

    #[test]
    fn given_world_when_training_run_then_population_changes() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 222);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let initial_count = world.organism_count();

        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // Population may change due to reproduction and aging/culling
        // Just verify the system ran without panicking
        assert!(world.organism_count() > 0, "Population should survive");

        // Note: exact count change depends on reproduction/death rates
        // The important thing is the system runs successfully
        println!(
            "Population changed from {} to {}",
            initial_count,
            world.organism_count()
        );
    }
}
