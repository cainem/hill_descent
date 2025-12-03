//! Reproduction phase of training run.

use std::sync::Arc;

use rand::{Rng, SeedableRng, rngs::StdRng};

use super::World;
use crate::organism::{
    CreateOrganism, GetPhenotypeRequest, GetPhenotypeResponse, ReproduceRequest, ReproduceResponse,
};

impl World {
    /// Performs reproduction for all selected pairs.
    ///
    /// For each pair:
    /// 1. Gets phenotypes from both parents
    /// 2. Sends reproduce request to parent1 with parent2's phenotype
    /// 3. Creates new organisms from offspring phenotypes in the pool
    ///
    /// # Arguments
    ///
    /// * `pairs` - Tuples of (parent1_id, parent2_id)
    ///
    /// # Returns
    ///
    /// The number of offspring created (2 per pair).
    pub fn perform_reproduction(&mut self, pairs: Vec<(u64, u64)>) -> usize {
        if pairs.is_empty() {
            return 0;
        }

        // Create a deterministic RNG for reproduction seeds
        let mut rng = StdRng::seed_from_u64(self.world_seed.wrapping_add(self.dimension_version));

        // Collect unique parent IDs to request phenotypes
        let mut unique_ids: Vec<u64> = Vec::with_capacity(pairs.len() * 2);
        for (p1, p2) in &pairs {
            if !unique_ids.contains(p1) {
                unique_ids.push(*p1);
            }
            if !unique_ids.contains(p2) {
                unique_ids.push(*p2);
            }
        }

        // Step 1: Get phenotypes from all unique parents
        let phenotype_requests = unique_ids.iter().map(|&id| GetPhenotypeRequest(id));

        let phenotype_responses: Vec<GetPhenotypeResponse> = self
            .organism_pool
            .send_and_receive(phenotype_requests)
            .expect("Thread pool should be available")
            .collect();

        // Build a map of id -> phenotype
        let phenotype_map: std::collections::HashMap<u64, Arc<crate::phenotype::Phenotype>> =
            phenotype_responses
                .into_iter()
                .map(|resp| (resp.id, resp.result))
                .collect();

        // Step 2: Send reproduce requests to parent1 with parent2's phenotype
        let reproduce_requests: Vec<_> = pairs
            .iter()
            .map(|(p1_id, p2_id)| {
                let partner_phenotype = phenotype_map
                    .get(p2_id)
                    .expect("Parent2 phenotype should exist")
                    .clone();
                let reproduction_seed = rng.random::<u64>();
                ReproduceRequest(*p1_id, partner_phenotype, reproduction_seed)
            })
            .collect();

        let reproduce_responses: Vec<ReproduceResponse> = self
            .organism_pool
            .send_and_receive(reproduce_requests.into_iter())
            .expect("Thread pool should be available")
            .collect();

        // Step 3: Create new organisms from offspring phenotypes
        let mut offspring_count = 0;

        for response in reproduce_responses {
            let result = response.result;
            let (phenotype1, phenotype2) = result.offspring_phenotypes;
            let parent_ids = result.parent_ids;

            // Create first offspring
            let id1 = self.next_organism_id;
            self.next_organism_id += 1;

            let create1 = CreateOrganism {
                id: id1,
                parent_ids: (Some(parent_ids.0), Some(parent_ids.1)),
                phenotype: phenotype1,
                dimensions: Arc::clone(&self.dimensions),
                world_function: Arc::clone(&self.world_function),
            };

            self.organism_pool
                .send_and_receive_once(create1)
                .expect("Thread pool should be available");

            self.organism_ids.push(id1);
            offspring_count += 1;

            // Create second offspring
            let id2 = self.next_organism_id;
            self.next_organism_id += 1;

            let create2 = CreateOrganism {
                id: id2,
                parent_ids: (Some(parent_ids.0), Some(parent_ids.1)),
                phenotype: phenotype2,
                dimensions: Arc::clone(&self.dimensions),
                world_function: Arc::clone(&self.world_function),
            };

            self.organism_pool
                .send_and_receive_once(create2)
                .expect("Thread pool should be available");

            self.organism_ids.push(id2);
            offspring_count += 1;
        }

        offspring_count
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
    fn given_pairs_when_reproduce_then_offspring_created() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_count = world.organism_count();

        // Calculate region keys (required for fitness evaluation)
        world.calculate_region_keys();

        // Create some pairs (using first few organism IDs)
        let pairs = vec![(0, 1), (2, 3)];

        let offspring = world.perform_reproduction(pairs);

        assert_eq!(offspring, 4); // 2 pairs * 2 offspring each
        assert_eq!(world.organism_count(), initial_count + 4);
    }

    #[test]
    fn given_empty_pairs_when_reproduce_then_no_change() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(10, 2, 123);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_count = world.organism_count();

        let offspring = world.perform_reproduction(vec![]);

        assert_eq!(offspring, 0);
        assert_eq!(world.organism_count(), initial_count);
    }

    #[test]
    fn given_same_seed_when_reproduce_then_deterministic() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 456);

        let mut world1 = World::new(&bounds, constants, Box::new(TestFunction));
        let mut world2 = World::new(&bounds, constants, Box::new(TestFunction));

        world1.calculate_region_keys();
        world2.calculate_region_keys();

        let pairs = vec![(0, 1)];

        let offspring1 = world1.perform_reproduction(pairs.clone());
        let offspring2 = world2.perform_reproduction(pairs);

        // Both worlds should create the same number of offspring
        assert_eq!(offspring1, offspring2);
        assert_eq!(world1.organism_count(), world2.organism_count());
    }

    #[test]
    fn given_pairs_when_reproduce_then_organism_ids_incremented() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(10, 2, 789);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        let initial_next_id = world.next_organism_id;

        world.calculate_region_keys();

        let pairs = vec![(0, 1), (2, 3)];
        world.perform_reproduction(pairs);

        // 4 offspring created, next_organism_id should increase by 4
        assert_eq!(world.next_organism_id, initial_next_id + 4);
    }

    #[test]
    fn given_self_pairing_when_reproduce_then_creates_offspring() {
        // Test case where an organism pairs with itself (odd number of organisms)
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(10, 2, 101);

        let mut world = World::new(&bounds, constants, Box::new(TestFunction));

        world.calculate_region_keys();

        // Self-pairing: organism 0 pairs with itself
        let pairs = vec![(0, 0)];
        let offspring = world.perform_reproduction(pairs);

        assert_eq!(offspring, 2);
    }
}
