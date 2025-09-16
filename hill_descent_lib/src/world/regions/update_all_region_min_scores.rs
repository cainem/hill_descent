use crate::world::organisms::Organisms;
use crate::world::regions::Regions;

impl Regions {
    /// Updates the minimum score for each region based on the scores of the organisms within them.
    ///
    /// **DEPRECATED**: This function is now largely obsolete since min_scores are updated
    /// automatically when organisms are added to regions via `Region::add_organism()`.
    /// This function is kept for compatibility and potential edge cases, but should generally
    /// not be needed in the normal flow.
    ///
    /// This function iterates through all organisms. For each organism with a positive score,
    /// it checks if its score is lower than the current minimum score recorded for its region.
    /// If it is, or if the region has no minimum score yet, the region's minimum score is updated.
    /// The minimum scores are persistent and are only ever updated downwards.
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, all_organisms))
    )]
    #[allow(dead_code)] // Function kept for compatibility but no longer used in main flow
    pub(super) fn update_all_region_min_scores(&mut self, all_organisms: &Organisms) {
        for organism in all_organisms.iter() {
            if let (Some(key), Some(score)) = (organism.region_key(), organism.score()) {
                // Only consider positive scores as per PDD (fitness includes e0)
                if score > 0.0
                    && let Some(region) = self.regions.get_mut(&key)
                {
                    match region.min_score() {
                        Some(current_min) => {
                            if score < current_min {
                                region.set_min_score(Some(score));
                            }
                        }
                        None => {
                            region.set_min_score(Some(score));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::organisms::{Organism, Organisms};
    use crate::world::regions::Region;
    use indexmap::IndexMap;
    use rustc_hash::FxBuildHasher;
    use std::rc::Rc;

    fn create_test_regions_and_gc(
        target_regions: usize,
        population_size: usize,
    ) -> (Regions, GlobalConstants) {
        if population_size == 0 {
            let gc_temp = GlobalConstants::new(1, target_regions);
            let regions = Regions {
                regions: IndexMap::with_hasher(FxBuildHasher),
                target_regions,
                population_size: 0,
            };
            return (regions, gc_temp);
        }
        let global_constants = GlobalConstants::new(population_size, target_regions);
        let regions = Regions::new(&global_constants);
        (regions, global_constants)
    }

    fn create_organism_with_score_and_key(score: Option<f64>, key: Option<Vec<usize>>) -> Organism {
        let phenotype = Phenotype::new_for_test(vec![0.0; 7]);
        {
            let organism = Organism::new(Rc::new(phenotype), 0);
            organism.set_score(score);
            organism.set_region_key(key);
            organism
        }
    }

    #[test]
    fn given_no_organisms_when_update_min_scores_then_scores_are_unchanged() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct
            .regions
            .get_mut(&key1)
            .unwrap()
            .set_min_score(Some(10.0));

        let organisms = Organisms::new_from_organisms(vec![]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(10.0));
    }

    #[test]
    fn given_organisms_no_scores_or_no_keys_when_update_min_scores_then_scores_are_unchanged() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct
            .regions
            .get_mut(&key1)
            .unwrap()
            .set_min_score(Some(10.0));

        let organism1 = create_organism_with_score_and_key(None, Some(key1.clone()));
        let organism2 = create_organism_with_score_and_key(Some(5.0), None);

        let organisms = Organisms::new_from_organisms(vec![organism1, organism2]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(10.0));
    }

    #[test]
    fn given_organisms_with_zero_or_negative_scores_when_update_min_scores_then_scores_are_unchanged()
     {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct
            .regions
            .get_mut(&key1)
            .unwrap()
            .set_min_score(Some(10.0));

        let organism1 = create_organism_with_score_and_key(Some(0.0), Some(key1.clone()));
        let organism2 = create_organism_with_score_and_key(Some(-5.0), Some(key1.clone()));

        let organisms = Organisms::new_from_organisms(vec![organism1, organism2]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(10.0));
    }

    #[test]
    fn given_single_organism_with_positive_score_in_empty_region_when_update_min_scores_then_score_set()
     {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());

        let organism1 = create_organism_with_score_and_key(Some(5.0), Some(key1.clone()));

        let organisms = Organisms::new_from_organisms(vec![organism1]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(5.0));
    }

    #[test]
    fn given_multiple_organisms_same_region_when_update_min_scores_then_min_positive_score_set() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());

        let organism1 = create_organism_with_score_and_key(Some(10.0), Some(key1.clone()));
        let organism2 = create_organism_with_score_and_key(Some(5.0), Some(key1.clone()));
        let organism3 = create_organism_with_score_and_key(Some(15.0), Some(key1.clone()));
        let organism4 = create_organism_with_score_and_key(Some(0.0), Some(key1.clone()));

        let organisms =
            Organisms::new_from_organisms(vec![organism1, organism2, organism3, organism4]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(5.0));
    }

    #[test]
    fn given_organisms_different_regions_when_update_min_scores_then_scores_set_correctly() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        let key2 = vec![2];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct.regions.insert(key2.clone(), Region::new());

        let organism1 = create_organism_with_score_and_key(Some(10.0), Some(key1.clone()));
        let organism2 = create_organism_with_score_and_key(Some(5.0), Some(key1.clone()));
        let organism3 = create_organism_with_score_and_key(Some(20.0), Some(key2.clone()));
        let organism4 = create_organism_with_score_and_key(Some(25.0), Some(key2.clone()));

        let organisms =
            Organisms::new_from_organisms(vec![organism1, organism2, organism3, organism4]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(5.0));
        assert_eq!(regions_struct.regions[&key2].min_score(), Some(20.0));
    }

    #[test]
    fn given_new_score_is_higher_when_update_min_scores_then_score_is_unchanged() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct
            .regions
            .get_mut(&key1)
            .unwrap()
            .set_min_score(Some(2.0));

        let organism1 = create_organism_with_score_and_key(Some(10.0), Some(key1.clone()));

        let organisms = Organisms::new_from_organisms(vec![organism1]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(2.0));
    }

    #[test]
    fn given_new_score_is_lower_when_update_min_scores_then_score_is_updated() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct
            .regions
            .get_mut(&key1)
            .unwrap()
            .set_min_score(Some(10.0));

        let organism1 = create_organism_with_score_and_key(Some(5.0), Some(key1.clone()));

        let organisms = Organisms::new_from_organisms(vec![organism1]);
        regions_struct.update_all_region_min_scores(&organisms);

        assert_eq!(regions_struct.regions[&key1].min_score(), Some(5.0));
    }
}
