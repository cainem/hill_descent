use crate::world::{dimensions::Dimensions, organisms::Organisms};

impl super::Regions {
    /// Handles the successful update of all organism region keys.
    ///
    /// This function repopulates the regions with the organisms, prunes any
    /// regions that are now empty, and then determines if the simulation
    /// should continue dividing dimensions or stop.
    ///
    /// # Returns
    ///
    /// Returns `true` if the simulation has reached a stable state and should
    /// stop, `false` otherwise.
    pub(super) fn handle_successful_update(
        &mut self,
        organisms: &mut Organisms,
        dimensions: &mut Dimensions,
    ) -> Option<usize> {
        // place the organisms in their appropriate regions
        self.refill(organisms);

        // current regions are greater than or equal to the allowed regions;
        // refill and return
        if self.regions.len() >= self.target_regions {
            return None;
        }

        // otherwise we have not got enough regions
        // we need to divide a dimension.
        // we need to work out what is the best dimension to divide based on the distribution within the most populous region.
        // we are essentially using the most populous regions as a sample for the whole population

        let most_populous_region_key = self.get_most_common_key();

        if let Some(most_diverse_dimension) =
            self.get_most_diverse_dimension(&most_populous_region_key)
        {
            // divide the most diverse dimension
            dimensions.divide_next_dimension(most_diverse_dimension);
            Some(most_diverse_dimension)
        } else {
            // get_most_diverse_dimension returns None if there is no variation in any dimensions
            // in this case no dimension divisions are necessary fill and return none
            self.refill(organisms);
            None
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::parameters::global_constants::GlobalConstants;
//     use crate::phenotype::Phenotype;
//     use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};
//     use std::ops::RangeInclusive;
// //     use crate::phenotype::Phenotype;
// //     use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};
// //     use std::ops::RangeInclusive;

//     fn default_system_parameters() -> Vec<f64> {
//         vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
//     }

//     fn phenotype_with_problem_values(problem_values: &[f64]) -> Phenotype {
//         let mut expressed = default_system_parameters();
//         expressed.extend_from_slice(problem_values);
//         Phenotype::new_for_test(expressed)
//     }

//     fn organisms_from_problem_values(values: Vec<Vec<f64>>) -> Organisms {
//         let phenotypes: Vec<Phenotype> = values
//             .into_iter()
//             .map(|pv| phenotype_with_problem_values(&pv))
//             .collect();
//         Organisms::new_from_phenotypes(phenotypes)
//     }

//     fn setup(target_regions: usize, bounds: Vec<RangeInclusive<f64>>) -> (Regions, Dimensions) {
//         let gc = GlobalConstants::new(100, target_regions);
//         let regions = Regions::new(&gc);
//         let dimensions = Dimensions::new(&bounds, &gc);
//         (regions, dimensions)
//     }

//     #[test]
//     fn given_target_regions_already_reached_when_handle_successful_update_then_returns_true() {
//         let (mut regions, mut dims) = setup(1, vec![0.0..=1.0]);
//         let mut organisms = organisms_from_problem_values(vec![vec![0.5]]);
//         let finished = regions.handle_successful_update(&mut organisms, &mut dims);
//         assert!(finished);
//     }

//     #[test]
//     fn given_all_organisms_same_location_when_handle_successful_update_then_returns_true() {
//         let (mut regions, mut dims) = setup(10, vec![0.0..=1.0, 0.0..=1.0]);
//         let mut organisms = organisms_from_problem_values(vec![vec![0.5, 0.5]; 3]);
//         let finished = regions.handle_successful_update(&mut organisms, &mut dims);
//         assert!(finished);
//     }

//     #[test]
//     fn given_each_location_has_own_region_when_handle_successful_update_then_returns_true() {
//         let (mut regions, mut dims) = setup(10, vec![0.0..=1.0, 0.0..=1.0]);
//         dims.divide_next_dimension();
//         dims.divide_next_dimension();
//         let mut organisms = organisms_from_problem_values(vec![vec![0.1, 0.1], vec![0.9, 0.9]]);
//         let _ = organisms.update_all_region_keys(&dims);
//         let finished = regions.handle_successful_update(&mut organisms, &mut dims);
//         assert!(finished);
//     }

//     #[test]
//     fn given_possible_to_divide_further_when_handle_successful_update_then_returns_false_and_clears_regions()
//      {
//         let (mut regions, mut dims) = setup(10, vec![0.0..=1.0]);
//         let mut organisms = organisms_from_problem_values(vec![vec![0.1], vec![0.9]]);
//         let finished = regions.handle_successful_update(&mut organisms, &mut dims);
//         assert!(!finished);
//         assert!(regions.regions().is_empty());
//     }

//     #[test]
//     fn given_no_dimensions_to_divide_when_handle_successful_update_then_returns_true() {
//         let (mut regions, mut dims) = setup(10, vec![]);
//         let mut organisms = organisms_from_problem_values(vec![vec![]]);
//         let finished = regions.handle_successful_update(&mut organisms, &mut dims);
//         assert!(finished);
//     }
// }
