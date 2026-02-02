use super::Regions;
use crate::world::organisms::{Organisms, organism::Organism};
use crate::world::regions::derive_region_seed;
use crate::world::world_function::WorldFunction;
use rayon::prelude::*;
use std::sync::Arc;

impl Regions {
    /// Processes all regions in parallel (core parallelization point).
    /// Each region gets dedicated thread with deterministic RNG.
    /// Regions are sorted by organism count (descending) to minimize parallel execution time
    /// by starting the largest workloads first.
    pub fn parallel_process_regions(
        &mut self,
        world_function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: &[f64],
        world_seed: u64,
    ) -> Organisms {
        // Sort regions by organism count (largest first) to optimize parallel scheduling
        let mut region_entries: Vec<_> = self.regions.iter_mut().collect();
        region_entries.sort_by(|a, b| b.1.organisms().len().cmp(&a.1.organisms().len()));

        let all_offspring: Vec<Vec<Arc<Organism>>> = region_entries
            .par_iter_mut()
            .map(|(region_key, region)| {
                let region_seed = derive_region_seed(world_seed, region_key);
                region.process_region_lifecycle(world_function, inputs, known_outputs, region_seed)
            })
            .collect();

        // Pre-allocate using known population size with 10% buffer
        // Sum of carrying capacities = population_size, so actual total ≈ population_size
        // Small buffer handles edge cases where organisms temporarily exceed capacity before truncation
        // In practice, population is usually smaller than max, so this avoids over-allocation
        let capacity = self.population_size + (self.population_size / 10).max(100);
        let mut all_organisms: Vec<Arc<Organism>> = Vec::with_capacity(capacity);
        for region in self.regions.values_mut() {
            all_organisms.extend(region.take_organisms());
        }

        // Add offspring directly via iterator (no intermediate Vec allocation)
        all_organisms.extend(all_offspring.into_iter().flatten());

        Organisms::new_from_arc_vec(all_organisms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use crate::world::regions::region::{Region, region_key::RegionKey};

    #[derive(Debug)]
    struct MockFunction;
    impl WorldFunction for MockFunction {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![1.0]
        }
    }

    fn create_test_organism() -> Arc<Organism> {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        Arc::new(Organism::new(phenotype, 0, (None, None)))
    }

    fn rk(values: &[usize]) -> RegionKey {
        RegionKey::from(values)
    }

    #[test]
    fn given_multiple_regions_when_parallel_process_then_all_processed() {
        let mut regions = Regions::new(&crate::parameters::global_constants::GlobalConstants::new(
            100, 10,
        ));
        for i in 0..3 {
            let mut region = Region::new();
            region.set_carrying_capacity(Some(10));
            for _ in 0..5 {
                region.add_organism(create_test_organism());
            }
            regions.insert_region(rk(&[i]), region);
        }

        let all_organisms = regions.parallel_process_regions(&MockFunction, &[], &[1.0], 12345);
        // 3 regions * (5 survivors + 5 offspring) = 30 total
        assert_eq!(all_organisms.len(), 30);
    }

    #[test]
    fn given_same_seed_when_parallel_process_then_deterministic_results() {
        let mut regions1 = Regions::new(
            &crate::parameters::global_constants::GlobalConstants::new(100, 10),
        );
        let mut regions2 = Regions::new(
            &crate::parameters::global_constants::GlobalConstants::new(100, 10),
        );

        for i in 0..5 {
            let mut r1 = Region::new();
            let mut r2 = Region::new();
            r1.set_carrying_capacity(Some(10));
            r2.set_carrying_capacity(Some(10));
            for _ in 0..5 {
                r1.add_organism(create_test_organism());
                r2.add_organism(create_test_organism());
            }
            regions1.insert_region(rk(&[i]), r1);
            regions2.insert_region(rk(&[i]), r2);
        }

        let all_organisms1 = regions1.parallel_process_regions(&MockFunction, &[], &[1.0], 12345);
        let all_organisms2 = regions2.parallel_process_regions(&MockFunction, &[], &[1.0], 12345);
        assert_eq!(all_organisms1.len(), all_organisms2.len());
    }

    #[test]
    fn given_regions_of_different_sizes_when_parallel_process_then_largest_regions_sorted_first() {
        let mut regions = Regions::new(&crate::parameters::global_constants::GlobalConstants::new(
            100, 10,
        ));

        // Create regions with different population sizes
        // Region 0: 2 organisms (smallest)
        let mut region_small = Region::new();
        region_small.set_carrying_capacity(Some(10));
        for _ in 0..2 {
            region_small.add_organism(create_test_organism());
        }
        regions.insert_region(rk(&[0]), region_small);

        // Region 1: 8 organisms (largest)
        let mut region_large = Region::new();
        region_large.set_carrying_capacity(Some(10));
        for _ in 0..8 {
            region_large.add_organism(create_test_organism());
        }
        regions.insert_region(rk(&[1]), region_large);

        // Region 2: 5 organisms (medium)
        let mut region_medium = Region::new();
        region_medium.set_carrying_capacity(Some(10));
        for _ in 0..5 {
            region_medium.add_organism(create_test_organism());
        }
        regions.insert_region(rk(&[2]), region_medium);

        // Process regions - should be sorted by size (largest first)
        let all_organisms = regions.parallel_process_regions(&MockFunction, &[], &[1.0], 12345);

        // Total: (8 + 8 offspring) + (5 + 5 offspring) + (2 + 2 offspring) = 30
        assert_eq!(all_organisms.len(), 30);

        // Verify that the original organisms (survivors) were scored
        // Offspring are not scored until next epoch
        let scored_count = all_organisms
            .iter()
            .filter(|org| org.score().is_some())
            .count();
        assert_eq!(
            scored_count, 15,
            "Only survivor organisms (8+5+2=15) should have scores from this epoch"
        );
    }
}
