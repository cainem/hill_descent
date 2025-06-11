use crate::NUM_SYSTEM_PARAMETERS;
use crate::phenotype::Phenotype;
use crate::world::dimensions::Dimensions;
use crate::world::dimensions::calculate_dimensions_key::{
    CalculateDimensionsKeyResult, calculate_dimensions_key,
};

impl Phenotype {
    /// Updates the `dimensions_key` of the phenotype based on its expressed values
    /// (excluding system parameters) and the provided dimensions.
    ///
    /// # Arguments
    /// * `dimensions_container`: A reference to the `Dimensions` struct containing the
    ///   definitions for each dimension of the problem space.
    ///
    /// # Returns
    /// * `Ok(())` if the dimensions key was successfully calculated and updated.
    /// * `Err(usize)` if a value was out of bounds for a dimension, returning the
    ///   index of the failing dimension.
    ///
    /// # Panics
    /// * Panics if the number of problem-specific expressed values (total expressed values
    ///   minus `NUM_SYSTEM_PARAMETERS`) does not match the number of dimensions
    ///   defined in `dimensions_container`, unless both are zero. This panic
    ///   originates from `calculate_dimensions_key`.
    pub fn update_dimensions_key(
        &mut self,
        dimensions_container: &Dimensions,
    ) -> Result<(), usize> {
        // System parameters are the first NUM_SYSTEM_PARAMETERS values in `expressed`.
        // The remaining values are for the problem's dimensions.
        let problem_expressed_values = if self.expressed.len() > NUM_SYSTEM_PARAMETERS {
            &self.expressed[NUM_SYSTEM_PARAMETERS..]
        } else {
            &[] // No problem-specific parameters
        };

        let actual_dimensions = dimensions_container.get_dimensions();

        match calculate_dimensions_key(actual_dimensions, problem_expressed_values) {
            CalculateDimensionsKeyResult::Success(key) => {
                self.dimensions_key = Some(key);
                Ok(())
            }
            CalculateDimensionsKeyResult::Failure {
                dimension_index, ..
            } => {
                self.dimensions_key = None; // Clear previous key on failure
                Err(dimension_index)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::world::dimensions::dimension::Dimension;
    use std::ops::RangeInclusive;

    // Helper function to create a Phenotype instance for testing purposes.
    // It initializes a Phenotype with specified expressed values using the test constructor.
    // The dimensions_key is explicitly set to None for test predictability.
    fn create_phenotype_for_test(expressed_values: Vec<f64>) -> Phenotype {
        // Ensure there are enough values for system parameters for Phenotype::new_for_test.
        if expressed_values.len() < NUM_SYSTEM_PARAMETERS {
            panic!(
                "Test setup error: expressed_values length {} is less than NUM_SYSTEM_PARAMETERS {}",
                expressed_values.len(),
                NUM_SYSTEM_PARAMETERS
            );
        }

        let mut phenotype = Phenotype::new_for_test(expressed_values);
        phenotype.set_dimensions_key(None); // Ensure it starts as None for tests
        phenotype
    }

    // Helper to create a Dimensions struct for testing.
    // bounds_divisions: Vec of (range, num_divisions_for_this_dimension)
    fn create_test_dimensions(bounds_divisions: Vec<(RangeInclusive<f64>, usize)>) -> Dimensions {
        let dims: Vec<Dimension> = bounds_divisions
            .into_iter()
            .map(|(bounds, num_divisions)| {
                let mut dim = Dimension::new(bounds, 0); // Start with 0 divisions
                dim.set_number_of_divisions(num_divisions);
                dim
            })
            .collect();

        Dimensions::new_for_test(dims)
    }

    #[test]
    fn given_valid_inputs_when_update_dimensions_key_then_success_and_key_updated() {
        let mut phenotype = create_phenotype_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0,
            2.0, // System params (NUM_SYSTEM_PARAMETERS)
            7.5, 60.0, // Problem params
        ]);
        let dimensions = create_test_dimensions(vec![
            (0.0..=10.0, 2), // Dimension for 7.5 (0-5, 5-10), 2 divisions = 3 intervals. Index 1: [5,10)
            (0.0..=100.0, 4), // Dimension for 60.0 (0-25, 25-50, 50-75, 75-100), 4 divisions = 5 intervals. Index 2: [50,75)
        ]);

        let result = phenotype.update_dimensions_key(&dimensions);

        assert_eq!(result, Ok(()));
        // For Dimension::new(0.0..=10.0, 2 divisions): intervals are [0,5), [5,10), [10,10]. 7.5 is in [5,10) -> index 1.
        // For Dimension::new(0.0..=100.0, 4 divisions): intervals are [0,25), [25,50), [50,75), [75,100), [100,100]. 60.0 is in [50,75) -> index 2.
        assert_eq!(phenotype.dimensions_key, Some(vec![1, 2]));
    }

    #[test]
    fn given_value_out_of_bounds_when_update_dimensions_key_then_failure_and_index_returned() {
        let mut phenotype = create_phenotype_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params
            7.5, 101.0, // Problem params, 101.0 is out of bounds for the second dimension
        ]);
        let dimensions = create_test_dimensions(vec![(0.0..=10.0, 2), (0.0..=100.0, 4)]);

        let result = phenotype.update_dimensions_key(&dimensions);

        assert_eq!(result, Err(1)); // Fails at problem dimension index 1 (overall dimension index 1)
        assert_eq!(phenotype.dimensions_key, None); // Key should be None on failure
    }

    #[test]
    fn given_no_problem_parameters_and_no_dimensions_when_update_dimensions_key_then_success_empty_key()
     {
        let mut phenotype = create_phenotype_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params only
        ]);
        let dimensions = create_test_dimensions(vec![]); // No dimensions

        let result = phenotype.update_dimensions_key(&dimensions);

        assert_eq!(result, Ok(()));
        assert_eq!(phenotype.dimensions_key, Some(vec![]));
    }

    #[test]
    #[should_panic(
        expected = "The number of dimensions must match the number of expressed values."
    )]
    fn given_mismatched_problem_params_and_dimensions_when_update_dimensions_key_then_panics() {
        let mut phenotype = create_phenotype_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params
            7.5, // One problem param
        ]);
        // Two dimensions defined, but only one problem-specific expressed value
        let dimensions = create_test_dimensions(vec![(0.0..=10.0, 2), (0.0..=100.0, 4)]);

        let _ = phenotype.update_dimensions_key(&dimensions); // Panics in calculate_dimensions_key
    }

    #[test]
    fn given_previous_key_exists_when_update_fails_then_key_is_cleared() {
        let mut phenotype = create_phenotype_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params
            5.0, 25.0, // Problem params, initially valid
        ]);
        let dimensions_spec = vec![
            (0.0..=10.0, 2), // For 5.0
            (0.0..=50.0, 2), // For 25.0
        ];
        let dimensions = create_test_dimensions(dimensions_spec.clone());

        // Set an initial valid key
        let _ = phenotype.update_dimensions_key(&dimensions);
        assert!(
            phenotype.dimensions_key.is_some(),
            "Initial key should be set"
        );

        // Now, modify phenotype's expressed values to cause a failure with the same dimensions
        phenotype.expressed = vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params
            5.0, 250.0, // Problem params, 250.0 is now out of bounds for the second dimension
        ];
        // Important: Re-update dependent fields if they were changed by Phenotype::new or other methods
        phenotype.system_parameters = crate::parameters::system_parameters::SystemParameters::new(
            &phenotype.expressed[0..NUM_SYSTEM_PARAMETERS],
        );
        phenotype.expressed_hash =
            Phenotype::compute_expressed_hash(&phenotype.expressed, NUM_SYSTEM_PARAMETERS);

        let result = phenotype.update_dimensions_key(&dimensions); // Using the same dimensions object

        assert_eq!(
            result,
            Err(1),
            "Update should fail for the second problem dimension"
        );
        assert_eq!(
            phenotype.dimensions_key, None,
            "Dimensions key should be cleared on failure."
        );
    }

    #[test]
    fn given_problem_parameters_but_no_dimensions_when_update_dimensions_key_then_panics() {
        let mut phenotype = create_phenotype_for_test(vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System params
            7.5, // One problem param
        ]);
        let dimensions = create_test_dimensions(vec![]); // No dimensions defined

        // This should panic because problem_expressed_values.len() (1) != actual_dimensions.len() (0)
        // The panic message comes from calculate_dimensions_key.
        let result = std::panic::catch_unwind(move || phenotype.update_dimensions_key(&dimensions));
        assert!(result.is_err());
        // Further check on panic message if possible, but catch_unwind is enough to show it panics.
    }
}
