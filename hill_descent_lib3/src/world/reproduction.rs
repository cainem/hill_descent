//! Reproduction phase of training run.

use std::collections::HashMap;
use std::sync::Arc;

use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::prelude::*;

use super::World;
use crate::organism::{CreateOrganism, Organism};
use crate::phenotype::Phenotype;

impl World {
    /// Performs reproduction for all selected pairs.
    pub fn perform_reproduction(&mut self, pairs: Vec<(u64, u64)>) -> usize {
        if pairs.is_empty() {
            return 0;
        }

        // Create a deterministic RNG for reproduction seeds
        let mut rng = StdRng::seed_from_u64(self.world_seed.wrapping_add(self.dimension_version));

        // Collect unique parent IDs needed
        let mut unique_ids: Vec<u64> = Vec::with_capacity(pairs.len() * 2);
        for (p1, p2) in &pairs {
            if !unique_ids.contains(p1) {
                unique_ids.push(*p1);
            }
            if !unique_ids.contains(p2) {
                unique_ids.push(*p2);
            }
        }

        // Gather phenotypes for all parents
        // We scan the population once to find them.
        let unique_ids_ref = &unique_ids;
        let phenotypes: Vec<(u64, Arc<Phenotype>)> = self
            .organisms
            .par_iter()
            .filter_map(|org_lock| {
                let org = org_lock.read().unwrap();
                if unique_ids_ref.contains(&org.id()) {
                    Some((org.id(), org.phenotype().clone()))
                } else {
                    None
                }
            })
            .collect();

        // Build quick lookup map
        let phenotype_map: HashMap<u64, Arc<Phenotype>> = phenotypes.into_iter().collect();

        // Perform reproduction
        // We generate seeds sequentially for determinism matching lib2
        let tasks: Vec<_> = pairs
            .into_iter()
            .map(|(p1_id, p2_id)| {
                let seed = rng.random();
                (p1_id, p2_id, seed)
            })
            .collect();

        // Parallel reproduction
        let mut new_organisms_data: Vec<CreateOrganism> = Vec::new(); // Will be populated from results

        let reproduction_results: Vec<_> = tasks
            .par_iter()
            .map(|(p1_id, p2_id, seed)| {
                let p1_pheno = phenotype_map.get(p1_id).expect("Parent 1 not found");
                let p2_pheno = phenotype_map.get(p2_id).expect("Parent 2 not found");

                // We can use static method if we had one, or reproduce on the organism.
                // But we only have phenotypes here.
                // The Organism::reproduce method calls reproduce_impl::reproduce.
                // We can call reproduce_impl::reproduce directly.
                // Or create a temporary dummy organism? No.
                // Let's use reproduce_impl directly if accessible, or call it on a "virtual" organism logic.
                // Organism::reproduce method is:
                // reproduce_impl::reproduce(&self.phenotype, self.id, &partner_phenotype, seed)

                let (o1, o2) = Phenotype::sexual_reproduction(
                    p1_pheno,
                    p2_pheno,
                    &mut StdRng::seed_from_u64(*seed),
                );
                crate::organism::ReproduceResult {
                    offspring_phenotypes: (Arc::new(o1), Arc::new(o2)),
                    parent_ids: (*p1_id, *p2_id),
                }
            })
            .collect();

        // Create new organisms (sequential ID assignment needed for determinism)
        let count = reproduction_results.len() * 2;
        let mut new_ids: Vec<u64> = Vec::with_capacity(count);

        for result in reproduction_results {
            let (pheno1, pheno2) = result.offspring_phenotypes;
            let (p1, p2) = result.parent_ids;
            let parent_ids = (Some(p1), Some(p2));

            // Offspring 1
            let id1 = self.next_organism_id;
            self.next_organism_id += 1;
            new_ids.push(id1);

            new_organisms_data.push(CreateOrganism {
                id: id1,
                parent_ids,
                phenotype: pheno1,
                dimensions: Arc::clone(&self.dimensions),
                world_function: Arc::clone(&self.world_function),
            });

            // Offspring 2
            let id2 = self.next_organism_id;
            self.next_organism_id += 1;
            new_ids.push(id2);

            new_organisms_data.push(CreateOrganism {
                id: id2,
                parent_ids,
                phenotype: pheno2,
                dimensions: Arc::clone(&self.dimensions),
                world_function: Arc::clone(&self.world_function),
            });
        }

        // Add to world
        for init in new_organisms_data {
            let org = Organism::new(init);
            self.organisms.push(Arc::new(std::sync::RwLock::new(org)));
        }

        // Update organism_ids with all new IDs
        self.organism_ids.extend(new_ids);

        count
    }
}
