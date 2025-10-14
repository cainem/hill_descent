use super::Regions;
use crate::world::organisms::{organism::Organism, Organisms};
use crate::world::regions::derive_region_seed;
use crate::world::world_function::WorldFunction;
use rayon::prelude::*;
use std::sync::Arc;

impl Regions {
    /// Processes all regions in parallel (core parallelization point).
    /// Each region gets dedicated thread with deterministic RNG.
    pub fn parallel_process_regions(
        &mut self,
        world_function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: Option<&[f64]>,
        world_seed: u64,
    ) -> Organisms {
        let all_offspring: Vec<Vec<Organism>> = self
            .regions
            .par_iter_mut()
            .map(|(region_key, region)| {
                let region_seed = derive_region_seed(world_seed, region_key);
                region.process_region_lifecycle(world_function, inputs, known_outputs, region_seed)
            })
            .collect();

        // Collect all surviving organisms from regions
        let mut all_organisms: Vec<Arc<Organism>> = Vec::new();
        for (_key, region) in self.regions.iter() {
            for organism in region.organisms() {
                all_organisms.push(Arc::clone(organism));
            }
        }

        // Clear organisms from all regions (they will be redistributed in update)
        for (_key, region) in self.regions.iter_mut() {
            region.clear_organisms();
        }

        // Add offspring
        let all_offspring_flat: Vec<Arc<Organism>> = all_offspring
            .into_iter()
            .flat_map(|v| v.into_iter().map(Arc::new))
            .collect();
        all_organisms.extend(all_offspring_flat);

        Organisms::new_from_arc_vec(all_organisms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use crate::world::regions::region::Region;

    #[derive(Debug)]
    struct MockFunction;
    impl WorldFunction for MockFunction {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> { vec![1.0] }
    }

    fn create_test_organism() -> Arc<Organism> {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        Arc::new(Organism::new(phenotype, 0, (None, None)))
    }

    #[test]
    fn given_multiple_regions_when_parallel_process_then_all_processed() {
        let mut regions = Regions::new(&crate::parameters::global_constants::GlobalConstants::new(100, 10));
        for i in 0..3 {
            let mut region = Region::new();
            region.set_carrying_capacity(Some(10));
            for _ in 0..5 { region.add_organism(create_test_organism()); }
            regions.insert_region(vec![i], region);
        }

        let all_organisms = regions.parallel_process_regions(&MockFunction, &[], Some(&[1.0]), 12345);
        // 3 regions * (5 survivors + 5 offspring) = 30 total
        assert_eq!(all_organisms.len(), 30);
    }

    #[test]
    fn given_same_seed_when_parallel_process_then_deterministic_results() {
        let mut regions1 = Regions::new(&crate::parameters::global_constants::GlobalConstants::new(100, 10));
        let mut regions2 = Regions::new(&crate::parameters::global_constants::GlobalConstants::new(100, 10));
        
        for i in 0..5 {
            let mut r1 = Region::new();
            let mut r2 = Region::new();
            r1.set_carrying_capacity(Some(10));
            r2.set_carrying_capacity(Some(10));
            for _ in 0..5 {
                r1.add_organism(create_test_organism());
                r2.add_organism(create_test_organism());
            }
            regions1.insert_region(vec![i], r1);
            regions2.insert_region(vec![i], r2);
        }

        let all_organisms1 = regions1.parallel_process_regions(&MockFunction, &[], Some(&[1.0]), 12345);
        let all_organisms2 = regions2.parallel_process_regions(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(all_organisms1.len(), all_organisms2.len());
    }
}
