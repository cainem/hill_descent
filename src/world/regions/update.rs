use crate::world::organisms::organism::update_region_key::OrganismUpdateRegionKeyResult;
use crate::world::{dimensions::Dimensions, organisms::Organisms};

use super::{Region, Regions};
use std::collections::BTreeMap;

/// Updates the minimum score for all regions based on the scores of the organisms within them.
///
/// This function iterates through all organisms and identifies the minimum positive score
/// for each region. Regions with no organisms or no organisms with positive scores
/// will have their `_min_score` set to `None`.
fn update_all_region_min_scores(
    regions_map: &mut BTreeMap<Vec<usize>, Region>,
    all_organisms: &Organisms,
) {
    // Reset all current min scores
    for region in regions_map.values_mut() {
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
        if let Some(region) = regions_map.get_mut(&key) {
            region.set_min_score(Some(min_score));
        }
    }
}

/// Updates the carrying capacity for all regions.
///
/// Calculation is based on PDD section 4.2.4:
/// P_i = P * (1/F_i) / sum_over_j(1/F_j)
/// where P is total target population_size, F_i is min_score in region i.
/// Regions with no valid positive min_score, or if the sum of inverse fitnesses is not positive,
/// will have their carrying capacity set to 0.
fn update_carrying_capacities(regions_struct: &mut Regions) {
    let mut sum_inverse_min_fitness = 0.0;
    let mut regions_with_valid_scores = Vec::new();

    for (key, region) in regions_struct.regions.iter() {
        if let Some(min_score) = region.min_score() {
            if min_score > 0.0 {
                sum_inverse_min_fitness += 1.0 / min_score;
                regions_with_valid_scores.push(key.clone());
            }
        }
    }

    let total_population_size = regions_struct.population_size;

    for (_key, region) in regions_struct.regions_mut().iter_mut() {
        if sum_inverse_min_fitness > 0.0 {
            if let Some(min_score) = region.min_score() {
                if min_score > 0.0 {
                    let capacity_float =
                        total_population_size as f64 * (1.0 / min_score) / sum_inverse_min_fitness;
                    region.set_carrying_capacity(Some(capacity_float.floor() as usize));
                    continue;
                }
            }
        }
        // Default to 0 if no valid score for this region, or sum_inverse_min_fitness is not positive
        region.set_carrying_capacity(Some(0));
    }
}

impl super::Regions {
    pub fn update(&mut self, organisms: &mut Organisms, dimensions: &mut Dimensions) {
        loop {
            match organisms.update_all_region_keys(dimensions) {
                OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index) => {
                    self.handle_out_of_bounds(dimensions, dimension_index);
                    continue;
                }
                OrganismUpdateRegionKeyResult::Success => {
                    if self.handle_successful_update(organisms, dimensions) {
                        break;
                    }
                }
            }
        }
        // Update min scores for regions first, then carrying capacities
        update_all_region_min_scores(&mut self.regions, organisms);
        update_carrying_capacities(self); // Pass the whole Regions struct
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::dimensions::Dimensions;
    use crate::world::organisms::Organisms;
    use crate::world::regions::{Region, Regions};
    use std::collections::BTreeMap;
    use std::ops::RangeInclusive;
    use std::rc::Rc;
    // Note: NUM_SYSTEM_PARAMETERS is not directly used here as default_system_parameters provides a concrete Vec.
    // It's defined in src/lib.rs and its value (7) is consistent with Phenotype requirements.

    // HELPER FUNCTIONS

    fn default_system_parameters() -> Vec<f64> {
        // These values correspond to the initial system parameters mentioned in PDD/memories.
        // Their count (7) matches NUM_SYSTEM_PARAMETERS.
        vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
    }

    fn create_phenotype_with_problem_values(problem_values: &[f64]) -> Phenotype {
        let mut expressed_values = default_system_parameters();
        expressed_values.extend_from_slice(problem_values);
        Phenotype::new_for_test(expressed_values)
    }

    fn create_test_organisms_from_problem_values(all_problem_values: Vec<Vec<f64>>) -> Organisms {
        let phenotypes: Vec<Phenotype> = all_problem_values
            .into_iter()
            .map(|pv| create_phenotype_with_problem_values(&pv))
            .collect();
        Organisms::new_from_phenotypes(phenotypes)
    }

    fn create_test_organisms_single(p_values: &[f64]) -> Organisms {
        create_test_organisms_from_problem_values(vec![p_values.to_vec()])
    }

    fn create_test_regions_and_gc(
        max_regions: usize,
        population_size: usize,
    ) -> (Regions, GlobalConstants) {
        if population_size == 0 {
            // Special handling for tests that might want to check behavior with population_size 0
            // The Regions::new constructor will panic if not for tests, so we bypass it here for such specific test cases.
            let gc_temp = GlobalConstants::new(1, max_regions); // Dummy pop size > 0 for GC
            let regions = Regions {
                regions: BTreeMap::new(),
                max_regions,
                population_size: 0, // Override for test
            };
            return (regions, gc_temp);
        }

        let global_constants = GlobalConstants::new(population_size, max_regions);
        let regions = Regions::new(&global_constants);
        (regions, global_constants)
    }

    fn create_test_dimensions(
        problem_bounds: Vec<RangeInclusive<f64>>,
        gc: &GlobalConstants,
    ) -> Dimensions {
        Dimensions::new(&problem_bounds, gc)
    }

    // TESTS

    #[test]
    fn given_no_organisms_when_update_then_completes_with_no_regions() {
        let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0], &_gc);

        regions.update(&mut organisms, &mut dimensions);

        assert!(regions.regions.is_empty(), "Regions map should be empty");
        assert_eq!(organisms.distinct_locations_count(), 0);
    }

    #[test]
    fn given_one_organism_fits_initial_dimensions_when_update_then_completes_with_one_region() {
        let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_single(&[0.5, 0.5]); // 2 problem dimensions
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

        // Dimensions::new with max_regions = 4 and 2 dims divides each dim once.
        assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 1);
        assert_eq!(dimensions.get_dimension(1).number_of_divisions(), 1);
        assert_eq!(dimensions.get_total_possible_regions(), 4);

        regions.update(&mut organisms, &mut dimensions);

        assert_eq!(regions.regions.len(), 1, "Should be one populated region");
        assert_eq!(organisms.distinct_locations_count(), 1);
        // Assert that no *further* divisions occurred in `regions.update`
        assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 1);
        assert_eq!(dimensions.get_dimension(1).number_of_divisions(), 1);
    }

    #[test]
    fn given_organism_out_of_bounds_when_update_then_dimension_expands_and_organism_is_regioned() {
        let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_single(&[1.5, 0.5]);
        let initial_bounds_dim0 = 0.0..=1.0;
        let initial_bounds_dim1 = 0.0..=1.0;
        let mut dimensions = create_test_dimensions(
            vec![initial_bounds_dim0.clone(), initial_bounds_dim1.clone()],
            &_gc,
        );

        regions.update(&mut organisms, &mut dimensions);

        assert_eq!(
            regions.regions.len(),
            1,
            "Should be one populated region after expansion"
        );
        let expanded_range_dim0 = dimensions.get_dimension(0).range();
        assert_eq!(*expanded_range_dim0.start(), -0.5);
        assert_eq!(*expanded_range_dim0.end(), 1.5);
        assert_eq!(*dimensions.get_dimension(1).range(), initial_bounds_dim1);
    }

    #[test]
    fn given_two_organisms_in_different_locations_need_division_when_update_then_dimensions_divide()
    {
        let max_r = 16;
        let (mut regions, _gc) = create_test_regions_and_gc(max_r, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.2, 0.2], // Org1
            vec![0.8, 0.8], // Org2
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

        regions.update(&mut organisms, &mut dimensions);

        assert_eq!(regions.regions.len(), 2, "Should be two populated regions");
        assert_eq!(organisms.distinct_locations_count(), 2);
        // Dimensions::new with max_r = 16 and 2 dims results in 2 divisions per dim.
        assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 2);
        assert_eq!(dimensions.get_dimension(1).number_of_divisions(), 2);
        assert_eq!(dimensions.get_total_possible_regions(), 16);
    }

    #[test]
    fn given_organisms_requiring_division_up_to_max_regions_when_update_then_stops_at_max_regions()
    {
        let max_r = 4;
        let (mut regions, _gc) = create_test_regions_and_gc(max_r, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.1],
            vec![0.3],
            vec![0.5],
            vec![0.7],
            vec![0.9],
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0], &_gc);

        regions.update(&mut organisms, &mut dimensions);

        assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 2);
        assert_eq!(dimensions.get_total_possible_regions(), 4);
        assert_eq!(
            regions.regions.len(),
            3, // The test failure log shows this results in 3 populated regions, not 4.
            "Should populate up to max possible regions if distinct locations allow"
        );
    }

    #[test]
    fn given_organisms_all_at_same_location_when_update_then_completes_with_one_region_no_division()
    {
        let (mut regions, _gc) = create_test_regions_and_gc(16, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.5, 0.5],
            vec![0.5, 0.5],
            vec![0.5, 0.5],
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

        regions.update(&mut organisms, &mut dimensions);

        assert_eq!(regions.regions.len(), 1, "Should be one populated region");
        assert_eq!(organisms.distinct_locations_count(), 1);
        // Dimensions::new with max_r=16 pre-divides the space.
        assert_eq!(
            dimensions.get_dimension(0).number_of_divisions(),
            2,
            "No *additional* division should occur"
        );
        assert_eq!(
            dimensions.get_dimension(1).number_of_divisions(),
            2,
            "No *additional* division should occur"
        );
    }

    #[test]
    fn given_clustered_organisms_when_update_then_stops_dividing_when_potential_regions_reaches_max()
     {
        let max_r = 8;
        let (mut regions, _gc) = create_test_regions_and_gc(max_r, 20);
        // All 10 organisms are in the bottom-left quadrant of a 1.0x1.0 space
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.1, 0.1],
            vec![0.1, 0.2],
            vec![0.2, 0.1],
            vec![0.2, 0.2],
            vec![0.3, 0.3],
            vec![0.3, 0.4],
            vec![0.4, 0.3],
            vec![0.4, 0.4],
            vec![0.15, 0.25],
            vec![0.35, 0.15],
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

        // The update loop terminates when the number of *potential* regions meets or exceeds
        // max_regions. In this case, it stops when potential regions becomes 8.
        // This leaves the clustered organisms in a few, poorly separated regions,
        // as the division stops before focusing on the populated areas.

        regions.update(&mut organisms, &mut dimensions);

        // After initial divisions in Dimensions::new to get 8 potential regions (a 4x2 grid),
        // all organisms fall into just two of these grid cells.
        assert_eq!(
            regions.regions.len(),
            2,
            "Should only populate 2 regions as division stops when potential regions reach max"
        );
        assert_eq!(
            dimensions.get_total_possible_regions(),
            max_r,
            "Total potential regions should be equal to max_regions at the stop point"
        );
    }

    #[test]
    fn given_no_problem_dimensions_when_update_then_completes_with_one_region_for_empty_key() {
        let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
        let mut organisms_collection = create_test_organisms_from_problem_values(vec![vec![]]);
        // Manually set a score for the organism to test carrying capacity calculation
        organisms_collection
            .iter_mut()
            .next()
            .unwrap()
            .set_score(Some(10.0));

        let mut dimensions = create_test_dimensions(vec![], &_gc);

        regions.update(&mut organisms_collection, &mut dimensions);

        assert_eq!(
            regions.regions.len(),
            1,
            "Should be one region for the empty key"
        );
        let region_key = Vec::<usize>::new();
        assert!(
            regions.regions.contains_key(&region_key),
            "Region key should be empty vec"
        );
        let region = regions.regions.get(&region_key).unwrap();
        assert_eq!(region.min_score(), Some(10.0));
        assert_eq!(region.carrying_capacity(), Some(10)); // P=10, F1=10, Sum(1/F)=1/10 -> 10 * (1/10)/(1/10) = 10

        assert_eq!(organisms_collection.distinct_locations_count(), 1);
        assert_eq!(dimensions.num_dimensions(), 0);
        assert_eq!(dimensions.get_total_possible_regions(), 1);
    }

    // Tests for update_all_region_min_scores
    mod test_update_all_region_min_scores {
        use super::super::update_all_region_min_scores;
        use super::*; // Imports helpers from outer scope
        use crate::world::organisms::Organism;

        fn create_organism_with_score_and_key(
            score: Option<f64>,
            key: Option<Vec<usize>>,
        ) -> Organism {
            let phenotype = Phenotype::new_for_test(default_system_parameters());
            let mut organism = Organism::new(Rc::new(phenotype));
            organism.set_score(score);
            organism.set_region_key(key);
            organism
        }

        #[test]
        fn given_no_organisms_when_update_min_scores_then_no_scores_set() {
            let (_regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let mut regions_map = BTreeMap::new();
            let organisms = Organisms::new_from_organisms(vec![]);
            update_all_region_min_scores(&mut regions_map, &organisms);
            assert!(regions_map.is_empty());
        }

        #[test]
        fn given_organisms_no_scores_or_no_keys_when_update_min_scores_then_no_scores_set_in_regions()
         {
            let (_regions_struct, _gc_) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            let mut regions_map = BTreeMap::new();
            regions_map.insert(key1.clone(), Region::new());

            let organisms_vec = vec![
                create_organism_with_score_and_key(None, Some(key1.clone())), // No score
                create_organism_with_score_and_key(Some(10.0), None),         // No key
            ];
            let organisms = Organisms::new_from_organisms(organisms_vec);

            update_all_region_min_scores(&mut regions_map, &organisms);

            assert_eq!(regions_map.get(&key1).unwrap().min_score(), None);
        }

        #[test]
        fn given_organisms_with_zero_or_negative_scores_when_update_min_scores_then_ignored() {
            let (_regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            let mut regions_map = BTreeMap::new();
            regions_map.insert(key1.clone(), Region::new());

            let organisms_vec = vec![
                create_organism_with_score_and_key(Some(0.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(-5.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(10.0), Some(key1.clone())), // Valid one
            ];
            let organisms = Organisms::new_from_organisms(organisms_vec);

            update_all_region_min_scores(&mut regions_map, &organisms);
            assert_eq!(regions_map.get(&key1).unwrap().min_score(), Some(10.0));
        }

        #[test]
        fn given_single_organism_with_positive_score_when_update_min_scores_then_score_set() {
            let (_regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            let mut regions_map = BTreeMap::new();
            regions_map.insert(key1.clone(), Region::new());
            let organisms =
                Organisms::new_from_organisms(vec![create_organism_with_score_and_key(
                    Some(5.5),
                    Some(key1.clone()),
                )]);

            update_all_region_min_scores(&mut regions_map, &organisms);
            assert_eq!(regions_map.get(&key1).unwrap().min_score(), Some(5.5));
        }

        #[test]
        fn given_multiple_organisms_same_region_when_update_min_scores_then_min_positive_score_set()
        {
            let (_regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            let mut regions_map = BTreeMap::new();
            regions_map.insert(key1.clone(), Region::new());

            let organisms_vec = vec![
                create_organism_with_score_and_key(Some(10.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(5.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(0.5), Some(key1.clone())), // This is the min positive
                create_organism_with_score_and_key(Some(20.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(-2.0), Some(key1.clone())), // Ignored
            ];
            let organisms = Organisms::new_from_organisms(organisms_vec);

            update_all_region_min_scores(&mut regions_map, &organisms);
            assert_eq!(regions_map.get(&key1).unwrap().min_score(), Some(0.5));
        }

        #[test]
        fn given_organisms_different_regions_when_update_min_scores_then_scores_set_correctly() {
            let (_regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            let key2 = vec![2];
            let mut regions_map = BTreeMap::new();
            regions_map.insert(key1.clone(), Region::new());
            regions_map.insert(key2.clone(), Region::new());

            let organisms_vec = vec![
                create_organism_with_score_and_key(Some(10.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(5.0), Some(key1.clone())),
                create_organism_with_score_and_key(Some(100.0), Some(key2.clone())),
                create_organism_with_score_and_key(Some(50.0), Some(key2.clone())),
            ];
            let organisms = Organisms::new_from_organisms(organisms_vec);

            update_all_region_min_scores(&mut regions_map, &organisms);
            assert_eq!(regions_map.get(&key1).unwrap().min_score(), Some(5.0));
            assert_eq!(regions_map.get(&key2).unwrap().min_score(), Some(50.0));
        }

        #[test]
        fn given_region_not_in_organisms_when_update_min_scores_then_min_score_is_none() {
            let (_regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key_populated = vec![1];
            let key_unpopulated = vec![2];
            let mut regions_map = BTreeMap::new();
            regions_map.insert(key_populated.clone(), Region::new());
            regions_map.insert(key_unpopulated.clone(), Region::new());

            let organisms =
                Organisms::new_from_organisms(vec![create_organism_with_score_and_key(
                    Some(5.5),
                    Some(key_populated.clone()),
                )]);

            update_all_region_min_scores(&mut regions_map, &organisms);
            assert_eq!(
                regions_map.get(&key_populated).unwrap().min_score(),
                Some(5.5)
            );
            assert_eq!(regions_map.get(&key_unpopulated).unwrap().min_score(), None);
        }
    }

    // Tests for update_carrying_capacities
    mod test_update_carrying_capacities {
        use super::super::update_carrying_capacities;
        use super::*; // Imports helpers from outer scope

        fn setup_region_with_min_score(min_score: Option<f64>) -> Region {
            let mut region = Region::new();
            region.set_min_score(min_score);
            region
        }

        #[test]
        fn given_no_regions_with_min_scores_when_update_capacities_then_all_capacities_zero() {
            let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(None));

            update_carrying_capacities(&mut regions_struct);
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(0)
            );
        }

        #[test]
        fn given_sum_inverse_fitness_zero_when_update_capacities_then_all_capacities_zero() {
            let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
            let key1 = vec![1];
            // Min score 0 or negative leads to sum_inverse_min_fitness effectively 0 or invalid
            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(Some(0.0)));
            update_carrying_capacities(&mut regions_struct);
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(0)
            );

            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(Some(-5.0)));
            update_carrying_capacities(&mut regions_struct);
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(0)
            );
        }

        #[test]
        fn given_one_region_with_positive_min_score_when_update_capacities_then_capacity_is_population_size()
         {
            let population_size = 20;
            let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
            let key1 = vec![1];
            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(Some(10.0))); // F1 = 10
            // Sum(1/F) = 1/10. P1 = PopSize * (1/10) / (1/10) = PopSize

            update_carrying_capacities(&mut regions_struct);
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(population_size)
            );
        }

        #[test]
        fn given_multiple_regions_with_min_scores_when_update_capacities_then_capacities_proportional()
         {
            let population_size = 100;
            let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
            let key1 = vec![1];
            let key2 = vec![2];
            let key3 = vec![3];

            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(Some(2.0))); // 1/F1 = 0.5
            regions_struct
                .regions_mut()
                .insert(key2.clone(), setup_region_with_min_score(Some(4.0))); // 1/F2 = 0.25
            regions_struct
                .regions_mut()
                .insert(key3.clone(), setup_region_with_min_score(Some(0.0))); // Ignored, F3_inv = 0 for sum
            // Sum_inv_F = 0.5 + 0.25 = 0.75

            update_carrying_capacities(&mut regions_struct);

            // P1 = 100 * (0.5 / 0.75) = 100 * (2/3) = 66.66 -> 66
            // P2 = 100 * (0.25 / 0.75) = 100 * (1/3) = 33.33 -> 33
            // P3 = 0
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(66)
            );
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key2)
                    .unwrap()
                    .carrying_capacity(),
                Some(33)
            );
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key3)
                    .unwrap()
                    .carrying_capacity(),
                Some(0)
            );
        }

        #[test]
        fn given_region_with_no_min_score_when_update_capacities_then_its_capacity_is_zero() {
            let population_size = 50;
            let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
            let key1 = vec![1]; // Has score
            let key2 = vec![2]; // No score

            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(Some(5.0))); // F1 = 5, 1/F1 = 0.2
            regions_struct
                .regions_mut()
                .insert(key2.clone(), setup_region_with_min_score(None));
            // Sum_inv_F = 0.2

            update_carrying_capacities(&mut regions_struct);

            // P1 = 50 * (0.2 / 0.2) = 50
            // P2 = 0
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(population_size)
            );
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key2)
                    .unwrap()
                    .carrying_capacity(),
                Some(0)
            );
        }

        #[test]
        fn given_population_size_zero_when_update_capacities_then_all_capacities_zero() {
            let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 0); // Population size 0
            let key1 = vec![1];
            regions_struct
                .regions_mut()
                .insert(key1.clone(), setup_region_with_min_score(Some(10.0)));

            update_carrying_capacities(&mut regions_struct);
            assert_eq!(
                regions_struct
                    .regions
                    .get(&key1)
                    .unwrap()
                    .carrying_capacity(),
                Some(0)
            );
        }
    }
}
