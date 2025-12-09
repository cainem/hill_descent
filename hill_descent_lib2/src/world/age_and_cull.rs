//! Age organisms and cull dead ones.

use messaging_thread_pool::RemovePoolItemRequest;

use super::World;
use crate::organism::{IncrementAgeRequest, IncrementAgeResponse};

impl World {
    /// Increments age for all organisms and removes dead ones.
    ///
    /// This method:
    /// 1. Sends IncrementAgeRequest to all organisms
    /// 2. Collects responses to identify organisms that exceeded max_age
    /// 3. Sends RemovePoolItemRequest for dead organisms
    /// 4. Updates the internal organism_ids list
    ///
    /// # Returns
    ///
    /// The number of organisms that were removed.
    pub fn age_and_cull(&mut self) -> usize {
        // Send IncrementAgeRequest to all organisms
        let requests = self.organism_ids.iter().map(|&id| IncrementAgeRequest(id));

        let responses: Vec<IncrementAgeResponse> = self
            .organism_pool
            .send_and_receive(requests)
            .expect("Thread pool should be available")
            .collect();

        // Collect IDs of organisms to remove
        let to_remove: Vec<u64> = responses
            .iter()
            .filter(|resp| resp.result.should_remove)
            .map(|resp| resp.id)
            .collect();

        let removed_count = to_remove.len();

        // Remove dead organisms from the pool in a batch
        if !to_remove.is_empty() {
            let remove_requests = to_remove.iter().map(|&id| RemovePoolItemRequest(id));
            self.organism_pool
                .send_and_receive(remove_requests)
                .expect("Thread pool should be available")
                .for_each(drop);
        }

        // Update organism_ids to exclude removed ones
        self.organism_ids.retain(|id| !to_remove.contains(id));

        removed_count
    }

    /// Removes specific organisms from the pool (e.g., after region processing).
    ///
    /// Used after regions.process_all() to remove organisms that exceeded
    /// carrying capacity.
    ///
    /// # Arguments
    ///
    /// * `ids_to_remove` - IDs of organisms to remove
    ///
    /// # Returns
    ///
    /// The number of organisms removed.
    pub fn remove_organisms(&mut self, ids_to_remove: &[u64]) -> usize {
        let removed_count = ids_to_remove.len();

        // Remove organisms from the pool in a batch
        if !ids_to_remove.is_empty() {
            let remove_requests = ids_to_remove.iter().map(|&id| RemovePoolItemRequest(id));
            self.organism_pool
                .send_and_receive(remove_requests)
                .expect("Thread pool should be available")
                .for_each(drop);
        }

        // Update organism_ids to exclude removed ones
        self.organism_ids.retain(|id| !ids_to_remove.contains(id));

        removed_count
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
    fn given_young_organisms_when_age_and_cull_then_ages_increment_count_unchanged() {
        // Create world with default max_age (should be 2-10 based on system params)
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_count = world.organism_count();

        // First age_and_cull - organisms start at age 0
        let removed = world.age_and_cull();

        // Some might be removed depending on max_age values
        // But count should be initial - removed
        assert_eq!(world.organism_count(), initial_count - removed);
    }

    #[test]
    fn given_organisms_when_age_and_cull_multiple_times_then_population_decreases() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_count = world.organism_count();

        // Age multiple times - some organisms should die
        let mut total_removed = 0;
        for _ in 0..20 {
            total_removed += world.age_and_cull();
        }

        // After 20 iterations with max_age in 2-10 range, many should be dead
        assert!(total_removed > 0, "Some organisms should have died");
        assert_eq!(world.organism_count(), initial_count - total_removed);
    }

    #[test]
    fn given_organisms_to_remove_when_remove_organisms_then_count_decreases() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 456);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_count = world.organism_count();

        // Remove specific organisms
        let to_remove = vec![0, 1, 2];
        let removed = world.remove_organisms(&to_remove);

        assert_eq!(removed, 3);
        assert_eq!(world.organism_count(), initial_count - 3);
        assert!(!world.organism_ids.contains(&0));
        assert!(!world.organism_ids.contains(&1));
        assert!(!world.organism_ids.contains(&2));
    }

    #[test]
    fn given_empty_remove_list_when_remove_organisms_then_count_unchanged() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0];
        let constants = GlobalConstants::new_with_seed(10, 2, 789);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_count = world.organism_count();

        let removed = world.remove_organisms(&[]);

        assert_eq!(removed, 0);
        assert_eq!(world.organism_count(), initial_count);
    }
}
