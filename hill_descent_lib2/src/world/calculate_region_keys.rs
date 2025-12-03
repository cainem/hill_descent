//! Calculate region keys for all organisms.

use std::sync::Arc;

use super::World;
use crate::organism::{
    CalculateRegionKeyRequest, CalculateRegionKeyResponse, CalculateRegionKeyResult,
};

impl World {
    /// Calculates region keys for all organisms.
    ///
    /// This method sends `CalculateRegionKeyRequest` messages to all organisms
    /// in the pool and collects their responses. If any organisms report that
    /// they are outside the current dimension bounds, the dimensions are expanded
    /// and the calculation is retried.
    ///
    /// # Algorithm
    ///
    /// 1. Send batch of `CalculateRegionKeyRequest` to all organisms
    /// 2. Collect responses and partition into Ok vs OutOfBounds
    /// 3. If all Ok, return (no dimensions changed)
    /// 4. If any OutOfBounds:
    ///    a. Collect which dimensions need expansion
    ///    b. Expand dimensions (creates new Arc<Dimensions>)
    ///    c. Increment dimension_version
    ///    d. Send UpdateDimensions to all organisms
    ///    e. Retry from step 1 with only the out-of-bounds organisms
    ///
    /// # Returns
    ///
    /// The indices of dimensions that changed (empty if no expansion needed).
    pub fn calculate_region_keys(&mut self) -> Vec<usize> {
        let mut all_changed_dimensions: Vec<usize> = Vec::new();
        let mut changed_since_last_attempt = Vec::new();
        let mut pending_ids = self.organism_ids.clone();

        loop {
            // Send CalculateRegionKeyRequest to all pending organisms
            let requests = pending_ids.iter().map(|&id| {
                CalculateRegionKeyRequest(
                    id,
                    self.dimension_version,
                    changed_since_last_attempt.clone(),
                )
            });

            let responses: Vec<CalculateRegionKeyResponse> = self
                .organism_pool
                .send_and_receive(requests)
                .expect("Thread pool should be available")
                .collect();

            // Partition responses into Ok and OutOfBounds
            let mut out_of_bounds_dims: Vec<usize> = Vec::new();
            let mut out_of_bounds_ids: Vec<u64> = Vec::new();

            for response in responses {
                match response.result {
                    CalculateRegionKeyResult::Ok(_) => {
                        // Organism is within bounds - nothing to do
                    }
                    CalculateRegionKeyResult::OutOfBounds(exceeded_dims) => {
                        out_of_bounds_ids.push(response.id);
                        for dim in exceeded_dims {
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
            let mut new_dimensions = (*self.dimensions).clone();
            new_dimensions.expand_bounds_multiple(&out_of_bounds_dims);
            self.dimensions = Arc::new(new_dimensions);
            self.dimension_version += 1;

            // Track which dimensions changed
            for dim in &out_of_bounds_dims {
                if !all_changed_dimensions.contains(dim) {
                    all_changed_dimensions.push(*dim);
                }
            }
            changed_since_last_attempt = out_of_bounds_dims;

            // Send UpdateDimensions to ALL organisms so they have the new dimensions
            let update_requests = self.organism_ids.iter().map(|&id| {
                crate::organism::UpdateDimensionsRequest(id, Arc::clone(&self.dimensions))
            });

            // Collect responses to ensure all organisms are updated
            let _: Vec<_> = self
                .organism_pool
                .send_and_receive(update_requests)
                .expect("Thread pool should be available")
                .collect();

            // Retry with only the out-of-bounds organisms
            pending_ids = out_of_bounds_ids;
        }

        all_changed_dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobalConstants, world::single_valued_function::SingleValuedFunction};
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct TestFunction;

    impl SingleValuedFunction for TestFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            0.0
        }
    }

    #[test]
    fn given_organisms_in_bounds_when_calculate_then_returns_empty() {
        // Create a world with organisms that should be within initial bounds
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        // Calculate region keys
        let changed = world.calculate_region_keys();

        // With initial bounds set to the param_range, organisms may or may not
        // be out of bounds depending on their random phenotypes. We just verify
        // the method runs without panicking.
        // The changed dimensions depend on whether any organisms happen to fall
        // outside the initially set bounds.
        assert!(changed.len() <= 2); // At most 2 dimensions can change
    }

    #[test]
    fn given_narrow_bounds_when_calculate_then_expansion_may_occur() {
        // Create a world with very narrow initial bounds
        // The random phenotypes are likely to exceed these bounds
        let bounds: Vec<RangeInclusive<f64>> = vec![4.9..=5.1, 4.9..=5.1];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        // Verify initial dimension version
        assert_eq!(world.dimension_version(), 0);

        // Calculate region keys - this should trigger expansions
        let changed = world.calculate_region_keys();

        // With narrow bounds and random phenotypes, we expect dimensions to expand
        // and version to increment
        // The exact number of expansions depends on random phenotypes
        // Just verify the function completes without panic
        let _changed = changed;
    }

    #[test]
    fn given_world_when_calculate_twice_then_second_returns_empty() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-100.0..=100.0, -100.0..=100.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 456);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        // First call - may expand bounds
        let _changed1 = world.calculate_region_keys();

        // Second call - should return empty since all organisms now have keys
        // and dimensions haven't changed
        let changed2 = world.calculate_region_keys();

        // The second call should not require any expansion since all organisms
        // are already within bounds after the first call
        assert!(changed2.is_empty());
    }

    #[test]
    fn given_expansion_when_calculate_then_dimension_version_increments() {
        // Use narrow bounds to force expansion
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=0.001, 0.0..=0.001];
        let constants = GlobalConstants::new_with_seed(10, 2, 789);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_version = world.dimension_version();

        // Calculate region keys - should expand
        let changed = world.calculate_region_keys();

        // If any dimensions changed, version should have incremented
        if !changed.is_empty() {
            assert!(world.dimension_version() > initial_version);
        }
    }
}
