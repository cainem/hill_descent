//! Calculate region keys for all organisms.

use std::sync::Arc;

use super::World;
use crate::organism::{
    CalculateRegionKeyRequest, CalculateRegionKeyResponse, CalculateRegionKeyResult,
};
use crate::world::dimensions::Dimensions;

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
        let mut dimensions_to_send: Option<Arc<Dimensions>> = None;

        loop {
            // Send CalculateRegionKeyRequest to all pending organisms
            let requests = pending_ids.iter().map(|&id| {
                CalculateRegionKeyRequest(
                    id,
                    dimensions_to_send.clone(),
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

            // Since dimensions changed, we must update EVERYONE and recalculate EVERYONE.
            // This ensures all organisms have the new dimensions and valid keys.
            pending_ids = self.organism_ids.clone();
            dimensions_to_send = Some(self.dimensions.clone());
        }

        all_changed_dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GlobalConstants,
        organism::{CalculateRegionKeyRequest, CalculateRegionKeyResponse},
        world::single_valued_function::SingleValuedFunction,
    };
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

        // With initial bounds set to the param_range, organisms are created within bounds.
        // So no expansion should occur.
        assert!(
            changed.is_empty(),
            "Should be empty as organisms are created within bounds"
        );
    }

    #[test]
    fn given_shrunken_dimensions_when_calculate_then_dimensions_expand() {
        // 1. Create world with normal bounds
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);
        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        // 2. Create tiny dimensions
        // Organisms are at random positions in [-10, 10], so they will be outside [0, 0.001]
        let tiny_bounds = vec![0.0..=0.001, 0.0..=0.001];
        let tiny_dims = Arc::new(Dimensions::new(&tiny_bounds));

        // 3. Manually update world dimensions
        world.dimensions = tiny_dims.clone();
        world.dimension_version += 1;

        // 4. Force organisms to adopt tiny dimensions and calculate keys
        // This puts them in an "OutOfBounds" state relative to their stored dimensions
        let requests = world.organism_ids.iter().map(|&id| {
            CalculateRegionKeyRequest(
                id,
                Some(tiny_dims.clone()),
                world.dimension_version,
                vec![], // No specific changed dims, full recalc
            )
        });

        // We must consume the responses to ensure processing completes
        let _responses: Vec<CalculateRegionKeyResponse> = world
            .organism_pool
            .send_and_receive(requests)
            .expect("Thread pool should be available")
            .collect();

        let initial_version = world.dimension_version;

        // 5. Now run calculate_region_keys
        // It should detect the OutOfBounds state and expand
        let changed = world.calculate_region_keys();

        // 6. Verify expansion
        assert!(!changed.is_empty(), "Should have expanded dimensions");
        assert!(
            world.dimension_version > initial_version,
            "Version should increment"
        );

        // Verify dimensions expanded back to cover the organisms (roughly)
        // The expansion logic increases bounds by 50% or doubles them, so they should be > 0.001
        let dims = world.dimensions.get_dimensions();
        assert!(dims[0].range().end() > &0.001 || dims[0].range().start() < &0.0);
    }

    #[test]
    fn given_specific_dimension_shrunken_when_calculate_then_only_that_dimension_expands() {
        // 1. Create world
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);
        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        // 2. Shrink ONLY dimension 1. Dimension 0 stays [-10, 10].
        // Organisms are in [-10, 10].
        // So Dim 0 is fine. Dim 1 is OOB.
        let mixed_bounds = vec![-10.0..=10.0, 0.0..=0.001];
        let mixed_dims = Arc::new(Dimensions::new(&mixed_bounds));

        world.dimensions = mixed_dims.clone();
        world.dimension_version += 1;

        // 3. Force update organisms
        let requests = world.organism_ids.iter().map(|&id| {
            CalculateRegionKeyRequest(
                id,
                Some(mixed_dims.clone()),
                world.dimension_version,
                vec![],
            )
        });
        let _responses: Vec<CalculateRegionKeyResponse> = world
            .organism_pool
            .send_and_receive(requests)
            .expect("Thread pool should be available")
            .collect();

        // 4. Calculate
        let changed = world.calculate_region_keys();

        // 5. Verify
        assert!(changed.contains(&1), "Dimension 1 should change");
        assert!(!changed.contains(&0), "Dimension 0 should NOT change");
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
}
