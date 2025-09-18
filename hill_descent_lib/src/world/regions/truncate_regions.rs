use super::Regions;

impl Regions {
    /// Truncates regions that exceed their carrying capacity by marking the worst-scoring organisms as dead.
    ///
    /// This function iterates through all regions and checks if the current population exceeds the region's
    /// carrying capacity. For regions that are over capacity, it marks the worst-scoring organisms as dead,
    /// starting from the end of the sorted organism list (which contains the organisms with the worst scores).
    ///
    /// IMPORTANT TEMPORAL CONSTRAINTS / KNOWN ISSUE FROM DESIGN:
    /// Carrying capacities are currently computed at the END of the previous training run (in `update`).
    /// Scores for the current iteration are only known AFTER the evaluation that precedes this call.
    /// Therefore region capacities do NOT yet reflect the newly computed scores of newborn (age==1) organisms –
    /// those offspring were added after the last scoring phase and had no score when capacities were allocated.
    /// To avoid biasing the search by immediately eliminating these newly evaluated organisms before their
    /// scores can influence the next capacity calculation, organisms with age == 1 are NEVER truncated here.
    /// (They will naturally contribute to min_score/capacity in the subsequent `update` call.)
    ///
    /// If a region is so over‑capacity that removing only older organisms cannot bring it down to its carrying
    /// capacity, the region is temporarily allowed to remain over capacity. This intentional relaxation avoids
    /// prematurely culling all young genetic material. A future enhancement could trigger an immediate
    /// reallocation of capacities after scoring to remove this temporal mismatch.
    ///
    /// # Preconditions
    /// - Organisms within each region must be sorted by fitness score (best to worst) followed by age (older first).
    ///   This is typically ensured by calling `sort_regions()` before this function.
    /// - Regions should have their carrying capacity set. On the very first iteration all capacities are `None` –
    ///   in that case truncation is skipped completely.
    ///
    /// # Side Effects
    /// - Organisms that exceed the carrying capacity are marked as dead using `mark_dead()`.
    /// - The actual removal of dead organisms must be done separately by calling `remove_dead()`.
    ///
    /// # Example Usage
    /// ```rust,no_run
    /// // After sorting organisms and before reproduction
    /// // world.regions.sort_regions();       // Ensure proper sorting
    /// // world.regions.truncate_regions();   // Mark excess organisms as dead
    /// // world.remove_dead();                // Actually remove the dead organisms
    /// ```
    pub fn truncate_regions(&mut self) {
        // First iteration detection: if EVERY region has `None` carrying capacity we skip truncation.
        // Previous heuristic (treating zero as first round) failed when `None` was present.
        if self
            .regions
            .iter()
            .all(|(_, r)| r.carrying_capacity().is_none())
        {
            crate::trace!("truncate_regions: skipping (first iteration - capacities all None)");
            return; // Nothing to do safely on first iteration
        }

        for (_region_key, region) in self.regions.iter_mut() {
            // Check if carrying capacity has been set - on the first training iteration,
            // it may not be set yet since capacities are calculated in regions.update().
            // In that case, skip truncation for this iteration.
            let Some(carrying_capacity) = region.carrying_capacity() else {
                continue;
            };

            let current_population = region.organism_count();

            // Skip regions that are within capacity
            if current_population <= carrying_capacity {
                continue;
            }

            let mut excess = current_population - carrying_capacity;

            // We only consider organisms with age > 1 as eligible for truncation (see design rationale above).
            // Organisms are already sorted best .. worst (score asc, age desc) so we iterate from the tail.
            let organisms = region.organisms_mut();
            let mut culled = 0usize;
            for organism in organisms.iter().rev() {
                // worst first
                if excess == 0 {
                    break;
                }
                if organism.age() <= 1 {
                    continue; // protect new organisms
                }
                organism.mark_dead();
                excess -= 1;
                culled += 1;
            }

            if culled > 0 {
                crate::debug!(
                    "Region {:?}: population {} exceeds capacity {} – culled {} older organisms (remaining overflow: {})",
                    _region_key,
                    current_population,
                    carrying_capacity,
                    culled,
                    excess
                );
            } else {
                crate::trace!(
                    "Region {:?}: over capacity but only age<=1 organisms available; allowing temporary overflow (pop={}, cap={})",
                    _region_key,
                    current_population,
                    carrying_capacity
                );
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
        world::{organisms::organism::Organism, regions::region::Region},
    };
    use std::rc::Rc;

    fn test_phenotype() -> Rc<Phenotype> {
        // 7 system parameters + 1 problem parameter (values arbitrary for tests)
        Rc::new(Phenotype::new_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5,
        ]))
    }

    fn make_org(score: Option<f64>, age: usize) -> Rc<Organism> {
        let o = Organism::new(test_phenotype(), age, (None, None));
        o.set_score(score);
        Rc::new(o)
    }

    #[test]
    fn given_all_capacities_none_when_truncate_then_no_kill() {
        let gc = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&gc);
        let mut region = Region::new();
        region.add_organism(make_org(Some(1.0), 5));
        region.add_organism(make_org(Some(2.0), 3));
        regions.regions.insert(vec![0], region);

        regions.sort_regions();
        regions.truncate_regions();

        let region_ref = regions.regions.get(&vec![0]).unwrap();
        assert!(region_ref.organisms().iter().all(|o| !o.is_dead()));
    }

    #[test]
    fn given_over_capacity_with_mixed_ages_when_truncate_then_only_older_removed() {
        let gc = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&gc);
        let mut region = Region::new();
        // Add 5 organisms: ages 4,3,2,1,1 scores increasing (worse to best order not important yet)
        region.add_organism(make_org(Some(5.0), 4));
        region.add_organism(make_org(Some(4.0), 3));
        region.add_organism(make_org(Some(3.0), 2));
        region.add_organism(make_org(Some(2.0), 1));
        region.add_organism(make_org(Some(1.0), 1));
        // Manually set capacity to 2 so we need to remove 3 but only ages >1 can be removed (3 available)
        region.set_carrying_capacity(Some(2));
        regions.regions.insert(vec![0], region);

        regions.sort_regions();
        regions.truncate_regions();

        let region_ref = regions.regions.get(&vec![0]).unwrap();
        let survivors_age1: usize = region_ref
            .organisms()
            .iter()
            .filter(|o| o.age() == 1 && !o.is_dead())
            .count();
        assert_eq!(survivors_age1, 2, "age 1 organisms must survive");
        // All organisms with age>1 should be dead
        assert!(
            region_ref
                .organisms()
                .iter()
                .filter(|o| o.age() > 1)
                .all(|o| o.is_dead())
        );
    }

    #[test]
    fn given_over_capacity_only_age1_when_truncate_then_no_kill_and_overflow_allowed() {
        let gc = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&gc);
        let mut region = Region::new();
        region.add_organism(make_org(Some(3.0), 1));
        region.add_organism(make_org(Some(2.0), 1));
        region.add_organism(make_org(Some(1.0), 1));
        region.set_carrying_capacity(Some(1));
        regions.regions.insert(vec![0], region);

        regions.sort_regions();
        regions.truncate_regions();

        let region_ref = regions.regions.get(&vec![0]).unwrap();
        assert_eq!(
            region_ref.organism_count(),
            3,
            "All age 1 organisms retained"
        );
        assert!(region_ref.organisms().iter().all(|o| !o.is_dead()));
    }
}
