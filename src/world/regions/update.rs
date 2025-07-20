use crate::world::organisms::organism::update_region_key::OrganismUpdateRegionKeyResult;
use crate::world::{dimensions::Dimensions, organisms::Organisms};

// update_carrying_capacities function has been moved to its own file.

impl super::Regions {
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, organisms, dimensions))
    )]
    pub fn update(&mut self, organisms: &mut Organisms, dimensions: &mut Dimensions) {
        let mut changed_dimension: Option<usize> = None;

        loop {
            if let OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index) =
                organisms.update_all_region_keys(dimensions, changed_dimension)
            {
                self.handle_out_of_bounds(dimensions, dimension_index);
                continue;
            }

            changed_dimension = self.handle_successful_update(organisms, dimensions);
            if changed_dimension.is_none() {
                break;
            }
        }
        // Update min scores for regions first, then carrying capacities
        self.update_all_region_min_scores(organisms);
        self.update_carrying_capacities();
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::parameters::global_constants::GlobalConstants;
//     use crate::phenotype::Phenotype;
//     use crate::world::dimensions::Dimensions;
//     use crate::world::organisms::Organisms;
//     use crate::world::regions::Regions;
//     use std::collections::BTreeMap;
//     use std::ops::RangeInclusive;
//     // Note: NUM_SYSTEM_PARAMETERS is not directly used here as default_system_parameters provides a concrete Vec.
//     // It's defined in src/lib.rs and its value (7) is consistent with Phenotype requirements.

//     // HELPER FUNCTIONS

//     fn default_system_parameters() -> Vec<f64> {
//         // These values correspond to the initial system parameters mentioned in PDD/memories.
//         // Their count (7) matches NUM_SYSTEM_PARAMETERS.
//         vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
//     }

//     fn create_phenotype_with_problem_values(problem_values: &[f64]) -> Phenotype {
//         let mut expressed_values = default_system_parameters();
//         expressed_values.extend_from_slice(problem_values);
//         Phenotype::new_for_test(expressed_values)
//     }

//     fn create_test_organisms_from_problem_values(all_problem_values: Vec<Vec<f64>>) -> Organisms {
//         let phenotypes: Vec<Phenotype> = all_problem_values
//             .into_iter()
//             .map(|pv| create_phenotype_with_problem_values(&pv))
//             .collect();
//         Organisms::new_from_phenotypes(phenotypes)
//     }

//     fn create_test_organisms_single(p_values: &[f64]) -> Organisms {
//         create_test_organisms_from_problem_values(vec![p_values.to_vec()])
//     }

//     fn create_test_regions_and_gc(
//         target_regions: usize,
//         population_size: usize,
//     ) -> (Regions, GlobalConstants) {
//         if population_size == 0 {
//             // Special handling for tests that might want to check behavior with population_size 0
//             // The Regions::new constructor will panic if not for tests, so we bypass it here for such specific test cases.
//             let gc_temp = GlobalConstants::new(1, target_regions); // Dummy pop size > 0 for GC
//             let regions = Regions {
//                 regions: BTreeMap::new(),
//                 target_regions,
//                 population_size: 0, // Override for test
//             };
//             return (regions, gc_temp);
//         }

//         let global_constants = GlobalConstants::new(population_size, target_regions);
//         let regions = Regions::new(&global_constants);
//         (regions, global_constants)
//     }

//     fn create_test_dimensions(
//         problem_bounds: Vec<RangeInclusive<f64>>,
//         gc: &GlobalConstants,
//     ) -> Dimensions {
//         Dimensions::new(&problem_bounds, gc)
//     }

//     // TESTS

//     #[test]
//     fn given_no_organisms_when_update_then_completes_with_no_regions() {
//         let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
//         let mut organisms = create_test_organisms_from_problem_values(vec![]);
//         let mut dimensions = create_test_dimensions(vec![0.0..=1.0], &_gc);

//         regions.update(&mut organisms, &mut dimensions);

//         assert!(regions.regions.is_empty(), "Regions map should be empty");
//         assert_eq!(organisms.distinct_locations_count(), 0);
//     }

//     #[test]
//     fn given_one_organism_fits_initial_dimensions_when_update_then_completes_with_one_region() {
//         let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
//         let mut organisms = create_test_organisms_single(&[0.5, 0.5]); // 2 problem dimensions
//         let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

//         // Dimensions::new with target_regions = 4 and 2 dims divides each dim once.
//         assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 1);
//         assert_eq!(dimensions.get_dimension(1).number_of_divisions(), 1);
//         assert_eq!(dimensions.get_total_possible_regions(), 4);

//         regions.update(&mut organisms, &mut dimensions);

//         assert_eq!(regions.regions.len(), 1, "Should be one populated region");
//         assert_eq!(organisms.distinct_locations_count(), 1);
//         // Assert that no *further* divisions occurred in `regions.update`
//         assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 1);
//         assert_eq!(dimensions.get_dimension(1).number_of_divisions(), 1);
//     }

//     #[test]
//     fn given_organism_out_of_bounds_when_update_then_dimension_expands_and_organism_is_regioned() {
//         let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
//         let mut organisms = create_test_organisms_single(&[1.5, 0.5]);
//         let initial_bounds_dim0 = 0.0..=1.0;
//         let initial_bounds_dim1 = 0.0..=1.0;
//         let mut dimensions = create_test_dimensions(
//             vec![initial_bounds_dim0.clone(), initial_bounds_dim1.clone()],
//             &_gc,
//         );

//         regions.update(&mut organisms, &mut dimensions);

//         assert_eq!(
//             regions.regions.len(),
//             1,
//             "Should be one populated region after expansion"
//         );
//         let expanded_range_dim0 = dimensions.get_dimension(0).range();
//         assert_eq!(*expanded_range_dim0.start(), -0.5);
//         assert_eq!(*expanded_range_dim0.end(), 1.5);
//         assert_eq!(*dimensions.get_dimension(1).range(), initial_bounds_dim1);
//     }

//     #[test]
//     fn given_two_organisms_in_different_locations_need_division_when_update_then_dimensions_divide()
//     {
//         let max_r = 16;
//         let (mut regions, _gc) = create_test_regions_and_gc(max_r, 10);
//         let mut organisms = create_test_organisms_from_problem_values(vec![
//             vec![0.2, 0.2], // Org1
//             vec![0.8, 0.8], // Org2
//         ]);

//         let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

//         regions.update(&mut organisms, &mut dimensions);

//         assert_eq!(regions.regions.len(), 2, "Should be two populated regions");
//         assert_eq!(organisms.distinct_locations_count(), 2);
//         // Dimensions::new with max_r = 16 and 2 dims results in 2 divisions per dim.
//         assert_eq!(dimensions.get_dimension(0).number_of_divisions(), 2);
//         assert_eq!(dimensions.get_dimension(1).number_of_divisions(), 2);
//         assert_eq!(dimensions.get_total_possible_regions(), 16);
//     }

//     #[test]
//     fn given_organisms_requiring_division_until_populated_regions_meet_max_when_update_then_stops()
//     {
//         let max_r = 4;
//         let (mut regions, _gc) = create_test_regions_and_gc(max_r, 10);
//         let mut organisms = create_test_organisms_from_problem_values(vec![
//             vec![0.1],
//             vec![0.3],
//             vec![0.5],
//             vec![0.7],
//             vec![0.9],
//         ]);
//         let mut dimensions = create_test_dimensions(vec![0.0..=1.0], &_gc);

//         regions.update(&mut organisms, &mut dimensions);

//         // After the first pass only three populated regions exist, so the algorithm should
//         // perform at least one more subdivision resulting in at least three divisions.
//         assert!(
//             dimensions.get_dimension(0).number_of_divisions() >= 3,
//             "There should be at least 3 divisions after further subdivision"
//         );
//         // The populated region count should now meet or exceed `target_regions`.
//         assert!(
//             regions.regions.len() >= max_r,
//             "Subdivision should continue until populated regions reach or exceed target_regions"
//         );
//     }

//     #[test]
//     fn given_organisms_all_at_same_location_when_update_then_completes_with_one_region_no_division()
//     {
//         let (mut regions, _gc) = create_test_regions_and_gc(16, 10);
//         let mut organisms = create_test_organisms_from_problem_values(vec![
//             vec![0.5, 0.5],
//             vec![0.5, 0.5],
//             vec![0.5, 0.5],
//         ]);
//         let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

//         regions.update(&mut organisms, &mut dimensions);

//         assert_eq!(regions.regions.len(), 1, "Should be one populated region");
//         assert_eq!(organisms.distinct_locations_count(), 1);
//         // Dimensions::new with max_r=16 pre-divides the space.
//         assert_eq!(
//             dimensions.get_dimension(0).number_of_divisions(),
//             2,
//             "No *additional* division should occur"
//         );
//         assert_eq!(
//             dimensions.get_dimension(1).number_of_divisions(),
//             2,
//             "No *additional* division should occur"
//         );
//     }

//     #[test]
//     fn given_clustered_organisms_when_update_then_divides_further_until_populated_regions_meet_max()
//     {
//         let max_r = 8;
//         let (mut regions, _gc) = create_test_regions_and_gc(max_r, 20);
//         // All 10 organisms are in the bottom-left quadrant of a 1.0x1.0 space
//         let mut organisms = create_test_organisms_from_problem_values(vec![
//             vec![0.1, 0.1],
//             vec![0.1, 0.2],
//             vec![0.2, 0.1],
//             vec![0.2, 0.2],
//             vec![0.3, 0.3],
//             vec![0.3, 0.4],
//             vec![0.4, 0.3],
//             vec![0.4, 0.4],
//             vec![0.15, 0.25],
//             vec![0.35, 0.15],
//         ]);
//         let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &_gc);

//         // The algorithm should now continue to subdivide the already–populated bottom-left
//         // quadrant until the number of populated regions meets or exceeds `target_regions`.

//         regions.update(&mut organisms, &mut dimensions);

//         assert!(
//             regions.regions.len() >= max_r,
//             "Subdivision should continue until populated regions reach or exceed target_regions"
//         );
//     }

//     #[test]
//     fn given_no_problem_dimensions_when_update_then_completes_with_one_region_for_empty_key() {
//         let (mut regions, _gc) = create_test_regions_and_gc(4, 10);
//         let mut organisms_collection = create_test_organisms_from_problem_values(vec![vec![]]);
//         // Manually set a score for the organism to test carrying capacity calculation
//         organisms_collection
//             .iter()
//             .next()
//             .unwrap()
//             .set_score(Some(10.0));

//         let mut dimensions = create_test_dimensions(vec![], &_gc);

//         regions.update(&mut organisms_collection, &mut dimensions);

//         assert_eq!(
//             regions.regions.len(),
//             1,
//             "Should be one region for the empty key"
//         );
//         let region_key = Vec::<usize>::new();
//         assert!(
//             regions.regions.contains_key(&region_key),
//             "Region key should be empty vec"
//         );
//         let region = regions.regions.get(&region_key).unwrap();
//         assert_eq!(region.min_score(), Some(10.0));
//         assert_eq!(region.carrying_capacity(), Some(10)); // P=10, F1=10, Sum(1/F)=1/10 -> 10 * (1/10)/(1/10) = 10
//     }
// } // Closing brace for mod tests
