use super::World;

impl World {
    /// Removes organisms that have been marked as dead from the world.
    ///
    /// The procedure is single-threaded and cache-friendly:
    /// 1. In-place `retain` over the master `Organisms` vector.
    /// 2. For every region, `retain` again to purge dead entries.
    /// 3. Prune regions that became empty.
    ///
    /// The algorithm is O(population) in memory traffic and performs no
    /// intermediate allocations, so it is already close to optimal on a single
    /// thread. Later this can be parallelised by switching to Rayon’s
    /// `par_retain` without changing the public API.
    pub fn remove_dead(&mut self) {
        // 1. Master list ----------------------------------------------------
        self.organisms.retain_live();

        // 2. Each region ----------------------------------------------------
        for region in self.regions.iter_region_values_mut() {
            region.retain_live();
        }

        // 3. Drop regions that are now empty --------------------------------
        self.regions.retain_regions(|_, r| !r.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::super::organisms::Organisms;
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::organisms::organism::Organism;
    use crate::world::world_function::WorldFunction;
    use std::ops::RangeInclusive;
    use std::sync::Arc;

    // Mock WorldFunction that returns 0.0 to create zero-score scenarios.
    #[derive(Debug)]
    struct DummyFn;
    impl WorldFunction for DummyFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![0.0]
        }
    }

    fn default_bounds() -> Vec<RangeInclusive<f64>> {
        vec![0.0..=1.0]
    }
    fn default_sys_params() -> Vec<f64> {
        vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
    }

    #[test]
    fn given_no_dead_organisms_when_remove_dead_then_world_unchanged() {
        let gc = GlobalConstants::new(30, 10);
        let mut world = World::new(&default_bounds(), gc, Box::new(DummyFn));
        let region_count_before = world.regions.len();
        world.remove_dead();
        assert_eq!(world.organisms.len(), 30);
        assert_eq!(world.regions.len(), region_count_before);
    }

    #[test]
    fn given_some_dead_organisms_when_remove_dead_then_only_live_remain() {
        let gc = GlobalConstants::new(20, 10);
        let mut world = World::new(&default_bounds(), gc, Box::new(DummyFn));
        let mut it = world.organisms.iter();
        it.next().unwrap().mark_dead();
        it.next().unwrap().mark_dead();
        world.remove_dead();
        assert_eq!(world.organisms.len(), 18);
        assert!(world.organisms.iter().all(|o| !o.is_dead()));
        assert!(
            world
                .regions
                .iter_region_values()
                .all(|r| r.organisms().iter().all(|o| !o.is_dead()))
        );
    }

    #[test]
    fn given_all_dead_organisms_when_remove_dead_then_world_is_empty() {
        let gc = GlobalConstants::new(20, 10);
        let mut world = World::new(&default_bounds(), gc, Box::new(DummyFn));
        for o in world.organisms.iter() {
            o.mark_dead();
        }
        // Also mark the Rc clones stored in regions (they are distinct instances)
        for region in world.regions.iter_region_values() {
            for o in region.organisms() {
                o.mark_dead();
            }
        }
        world.remove_dead();
        assert_eq!(world.organisms.len(), 0);
        assert!(
            world
                .regions
                .iter_region_values()
                .all(|r| r.organisms().iter().all(|o| o.is_dead()))
        );
    }

    #[test]
    fn given_dead_organism_when_organisms_retain_live_then_removed() {
        let phenotype = Arc::new(Phenotype::new_for_test(default_sys_params()));
        let mut organisms = Organisms::new_from_organisms(vec![
            Organism::new(Arc::clone(&phenotype), 0, (None, None)),
            Organism::new(Arc::clone(&phenotype), 0, (None, None)),
        ]);
        organisms.iter().next().unwrap().mark_dead();
        organisms.retain_live();
        assert_eq!(organisms.len(), 1);
        assert!(!organisms.iter().next().unwrap().is_dead());
    }

    #[test]
    fn given_dead_organism_in_region_when_region_retain_live_then_removed() {
        use crate::world::regions::region::Region;
        let phenotype = Arc::new(Phenotype::new_for_test(default_sys_params()));
        let live = Arc::new(Organism::new(Arc::clone(&phenotype), 0, (None, None)));
        let dead = Arc::new(Organism::new(Arc::clone(&phenotype), 0, (None, None)));
        dead.mark_dead();
        let mut region = Region::new();
        region.add_organism(Arc::clone(&live));
        region.add_organism(Arc::clone(&dead));
        region.retain_live();
        assert_eq!(region.organism_count(), 1);
        assert!(Arc::ptr_eq(&region.organisms()[0], &live));
    }
}
