//! Combined epoch processing for all organisms.
//!
//! This module combines region key calculation, fitness evaluation, and age
//! increment into a single message pass per organism, reducing synchronization
//! barriers.

use std::sync::Arc;

use super::{
    World,
    regions::{OrganismEntry, RegionKey},
};
use crate::organism::{ProcessEpochRequest, ProcessEpochResponse, ProcessEpochResult};
use crate::world::dimensions::Dimensions;

impl World {
    /// Processes an epoch for all organisms using combined messages.
    ///
    /// This method sends `ProcessEpochRequest` to all organisms and handles:
    /// - Region key calculation
    /// - Fitness evaluation
    /// - Age increment
    ///
    /// If any organisms report OutOfBounds, dimensions are expanded and those
    /// organisms are retried.
    ///
    /// # Arguments
    ///
    /// * `training_data_index` - Index into training data (0 for function optimization)
    ///
    /// # Returns
    ///
    /// Tuple of (dimensions_changed, organisms_to_remove_due_to_age)
    /// - `dimensions_changed`: true if any dimensions were expanded
    /// - The dead organism IDs are collected for removal
    pub fn process_epoch_all(&mut self, training_data_index: usize) -> (bool, Vec<u64>) {
        let mut dimensions_changed = false;
        let mut pending_ids = self.organism_ids.clone();
        let mut all_results: Vec<(u64, ProcessEpochResult)> = Vec::new();
        let mut dimensions_to_send: Option<Arc<Dimensions>> = None;
        let mut changed_since_last_attempt = Vec::new();

        loop {
            // Send ProcessEpochRequest to all pending organisms
            let requests = pending_ids.iter().map(|&id| {
                ProcessEpochRequest(
                    id,
                    dimensions_to_send.clone(),
                    self.dimension_version,
                    changed_since_last_attempt.clone(),
                    training_data_index,
                )
            });

            let responses: Vec<ProcessEpochResponse> = self
                .organism_pool
                .send_and_receive(requests)
                .expect("Thread pool should be available")
                .collect();

            // Partition responses into Ok and OutOfBounds
            let mut out_of_bounds_dims: Vec<usize> = Vec::new();
            let mut out_of_bounds_ids: Vec<u64> = Vec::new();

            for response in responses {
                match &response.result {
                    ProcessEpochResult::Ok { .. } => {
                        all_results.push((response.id, response.result));
                    }
                    ProcessEpochResult::OutOfBounds {
                        dimensions_exceeded,
                    } => {
                        out_of_bounds_ids.push(response.id);
                        for &dim in dimensions_exceeded {
                            if !out_of_bounds_dims.contains(&dim) {
                                out_of_bounds_dims.push(dim);
                            }
                        }
                    }
                }
            }

            // If all organisms are in bounds, we're done
            if out_of_bounds_ids.is_empty() {
                break;
            }

            // Expand dimensions for out-of-bounds dimensions
            dimensions_changed = true;
            let mut new_dimensions = (*self.dimensions).clone();
            new_dimensions.expand_bounds_multiple(&out_of_bounds_dims);
            self.dimensions = Arc::new(new_dimensions);
            self.dimension_version += 1;

            // Track changed dimensions
            changed_since_last_attempt = out_of_bounds_dims;

            // Since dimensions changed, we must update EVERYONE and recalculate EVERYONE.
            // This ensures all organisms have the new dimensions and valid keys.
            pending_ids = self.organism_ids.clone();
            dimensions_to_send = Some(self.dimensions.clone());

            // Clear previous results as they might be invalid (based on old dimensions)
            all_results.clear();
        }

        // Now process all the successful results
        let mut entries: Vec<(RegionKey, OrganismEntry)> = Vec::with_capacity(all_results.len());
        let mut dead_organisms: Vec<u64> = Vec::new();
        let mut new_best_id: Option<u64> = None;

        for (id, result) in all_results {
            if let ProcessEpochResult::Ok {
                region_key,
                score,
                new_age,
                should_remove,
            } = result
            {
                // Track best score
                if score < self.best_score {
                    self.best_score = score;
                    self.best_organism_id = Some(id);
                    new_best_id = Some(id);
                }

                // Track dead organisms for later removal (AFTER reproduction)
                if should_remove {
                    dead_organisms.push(id);
                }

                // Create entry for region population
                // NOTE: Dead organisms ARE included so they can participate in reproduction
                // before being removed. This prevents guaranteed extinction in low-population
                // scenarios (e.g., carrying capacity of 1).
                let entry = OrganismEntry::new(id, new_age, Some(score));
                entries.push((region_key, entry));
            }
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

        (!dimensions_changed, dead_organisms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobalConstants, world::single_valued_function::SingleValuedFunction};
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_when_process_epoch_all_then_fitness_evaluated() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Initial best score should be MAX
        assert_eq!(world.best_score, f64::MAX);

        // Process epoch
        let (at_resolution_limit, dead_organisms) = world.process_epoch_all(0);

        // Best score should be updated
        assert!(world.best_score < f64::MAX);
        // We may or may not have dead organisms depending on random max_age values
        let _ = dead_organisms;
        // Check resolution limit flag
        let _ = at_resolution_limit;
    }

    #[test]
    fn given_world_when_process_epoch_all_then_regions_populated() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Process epoch
        world.process_epoch_all(0);

        // Regions should be populated (at least one should have organisms)
        let total_organisms: usize = world.regions.iter().count();
        assert!(
            total_organisms > 0,
            "At least one region should have organisms"
        );
    }

    #[test]
    fn given_out_of_bounds_organism_when_process_epoch_all_then_dimensions_expanded() {
        // Create organisms far from zero
        let bounds: Vec<RangeInclusive<f64>> = vec![100.0..=101.0, 100.0..=101.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 42);
        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Manually change dimensions to be near zero (disjoint from organisms)
        let shrunk_bounds = vec![0.0..=1.0, 0.0..=1.0];
        let new_dims = Arc::new(Dimensions::new(&shrunk_bounds));

        // Force update organisms to use new dimensions
        let requests = world
            .organism_ids
            .iter()
            .map(|&id| crate::organism::UpdateDimensionsRequest(id, new_dims.clone()));
        world
            .organism_pool
            .send_and_receive(requests)
            .expect("Pool should work")
            .for_each(drop);

        // Update world dimensions
        world.dimensions = new_dims;
        let initial_version = world.dimension_version;

        // Process epoch - should trigger expansion
        let (at_resolution_limit, _) = world.process_epoch_all(0);

        assert!(
            !at_resolution_limit,
            "Dimensions should have changed due to out-of-bounds organisms"
        );
        assert!(
            world.dimension_version > initial_version,
            "Dimension version should increment"
        );

        // Verify dimensions expanded to include the organisms (at least > 100)
        let dim = world.dimensions.get_dimension(0);
        assert!(
            *dim.range().end() >= 100.0,
            "Dimensions should have expanded to include organisms"
        );
    }

    #[test]
    fn given_new_best_score_when_process_epoch_all_then_best_params_updated() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 42);
        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Initial state
        assert!(world.best_params.is_empty());

        // Process epoch
        world.process_epoch_all(0);

        // Best params should be updated
        assert!(!world.best_params.is_empty());
        assert_eq!(world.best_params.len(), 1);
    }
}
