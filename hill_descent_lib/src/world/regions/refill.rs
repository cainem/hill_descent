use crate::world::organisms::Organisms;
use std::sync::Arc;

impl super::Regions {
    /// Handles the successful update of all organism region keys.
    ///
    /// This function repopulates the regions with the organisms, prunes any
    /// regions that are now empty, and then determines if the simulation
    /// should continue dividing dimensions or stop.
    ///
    /// # Arguments
    ///
    /// * `organisms` - The organisms to add to regions
    ///
    /// # Returns
    ///
    /// Returns `true` if the simulation has reached a stable state and should
    /// stop, `false` otherwise.
    pub(super) fn refill(&mut self, organisms: &mut Organisms) {
        crate::trace!("refill: total organisms before: {}", organisms.len());

        // Before adding the current generation of organisms, clear the regions of any
        // organisms from the previous generation. This ensures the region state is
        // always in sync with the master organism list.
        for region in self.regions.values_mut() {
            region.clear_organisms();
            // Min scores are not cleared here - they are cleared manually where dimensions change
        }

        // Move organisms to avoid Arc cloning, but preserve capacity for the refill
        let initial_capacity = organisms.capacity();
        let orgs_to_move = std::mem::replace(organisms, Organisms::with_capacity(initial_capacity));
        self.add_organisms(orgs_to_move);

        // Populate back into the organisms collection to maintain the master list
        for region in self.regions.values() {
            for organism in region.organisms() {
                organisms.push(Arc::clone(organism));
            }
        }

        crate::trace!(
            "refill: total organisms after: {} (in regions: {})",
            organisms.len(),
            self.regions
                .values()
                .map(|r| r.organism_count())
                .sum::<usize>()
        );

        crate::debug!("regions before prune {}", self.regions.len());
        self.prune_empty_regions();
        crate::debug!("regions after prune {}", self.regions.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parameters::global_constants::GlobalConstants,
        phenotype::Phenotype,
        world::{
            organisms::organism::Organism,
            regions::{Regions, region::region_key::RegionKey},
        },
    };
    use std::sync::Arc;

    // Helper to create a test organism with a given score and region key
    fn create_test_organism_with_score_and_key(
        score: Option<f64>,
        region_key: Option<RegionKey>,
    ) -> Organism {
        let expressed_values = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]; // System params
        let phenotype = Arc::new(Phenotype::new_for_test(expressed_values));
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(score);
        organism.set_region_key(region_key);
        organism
    }

    fn rk(values: &[usize]) -> RegionKey {
        RegionKey::from(values)
    }

    #[test]
    fn given_empty_organisms_when_refill_then_all_regions_pruned() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        // Manually add some regions with min_scores to test they get cleared
        let mut region1 = crate::world::regions::region::Region::new();
        region1.set_min_score(Some(5.0));
        let mut region2 = crate::world::regions::region::Region::new();
        region2.set_min_score(Some(3.0));

        regions.regions.insert(rk(&[0, 0]), region1);
        regions.regions.insert(rk(&[1, 1]), region2);

        assert_eq!(regions.len(), 2);

        let mut empty_organisms = Organisms::new_from_organisms(vec![]);

        // Act
        regions.refill(&mut empty_organisms);

        // Assert
        assert_eq!(
            regions.len(),
            0,
            "All regions should be pruned when no organisms"
        );
    }

    #[test]
    fn given_organisms_with_scores_when_refill_then_min_scores_updated() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        // Start with some regions with old min_scores
        let mut region1 = crate::world::regions::region::Region::new();
        region1.set_min_score(Some(100.0)); // Old high score
        regions.regions.insert(rk(&[0, 0]), region1);

        // Create organisms with better scores
        let organism1 = create_test_organism_with_score_and_key(Some(5.0), Some(rk(&[0, 0])));
        let organism2 = create_test_organism_with_score_and_key(Some(3.0), Some(rk(&[0, 0])));

        let mut organisms = Organisms::new_from_organisms(vec![organism1, organism2]);

        // Act - manually clear min scores first to test the scenario where dimensions changed
        for region in regions.regions.values_mut() {
            region.set_min_score(None);
        }
        regions.refill(&mut organisms);

        // Assert
        assert_eq!(regions.len(), 1);
        let key = rk(&[0, 0]);
        let region = regions.get_region(&key).unwrap();
        assert_eq!(region.organism_count(), 2);
        assert_eq!(
            region.min_score(),
            Some(3.0),
            "Min score should be updated to the lowest score (3.0)"
        );
    }

    #[test]
    fn given_organisms_without_scores_when_refill_then_min_scores_remain_none() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        // Create organisms without scores
        let organism1 = create_test_organism_with_score_and_key(None, Some(rk(&[0, 0])));
        let organism2 = create_test_organism_with_score_and_key(None, Some(rk(&[1, 1])));

        let mut organisms = Organisms::new_from_organisms(vec![organism1, organism2]);

        // Act
        regions.refill(&mut organisms);

        // Assert
        assert_eq!(regions.len(), 2);
        let key1 = rk(&[0, 0]);
        let key2 = rk(&[1, 1]);
        let region1 = regions.get_region(&key1).unwrap();
        let region2 = regions.get_region(&key2).unwrap();
        assert_eq!(region1.min_score(), None);
        assert_eq!(region2.min_score(), None);
    }

    #[test]
    fn given_mixed_organisms_when_refill_then_empty_regions_pruned_and_populated_kept() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        // Start with regions that will become empty and populated
        let region1 = crate::world::regions::region::Region::new();
        let region2 = crate::world::regions::region::Region::new();
        let region3 = crate::world::regions::region::Region::new();

        regions.regions.insert(rk(&[0, 0]), region1);
        regions.regions.insert(rk(&[1, 1]), region2);
        regions.regions.insert(rk(&[2, 2]), region3);

        // Create organisms that only populate some regions
        let organism1 = create_test_organism_with_score_and_key(Some(5.0), Some(rk(&[0, 0])));
        let organism2 = create_test_organism_with_score_and_key(Some(3.0), Some(rk(&[2, 2])));

        let mut organisms = Organisms::new_from_organisms(vec![organism1, organism2]);

        // Act
        regions.refill(&mut organisms);

        // Assert
        assert_eq!(regions.len(), 2, "Only populated regions should remain");
        let key0 = rk(&[0, 0]);
        let key1 = rk(&[1, 1]);
        let key2 = rk(&[2, 2]);
        assert!(regions.get_region(&key0).is_some());
        assert!(
            regions.get_region(&key1).is_none(),
            "Empty region should be pruned"
        );
        assert!(regions.get_region(&key2).is_some());
    }

    #[test]
    fn given_organisms_with_zero_and_negative_scores_when_refill_then_zero_is_min_score() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        // Create organisms with zero/negative scores
        let organism1 = create_test_organism_with_score_and_key(Some(0.0), Some(rk(&[0, 0])));
        let organism2 = create_test_organism_with_score_and_key(Some(-1.0), Some(rk(&[0, 0])));
        let organism3 = create_test_organism_with_score_and_key(Some(5.0), Some(rk(&[0, 0]))); // Higher score

        let mut organisms = Organisms::new_from_organisms(vec![organism1, organism2, organism3]);

        // Act
        regions.refill(&mut organisms);

        // Assert
        let key = rk(&[0, 0]);
        let region = regions.get_region(&key).unwrap();
        assert_eq!(region.organism_count(), 3);
        assert_eq!(
            region.min_score(),
            Some(-1.0),
            "Negative scores are better than zero or positive scores"
        );
    }
}
