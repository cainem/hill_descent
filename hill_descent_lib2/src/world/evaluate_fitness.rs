//! Evaluate fitness for all organisms.

use super::{
    World,
    regions::{OrganismEntry, RegionKey},
};
use crate::organism::{EvaluateFitnessRequest, EvaluateFitnessResponse};

impl World {
    /// Evaluates fitness for all organisms and populates regions.
    ///
    /// Sends EvaluateFitnessRequest to all organisms and collects responses.
    /// Updates best_score and best_organism_id if a better score is found.
    /// Populates regions with organism entries for capacity calculation and
    /// reproduction selection.
    ///
    /// # Arguments
    ///
    /// * `training_data_index` - Index into training data (0 for function optimization)
    ///
    /// # Side Effects
    ///
    /// - Updates `best_score` and `best_organism_id` if better score found
    /// - Populates regions with organism entries
    pub fn evaluate_fitness(&mut self, training_data_index: usize) {
        // Send EvaluateFitnessRequest to all organisms
        let requests = self
            .organism_ids
            .iter()
            .map(|&id| EvaluateFitnessRequest(id, training_data_index));

        let responses: Vec<EvaluateFitnessResponse> = self
            .organism_pool
            .send_and_receive(requests)
            .expect("Thread pool should be available")
            .collect();

        // Process responses and build region entries
        let mut entries: Vec<(RegionKey, OrganismEntry)> = Vec::with_capacity(responses.len());
        let mut new_best_id: Option<u64> = None;

        for response in responses {
            let id = response.id;
            let result = response.result;

            // Track best score
            if result.score < self.best_score {
                self.best_score = result.score;
                self.best_organism_id = Some(id);
                new_best_id = Some(id);
            }

            // Create entry for region population
            let entry = OrganismEntry::new(id, result.age, Some(result.score));
            entries.push((result.region_key, entry));
        }

        // If we have a new best organism, cache its params
        if let Some(best_id) = new_best_id {
            let phenotype_response = self
                .organism_pool
                .send_and_receive_once(crate::organism::GetPhenotypeRequest(best_id))
                .expect("Thread pool should be available");
            self.best_params = phenotype_response
                .result
                .expression_problem_values()
                .to_vec();
        }

        // Populate regions with organism entries
        self.regions.populate(entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobalConstants, world::single_valued_function::SingleValuedFunction};
    use std::ops::RangeInclusive;

    // Test function that returns the sum of squares
    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_when_evaluate_fitness_then_best_score_updated() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // First calculate region keys (required before fitness evaluation)
        world.calculate_region_keys();

        // Evaluate fitness
        world.evaluate_fitness(0);

        // Best score should be less than MAX
        assert!(world.best_score < f64::MAX);
    }

    #[test]
    fn given_world_when_evaluate_fitness_then_best_organism_id_set() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        world.calculate_region_keys();
        world.evaluate_fitness(0);

        assert!(world.best_organism_id.is_some());
    }

    #[test]
    fn given_world_when_evaluate_fitness_then_regions_populated() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        world.calculate_region_keys();

        // Regions should be empty before evaluate_fitness
        assert!(world.regions.is_empty());

        world.evaluate_fitness(0);

        // Regions should now be populated
        assert!(!world.regions.is_empty());
    }

    #[test]
    fn given_world_when_evaluate_multiple_times_then_best_score_improves_or_stays() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 789);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        world.calculate_region_keys();
        world.evaluate_fitness(0);

        let first_score = world.best_score;

        // Re-evaluate shouldn't make score worse (same organisms)
        world.evaluate_fitness(0);

        assert!(world.best_score <= first_score);
    }
}
