use rand::Rng;
use std::sync::Arc;

use crate::world::{
    organisms::{Organisms, organism::Organism},
    regions::Regions,
};

impl Regions {
    /// For every region, generate enough offspring to reach its carrying capacity
    /// and append those offspring to the supplied `organisms` collection.
    ///
    /// Offspring are not assigned a region key here – that is done later via
    /// `Organisms::update_all_region_keys` in the caller.
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, rng, organisms))
    )]
    pub fn repopulate<R: Rng>(&mut self, rng: &mut R, organisms: &mut Organisms) {
        let region_keys: Vec<Vec<usize>> = self.regions.keys().cloned().collect();
        for key in region_keys {
            if let Some(region) = self.regions.get_mut(&key) {
                let capacity = region
                    .carrying_capacity()
                    .expect("Region carrying_capacity not set before repopulate");
                if region.organism_count() >= capacity {
                    continue;
                }
                let deficit = capacity - region.organism_count();
                let offspring = region.reproduce(deficit, rng);
                // wrap offspring in Rc before extending
                let offspring_rc: Vec<Arc<Organism>> = offspring.into_iter().map(Arc::new).collect();
                organisms.extend(offspring_rc);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parameters::global_constants::GlobalConstants,
        phenotype::Phenotype,
        world::{organisms::Organism, regions::region::Region},
    };
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::sync::Arc;

    fn create_region_with_two(capacity: usize) -> Region {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(capacity));
        let phen = Phenotype::new_for_test(vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5]);
        for _ in 0..2 {
            let org = Organism::new(Arc::new(phen.clone()), 0, (None, None));
            region.add_organism(Arc::new(org));
        }
        region
    }

    #[test]
    fn given_regions_with_deficit_when_repopulate_then_offspring_collected() {
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);
        let region_key = vec![0];
        regions
            .regions
            .insert(region_key, create_region_with_two(5));

        let mut rng = SmallRng::seed_from_u64(0);
        let mut offspring = Organisms::new_from_organisms(Vec::new());
        regions.repopulate(&mut rng, &mut offspring);

        // deficit requested was 3 but only 2 parents available; expect at least 2 offspring
        assert!(offspring.len() >= 2);
    }
}
