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
        for region in self.regions.regions_mut().values_mut() {
            region.retain_live();
        }

        // 3. Drop regions that are now empty --------------------------------
        self.regions.regions_mut().retain(|_, r| !r.is_empty());
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
    use std::rc::Rc;

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
        let gc = GlobalConstants::new(3, 10);
        let mut world = World::new(&default_bounds(), gc, Box::new(DummyFn));
        let region_count_before = world.regions.regions().len();
        world.remove_dead();
        assert_eq!(world.organisms.len(), 3);
        assert_eq!(world.regions.regions().len(), region_count_before);
    }

    #[test]
    fn given_some_dead_organisms_when_remove_dead_then_only_live_remain() {
        let gc = GlobalConstants::new(4, 10);
        let mut world = World::new(&default_bounds(), gc, Box::new(DummyFn));
        let mut it = world.organisms.iter();
        it.next().unwrap().mark_dead();
        it.next().unwrap().mark_dead();
        world.remove_dead();
        assert_eq!(world.organisms.len(), 2);
        assert!(world.organisms.iter().all(|o| !o.is_dead()));
        assert!(
            world
                .regions
                .regions()
                .values()
                .all(|r| r.organisms().iter().all(|o| !o.is_dead()))
        );
    }

    #[test]
    fn given_all_dead_organisms_when_remove_dead_then_world_is_empty() {
        let gc = GlobalConstants::new(2, 10);
        let mut world = World::new(&default_bounds(), gc, Box::new(DummyFn));
        for o in world.organisms.iter() {
            o.mark_dead();
        }
        // Also mark the Rc clones stored in regions (they are distinct instances)
        for region in world.regions.regions().values() {
            for o in region.organisms() {
                o.mark_dead();
            }
        }
        world.remove_dead();
        assert_eq!(world.organisms.len(), 0);
        assert!(
            world
                .regions
                .regions()
                .values()
                .all(|r| r.organisms().iter().all(|o| o.is_dead()))
        );
    }

    #[test]
    fn given_dead_organism_when_organisms_retain_live_then_removed() {
        let phenotype = Rc::new(Phenotype::new_for_test(default_sys_params()));
        let mut organisms = Organisms::new_from_organisms(vec![
            Organism::new(Rc::clone(&phenotype), 0),
            Organism::new(Rc::clone(&phenotype), 0),
        ]);
        organisms.iter().next().unwrap().mark_dead();
        organisms.retain_live();
        assert_eq!(organisms.len(), 1);
        assert!(!organisms.iter().next().unwrap().is_dead());
    }

    #[test]
    fn given_dead_organism_in_region_when_region_retain_live_then_removed() {
        use crate::world::regions::region::Region;
        let phenotype = Rc::new(Phenotype::new_for_test(default_sys_params()));
        let live = Rc::new(Organism::new(Rc::clone(&phenotype), 0));
        let dead = Rc::new(Organism::new(Rc::clone(&phenotype), 0));
        dead.mark_dead();
        let mut region = Region::new();
        region.add_phenotype(Rc::clone(&live));
        region.add_phenotype(Rc::clone(&dead));
        region.retain_live();
        assert_eq!(region.organism_count(), 1);
        assert!(Rc::ptr_eq(&region.organisms()[0], &live));
    }
}
