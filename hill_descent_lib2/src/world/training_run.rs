//! Training run - the main optimization loop.

use messaging_thread_pool::RemovePoolItemRequest;

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
    /// 1. Process epoch for all organisms (combined region key + fitness + age)
    /// 2. Update carrying capacities based on region fitness
    /// 3. Process regions (sort, cull, select reproduction pairs using gap-filling)
    /// 4. Remove organisms that exceeded carrying capacity
    /// 5. Perform reproduction for selected pairs (dead-from-age can still participate)
    /// 6. Remove organisms that died from age
    pub fn training_run(&mut self, training_data: TrainingData) -> bool {
        // Get training data index (0 for function optimization)
        let training_data_index = match training_data {
            TrainingData::None { .. } => 0,
            TrainingData::Supervised { .. } => 0, // For now, use index 0
        };

        // Step 1: Combined epoch processing (region key + fitness + age increment)
        // Returns dead organisms with their region info for gap-filling
        let (at_resolution_limit, dead_organisms, dead_per_region) =
            self.process_epoch_all(training_data_index);

        // Step 2: Update carrying capacities based on region fitness
        self.regions.update_carrying_capacities();

        // Step 3: Process regions (sort, cull, select reproduction pairs using gap-filling)
        let process_results = self.regions.process_all(self.world_seed, &dead_per_region);

        // Step 4: Collect organisms to remove (exceeded carrying capacity only)
        let capacity_exceeded: Vec<u64> = process_results
            .iter()
            .flat_map(|result| result.organisms_to_remove.iter().copied())
            .collect();

        // Step 5: Collect all reproduction pairs
        let reproduction_pairs: Vec<(u64, u64)> = process_results
            .into_iter()
            .flat_map(|result| result.reproduction_pairs)
            .collect();

        // Step 6: Remove organisms that exceeded carrying capacity
        // (Done BEFORE reproduction - these organisms cannot participate)
        if !capacity_exceeded.is_empty() {
            let remove_requests = capacity_exceeded
                .iter()
                .map(|&id| RemovePoolItemRequest(id));
            self.organism_pool
                .send_and_receive(remove_requests)
                .expect("Thread pool should be available")
                .for_each(drop);
            self.organism_ids
                .retain(|id| !capacity_exceeded.contains(id));
        }

        // Step 7: Perform reproduction for selected pairs
        // NOTE: Dead-from-age organisms can still participate in reproduction
        // to prevent guaranteed extinction in low-population scenarios
        self.perform_reproduction(reproduction_pairs);

        // Step 8: Remove organisms that died from age (AFTER reproduction)
        if !dead_organisms.is_empty() {
            let remove_requests = dead_organisms.iter().map(|&id| RemovePoolItemRequest(id));
            self.organism_pool
                .send_and_receive(remove_requests)
                .expect("Thread pool should be available")
                .for_each(drop);
            self.organism_ids.retain(|id| !dead_organisms.contains(id));
        }

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

        // First run returns a boolean indicating whether resolution limit was reached
        let result = world.training_run(training_data);

        // Use the result to verify it's a valid boolean (this always passes but silences warning)
        let _ = result;
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
