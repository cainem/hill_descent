use crate::world::organisms::Organisms;
use crate::world::regions::Regions;
use std::collections::BTreeMap;

impl Regions {
    /// Updates the minimum score for all regions based on the scores of the organisms within them.
    ///
    /// This function iterates through all organisms and identifies the minimum positive score
    /// for each region. Regions with no organisms or no organisms with positive scores
    /// will have their `_min_score` set to `None`.
    pub(super) fn update_all_region_min_scores(&mut self, all_organisms: &Organisms) {
        // Reset all current min scores
        for region in self.regions.values_mut() {
            region.set_min_score(None);
        }

        // Temporary map to find the true minimum score for each region key
        let mut min_scores_for_keys: BTreeMap<Vec<usize>, f64> = BTreeMap::new();

        for organism in all_organisms.iter() {
            if let (Some(key), Some(score)) = (organism.region_key(), organism.score()) {
                // Only consider positive scores as per PDD (fitness includes e0)
                if score > 0.0 {
                    min_scores_for_keys
                        .entry(key.clone())
                        .and_modify(|current_min| {
                            if score < *current_min {
                                *current_min = score;
                            }
                        })
                        .or_insert(score);
                }
            }
        }

        // Apply the found minimum scores to the actual regions
        for (key, min_score) in min_scores_for_keys {
            if let Some(region) = self.regions.get_mut(&key) {
                region.set_min_score(Some(min_score));
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
    use std::collections::BTreeMap;
    use std::rc::Rc;

    fn default_system_parameters() -> Vec<f64> {
        vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
    }

    fn create_test_regions_and_gc(
        max_regions: usize,
        population_size: usize,
    ) -> (Regions, GlobalConstants) {
        if population_size == 0 {
            let gc_temp = GlobalConstants::new(1, max_regions);
            let regions = Regions {
                regions: BTreeMap::new(),
                max_regions,
                population_size: 0,
            };
            return (regions, gc_temp);
        }
        let global_constants = GlobalConstants::new(population_size, max_regions);
        let regions = Regions::new(&global_constants);
        (regions, global_constants)
    }

    fn create_organism_with_score_and_key(score: Option<f64>, key: Option<Vec<usize>>) -> Organism {
        let phenotype = Phenotype::new_for_test(default_system_parameters());
        let mut organism = Organism::new(Rc::new(phenotype));
        organism.set_score(score);
        organism.set_region_key(key);
        organism
    }

    #[test]
    fn given_no_organisms_when_update_min_scores_then_no_scores_set() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let organisms = Organisms::new_from_organisms(Vec::new());

        regions_struct.update_all_region_min_scores(&organisms);

        for region in regions_struct.regions.values() {
            assert_eq!(region.min_score(), None);
        }
    }

    #[test]
    fn given_organisms_no_scores_or_no_keys_when_update_min_scores_then_no_scores_set_in_regions() {
        let (mut regions_struct, _gc_) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());

        let organisms_vec = vec![
            create_organism_with_score_and_key(None, Some(key1.clone())),
            create_organism_with_score_and_key(Some(10.0), None),
        ];
        let organisms = Organisms::new_from_organisms(organisms_vec);

        regions_struct.update_all_region_min_scores(&organisms);
        assert_eq!(regions_struct.regions.get(&key1).unwrap().min_score(), None);
    }

    #[test]
    fn given_organisms_with_zero_or_negative_scores_when_update_min_scores_then_ignored() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());

        let organisms_vec = vec![
            create_organism_with_score_and_key(Some(0.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(-5.0), Some(key1.clone())),
        ];
        let organisms = Organisms::new_from_organisms(organisms_vec);
        regions_struct.update_all_region_min_scores(&organisms);
        assert_eq!(regions_struct.regions.get(&key1).unwrap().min_score(), None);
    }

    #[test]
    fn given_single_organism_with_positive_score_when_update_min_scores_then_score_set() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());

        let organisms_vec = vec![create_organism_with_score_and_key(
            Some(5.5),
            Some(key1.clone()),
        )];
        let organisms = Organisms::new_from_organisms(organisms_vec);
        regions_struct.update_all_region_min_scores(&organisms);
        assert_eq!(
            regions_struct.regions.get(&key1).unwrap().min_score(),
            Some(5.5)
        );
    }

    #[test]
    fn given_multiple_organisms_same_region_when_update_min_scores_then_min_positive_score_set() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.regions.insert(key1.clone(), Region::new());

        let organisms_vec = vec![
            create_organism_with_score_and_key(Some(10.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(5.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(0.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(20.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(-2.0), Some(key1.clone())),
        ];
        let organisms = Organisms::new_from_organisms(organisms_vec);
        regions_struct.update_all_region_min_scores(&organisms);
        assert_eq!(
            regions_struct.regions.get(&key1).unwrap().min_score(),
            Some(5.0)
        );
    }

    #[test]
    fn given_organisms_different_regions_when_update_min_scores_then_scores_set_correctly() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        let key2 = vec![2];
        regions_struct.regions.insert(key1.clone(), Region::new());
        regions_struct.regions.insert(key2.clone(), Region::new());

        let organisms_vec = vec![
            create_organism_with_score_and_key(Some(10.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(5.0), Some(key1.clone())),
            create_organism_with_score_and_key(Some(100.0), Some(key2.clone())),
            create_organism_with_score_and_key(Some(50.0), Some(key2.clone())),
        ];
        let organisms = Organisms::new_from_organisms(organisms_vec);
        regions_struct.update_all_region_min_scores(&organisms);
        assert_eq!(
            regions_struct.regions.get(&key1).unwrap().min_score(),
            Some(5.0)
        );
        assert_eq!(
            regions_struct.regions.get(&key2).unwrap().min_score(),
            Some(50.0)
        );
    }

    #[test]
    fn given_region_not_in_organisms_when_update_min_scores_then_min_score_is_none() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key_populated = vec![1];
        let key_unpopulated = vec![2];
        regions_struct
            .regions
            .insert(key_populated.clone(), Region::new());
        regions_struct
            .regions
            .insert(key_unpopulated.clone(), Region::new());

        let organisms_vec = vec![create_organism_with_score_and_key(
            Some(5.5),
            Some(key_populated.clone()),
        )];
        let organisms = Organisms::new_from_organisms(organisms_vec);
        regions_struct.update_all_region_min_scores(&organisms);
        assert_eq!(
            regions_struct
                .regions
                .get(&key_populated)
                .unwrap()
                .min_score(),
            Some(5.5)
        );
        assert_eq!(
            regions_struct
                .regions
                .get(&key_unpopulated)
                .unwrap()
                .min_score(),
            None
        );
    }
}
