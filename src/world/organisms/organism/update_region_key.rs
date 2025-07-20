use crate::world::dimensions::{
    CalculateDimensionsKeyResult, Dimensions, calculate_dimensions_key,
};
use crate::world::organisms::organism::Organism;

pub enum OrganismUpdateRegionKeyResult {
    Success,
    OutOfBounds(usize), // Index of the dimension that caused the error
}

impl Organism {
    /// Updates the `region_key` of the organism based on its phenotype's expressed values
    /// (excluding system parameters) and the provided dimensions.
    ///
    /// # Arguments
    /// * `dimensions_container`: A reference to the `Dimensions` struct containing the
    ///   definitions for each dimension of the problem space.
    ///
    /// # Returns
    /// * `OrganismUpdateRegionKeyResult::Success` if the region key was successfully calculated and updated.
    /// * `OrganismUpdateRegionKeyResult::OutOfBounds(usize)` if a value was out of bounds for a dimension, returning the
    ///   index of the failing dimension.
    ///
    /// # Panics
    /// * Panics if the number of problem-specific expressed values from the phenotype
    ///   does not match the number of dimensions defined in `dimensions_container`,
    ///   unless both are zero. This panic originates from `calculate_dimensions_key`.
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "trace", skip(self, dimensions_container))
    )]
    pub fn update_region_key(
        &self,
        dimensions_container: &Dimensions,
        dimension_changed: Option<usize>,
    ) -> OrganismUpdateRegionKeyResult {
        if dimension_changed.is_none() || self.region_key().is_none() {
            let problem_expressed_values = self.phenotype().expression_problem_values();
            let actual_dimensions = dimensions_container.get_dimensions();

            match calculate_dimensions_key(actual_dimensions, problem_expressed_values) {
                CalculateDimensionsKeyResult::Success(key) => {
                    self.set_region_key(Some(key));
                    OrganismUpdateRegionKeyResult::Success
                }
                CalculateDimensionsKeyResult::Failure {
                    dimension_index, ..
                } => {
                    self.set_region_key(None);
                    OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index)
                }
            }
        } else {
            // TODO evaluate the key value for the one dimension that has changed and update the region key
            // if the region key is None at this point panic.

            // TODO return success or failure depending if it was possible or not
            OrganismUpdateRegionKeyResult::Success
        }
    }
}

// #[cfg(test)]
// mod tests {
// use super::*;
// use crate::NUM_SYSTEM_PARAMETERS;
// use crate::phenotype::Phenotype;
// use crate::world::dimensions::dimension::Dimension;
// use std::ops::RangeInclusive;
// use std::rc::Rc;

// // Helper function to create an Organism instance for testing purposes.
// fn create_organism_for_test(expressed_values: Vec<f64>) -> Organism {
//     // Ensure there are enough values for system parameters for Phenotype::new_for_test.
//     if expressed_values.len() < NUM_SYSTEM_PARAMETERS {
//         panic!(
//             "Test setup error: expressed_values length {} is less than NUM_SYSTEM_PARAMETERS {}",
//             expressed_values.len(),
//             NUM_SYSTEM_PARAMETERS
//         );
//     }

//     let phenotype = Phenotype::new_for_test(expressed_values);
//     let organism = Organism::new(Rc::new(phenotype), 0);
//     organism.set_region_key(None); // Ensure it starts as None for tests
//     organism
// }

// // Helper to create a Dimensions struct for testing.
// // bounds_divisions: Vec of (range, num_divisions_for_this_dimension)
// fn create_test_dimensions(bounds_divisions: Vec<(RangeInclusive<f64>, usize)>) -> Dimensions {
//     let dims: Vec<Dimension> = bounds_divisions
//         .into_iter()
//         .map(|(bounds, num_divisions)| {
//             let mut dim = Dimension::new(bounds, 0); // Start with 0 divisions
//             dim.set_number_of_divisions(num_divisions);
//             dim
//         })
//         .collect();

//     Dimensions::new_for_test(dims)
// }

// #[test]
// fn given_valid_inputs_when_update_region_key_then_success_and_key_updated() {
//     let organism = create_organism_for_test(vec![
//         0.1, 0.5, 0.001, 0.001, 0.001, 100.0,
//         2.0, // System params (NUM_SYSTEM_PARAMETERS)
//         7.5, 60.0, // Problem params
//     ]);
//     let dimensions = create_test_dimensions(vec![
//         (0.0..=10.0, 2),  // Dimension for 7.5. Index 1
//         (0.0..=100.0, 4), // Dimension for 60.0. Index 2
//     ]);

//     let result = organism.update_region_key(&dimensions, None);

//     assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
//     assert_eq!(organism.region_key(), Some(vec![2, 3]));
// }

// #[test]
// fn given_value_out_of_bounds_when_update_region_key_then_failure_and_key_is_none() {
//     let organism = create_organism_for_test(vec![
//         0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params
//         12.0, 50.0, // Problem params. 12.0 is out of bounds for the first dimension.
//     ]);
//     let dimensions = create_test_dimensions(vec![(0.0..=10.0, 2), (0.0..=100.0, 4)]);

//     let result = organism.update_region_key(&dimensions, None);

//     assert!(matches!(
//         result,
//         OrganismUpdateRegionKeyResult::OutOfBounds(0)
//     )); // Fails on the first dimension (index 0)
//     assert!(organism.region_key().is_none());
// }

// #[test]
// fn given_value_on_upper_bound_when_update_region_key_then_success_and_key_is_last_index() {
//     let organism = create_organism_for_test(vec![
//         0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0,  // System params
//         10.0, // Problem param on the boundary
//     ]);
//     let dimensions = create_test_dimensions(vec![(0.0..=10.0, 2)]); // 2 divisions -> 3 intervals [0, 5), [5, 10), [10, 10]

//     let result = organism.update_region_key(&dimensions, None);

//     assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
//     assert_eq!(organism.region_key(), Some(vec![2])); // Should be in the last interval
// }

// #[test]
// fn given_no_problem_params_when_update_region_key_then_success_and_key_is_empty() {
//     let organism = create_organism_for_test(vec![
//         0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params only
//     ]);
//     let dimensions = create_test_dimensions(vec![]); // No dimensions

//     let result = organism.update_region_key(&dimensions, None);

//     assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
//     assert_eq!(organism.region_key(), Some(vec![]));
// }
// }
