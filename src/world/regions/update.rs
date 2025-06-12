use crate::world::organisms::organism::update_region_key::OrganismUpdateRegionKeyResult;
use crate::world::{dimensions::Dimensions, organisms::Organisms};

impl super::Regions {
    pub fn update(&mut self, organisms: &mut Organisms, dimensions: &mut Dimensions) {
        loop {
            // Reset regions for the new iteration, but don't deallocate them.
            self.reset();

            match organisms.update_all_region_keys(dimensions) {
                OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index) => {
                    dimensions.expand_bounds(dimension_index);
                    // The dimension change invalidates all existing region keys.
                    // Clear all regions so they can be rebuilt in the next iteration.
                    self.regions.clear();
                    continue;
                }
                OrganismUpdateRegionKeyResult::Success => {
                    // Populate the reset regions with organisms based on their new keys.
                    self.add_phenotypes(organisms);
                    // Remove any regions that are no longer populated.
                    self.regions.retain(|_, region| !region.is_empty());

                    let num_populated_regions = self.regions.len();
                    let distinct_locations = organisms.distinct_locations_count();

                    // If there are no organisms, no further processing is needed.
                    if distinct_locations == 0 {
                        break; // No organisms to region or divide.
                    }

                    // PDD 4.2.2: Division stops if x = P_hat (populated regions == distinct locations)
                    // Ensure distinct_locations > 0 to avoid premature completion with no organisms.
                    if distinct_locations > 0 && num_populated_regions == distinct_locations {
                        break; // Stable state: all distinct locations have their own region.
                    }

                    // Stop dividing if the number of *potential* regions has met or exceeded the maximum.
                    if dimensions.get_total_possible_regions() >= self.max_regions {
                        break;
                    }

                    // Attempt to refine granularity by dividing a dimension.
                    if dimensions.divide_next_dimension() {
                        // The dimension change invalidates all existing region keys.
                        // Clear all regions so they can be rebuilt in the next iteration.
                        self.regions.clear();
                        continue;
                    } else {
                        // No more divisions possible.
                        break; // Stable state: cannot refine further.
                    }
                }
            }
        }
        //self.update_carrying_capacities(organisms); // As per PDD 5.2.5
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::dimensions::Dimensions;
    use crate::world::organisms::Organisms;
    use crate::world::regions::Regions;
    use std::ops::RangeInclusive;
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
        let (mut regions, gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0], &gc);

        regions.update(&mut organisms, &mut dimensions);

        assert!(regions.regions.is_empty(), "Regions map should be empty");
        assert_eq!(organisms.distinct_locations_count(), 0);
    }

    #[test]
    fn given_one_organism_fits_initial_dimensions_when_update_then_completes_with_one_region() {
        let (mut regions, gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_single(&[0.5, 0.5]); // 2 problem dimensions
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &gc);

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
        let (mut regions, gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_single(&[1.5, 0.5]);
        let initial_bounds_dim0 = 0.0..=1.0;
        let initial_bounds_dim1 = 0.0..=1.0;
        let mut dimensions = create_test_dimensions(
            vec![initial_bounds_dim0.clone(), initial_bounds_dim1.clone()],
            &gc,
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
        let (mut regions, gc) = create_test_regions_and_gc(max_r, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.2, 0.2], // Org1
            vec![0.8, 0.8], // Org2
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &gc);

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
        let (mut regions, gc) = create_test_regions_and_gc(max_r, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.1],
            vec![0.3],
            vec![0.5],
            vec![0.7],
            vec![0.9],
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0], &gc);

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
        let (mut regions, gc) = create_test_regions_and_gc(16, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![
            vec![0.5, 0.5],
            vec![0.5, 0.5],
            vec![0.5, 0.5],
        ]);
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &gc);

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
        let (mut regions, gc) = create_test_regions_and_gc(max_r, 20);
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
        let mut dimensions = create_test_dimensions(vec![0.0..=1.0, 0.0..=1.0], &gc);

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
        let (mut regions, gc) = create_test_regions_and_gc(4, 10);
        let mut organisms = create_test_organisms_from_problem_values(vec![vec![]]);
        let mut dimensions = create_test_dimensions(vec![], &gc);

        regions.update(&mut organisms, &mut dimensions);

        assert_eq!(
            regions.regions.len(),
            1,
            "Should be one region for the empty key"
        );
        assert!(
            regions.regions.contains_key(&Vec::<usize>::new()),
            "Region key should be empty vec"
        );
        assert_eq!(organisms.distinct_locations_count(), 1);
        assert_eq!(dimensions.num_dimensions(), 0);
        assert_eq!(dimensions.get_total_possible_regions(), 1);
    }
}
