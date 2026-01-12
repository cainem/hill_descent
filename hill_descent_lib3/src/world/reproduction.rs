//! Reproduction phase of training run.

use std::sync::{Arc, RwLock};

use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::prelude::*;

use super::World;
use crate::organism::Organism;
use crate::phenotype::Phenotype;

impl World {
    /// Performs reproduction for all selected pairs.
    ///
    /// For each pair, creates two offspring from the parents' phenotypes
    /// and adds them to the world.
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

        // Pre-generate all reproduction seeds for determinism
        // (seeds must be generated in order before parallel reproduction)
        let tasks: Vec<_> = pairs
            .into_iter()
            .map(|(p1_id, p2_id)| {
                let seed: u64 = rng.random();
                (p1_id, p2_id, seed)
            })
            .collect();

        // Fetch phenotypes for all unique parents using direct O(1) lookups
        // We need to collect phenotypes before parallel reproduction to avoid
        // holding locks during the parallel phase
        let phenotypes: Vec<_> = tasks
            .iter()
            .flat_map(|(p1_id, p2_id, seed)| {
                let p1_pheno = self
                    .organisms
                    .get(p1_id)
                    .expect("Parent 1 not found")
                    .read()
                    .unwrap()
                    .phenotype()
                    .clone();
                let p2_pheno = self
                    .organisms
                    .get(p2_id)
                    .expect("Parent 2 not found")
                    .read()
                    .unwrap()
                    .phenotype()
                    .clone();
                [((*p1_id, *p2_id, *seed), (p1_pheno, p2_pheno))]
            })
            .collect();

        // Parallel reproduction
        let reproduction_results: Vec<_> = phenotypes
            .par_iter()
            .map(|((p1_id, p2_id, seed), (p1_pheno, p2_pheno))| {
                let (o1, o2) = Phenotype::sexual_reproduction(
                    p1_pheno,
                    p2_pheno,
                    &mut StdRng::seed_from_u64(*seed),
                );
                ((*p1_id, *p2_id), (Arc::new(o1), Arc::new(o2)))
            })
            .collect();

        // Create and add new organisms (sequential for deterministic ID assignment)
        let count = reproduction_results.len() * 2;
        let dimensions = Arc::clone(&self.dimensions);
        let world_function = Arc::clone(&self.world_function);

        for ((p1_id, p2_id), (pheno1, pheno2)) in reproduction_results {
            let parent_ids = (Some(p1_id), Some(p2_id));

            // Offspring 1
            let id1 = self.next_organism_id;
            self.next_organism_id += 1;
            let org1 = Organism::new(
                id1,
                parent_ids,
                pheno1,
                Arc::clone(&dimensions),
                Arc::clone(&world_function),
            );
            self.organisms.insert(id1, Arc::new(RwLock::new(org1)));

            // Offspring 2
            let id2 = self.next_organism_id;
            self.next_organism_id += 1;
            let org2 = Organism::new(
                id2,
                parent_ids,
                pheno2,
                Arc::clone(&dimensions),
                Arc::clone(&world_function),
            );
            self.organisms.insert(id2, Arc::new(RwLock::new(org2)));
        }

        count
    }
}
