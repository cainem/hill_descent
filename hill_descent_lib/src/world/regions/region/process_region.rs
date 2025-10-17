use super::Region;
use crate::world::organisms::organism::Organism;
use crate::world::world_function::WorldFunction;
use rand::SeedableRng;
use rand::rngs::StdRng;

impl Region {
    /// Processes region's complete lifecycle independently (designed for parallel execution).
    /// Operations: Fitness → Sort → Truncate → Cull → Reproduce → Age → Cull
    pub fn process_region_lifecycle(
        &mut self,
        world_function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: Option<&[f64]>,
        region_seed: u64,
    ) -> Vec<Organism> {
        // 1. Fitness evaluation
        for organism in self.organisms.iter() {
            organism.run(world_function, inputs, known_outputs);
        }

        // 2. Sort by fitness (best first) then age (older first)
        self.organisms.sort_by(|a, b| {
            let score_a = a.score().unwrap_or(f64::INFINITY);
            let score_b = b.score().unwrap_or(f64::INFINITY);
            let score_cmp = score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal);
            score_cmp.then_with(|| b.age().cmp(&a.age()))
        });

        // 3. Truncate to capacity
        // Skip truncation if capacity is None or 0 (first iteration or no min_score)
        if let Some(capacity) = self.carrying_capacity
            && capacity > 0
            && self.organism_count() > capacity
        {
            for organism in self.organisms.iter().skip(capacity) {
                organism.mark_dead();
            }
        }

        // 4. Remove dead
        self.organisms.retain(|org| !org.is_dead());

        // 5. Reproduce offspring
        let offspring = if let Some(capacity) = self.carrying_capacity {
            let current = self.organism_count();
            if current < capacity {
                let mut region_rng = StdRng::seed_from_u64(region_seed);
                self.reproduce(capacity - current, &mut region_rng)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // 6. Age organisms
        for organism in self.organisms.iter() {
            organism.increment_age();
        }

        // 7. Remove aged-out
        self.organisms.retain(|org| !org.is_dead());

        offspring
    }

    /// Processes region's complete lifecycle independently, returning offspring as a Vec.
    /// Operations: Fitness → Sort → Truncate → Cull → Reproduce → Age → Cull
    /// 
    /// This version uses the iterator-based `reproduce_iter()` internally but collects the result
    /// into a Vec because we need to age organisms and remove aged-out ones after reproduction.
    /// The real benefit comes from parallel_process_regions not allocating Vec<Vec<>>.
    pub fn process_region_lifecycle_iter(
        &mut self,
        world_function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: Option<&[f64]>,
        region_seed: u64,
    ) -> Vec<Organism> {
        // 1. Fitness evaluation
        for organism in self.organisms.iter() {
            organism.run(world_function, inputs, known_outputs);
        }

        // 2. Sort by fitness (best first) then age (older first)
        self.organisms.sort_by(|a, b| {
            let score_a = a.score().unwrap_or(f64::INFINITY);
            let score_b = b.score().unwrap_or(f64::INFINITY);
            let score_cmp = score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal);
            score_cmp.then_with(|| b.age().cmp(&a.age()))
        });

        // 3. Truncate to capacity
        if let Some(capacity) = self.carrying_capacity
            && capacity > 0
            && self.organism_count() > capacity
        {
            for organism in self.organisms.iter().skip(capacity) {
                organism.mark_dead();
            }
        }

        // 4. Remove dead
        self.organisms.retain(|org| !org.is_dead());

        // 5. Reproduce offspring (using iterator version internally)
        let offspring = if let Some(capacity) = self.carrying_capacity {
            let current = self.organism_count();
            if current < capacity {
                let mut region_rng = StdRng::seed_from_u64(region_seed);
                self.reproduce_iter(capacity - current, &mut region_rng).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // 6. Age organisms
        for organism in self.organisms.iter() {
            organism.increment_age();
        }

        // 7. Remove aged-out
        self.organisms.retain(|org| !org.is_dead());

        offspring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockFunction;
    impl WorldFunction for MockFunction {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![1.0]
        }
    }

    fn create_test_organism(age: usize) -> Arc<Organism> {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        Arc::new(Organism::new(phenotype, age, (None, None)))
    }

    #[test]
    fn given_region_with_organisms_when_process_lifecycle_then_fitness_evaluated() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(10));
        for i in 0..5 {
            region.add_organism(create_test_organism(i));
        }

        let offspring = region.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);

        for org in region.organisms() {
            assert!(org.score().is_some());
        }
        assert_eq!(offspring.len(), 5);
    }

    #[test]
    fn given_region_over_capacity_when_process_lifecycle_then_truncated() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(3));
        for i in 0..5 {
            region.add_organism(create_test_organism(i));
        }

        region.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(region.organism_count(), 3);
    }

    #[test]
    fn given_same_seed_when_process_lifecycle_then_deterministic_offspring() {
        let mut region1 = Region::new();
        let mut region2 = Region::new();
        region1.set_carrying_capacity(Some(10));
        region2.set_carrying_capacity(Some(10));

        for i in 0..5 {
            region1.add_organism(create_test_organism(i));
            region2.add_organism(create_test_organism(i));
        }

        let offspring1 = region1.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
        let offspring2 = region2.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(offspring1.len(), offspring2.len());
    }

    // Tests for iterator version (uses reproduce_iter internally)
    #[test]
    fn given_region_with_organisms_when_process_lifecycle_iter_then_fitness_evaluated() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(10));
        for i in 0..5 {
            region.add_organism(create_test_organism(i));
        }

        let offspring = region.process_region_lifecycle_iter(&MockFunction, &[], Some(&[1.0]), 12345);

        for org in region.organisms() {
            assert!(org.score().is_some());
        }
        assert_eq!(offspring.len(), 5);
    }

    #[test]
    fn given_region_over_capacity_when_process_lifecycle_iter_then_truncated() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(3));
        for i in 0..5 {
            region.add_organism(create_test_organism(i));
        }

        let _offspring = region.process_region_lifecycle_iter(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(region.organism_count(), 3);
    }

    #[test]
    fn given_same_seed_when_process_lifecycle_iter_then_deterministic_offspring() {
        let mut region1 = Region::new();
        let mut region2 = Region::new();
        region1.set_carrying_capacity(Some(10));
        region2.set_carrying_capacity(Some(10));

        for i in 0..5 {
            region1.add_organism(create_test_organism(i));
            region2.add_organism(create_test_organism(i));
        }

        let offspring1 = region1.process_region_lifecycle_iter(&MockFunction, &[], Some(&[1.0]), 12345);
        let offspring2 = region2.process_region_lifecycle_iter(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(offspring1.len(), offspring2.len());
    }
}
