use super::Organisms;
use crate::world::dimensions::Dimensions;

impl Organisms {
    /// Updates the region key for all organisms.
    ///
    /// Iterates through each organism and calls `update_region_key`.
    /// If any update fails, this function returns the index of the first failing dimension.
    ///
    /// # Arguments
    /// * `dimensions`: The dimensions to use for calculating the keys.
    ///
    /// # Returns
    /// * `Ok(())` if all organism region keys were updated successfully.
    /// * `Err(usize)` with the dimension index of the first failure encountered.
    pub fn update_all_region_keys(&mut self, dimensions: &Dimensions) -> Result<(), usize> {
        for organism in self.organisms.iter_mut() {
            organism.update_region_key(dimensions)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    // Removed: use crate::phenotype::new_random_phenotype::new_random_phenotype;
    // No longer using Parameter directly in these tests
    // Removed: use crate::parameters::Parameter;
    use crate::parameters::global_constants::GlobalConstants;
    // Removed: use crate::world::dimensions::new_dimensions;
    use rand::thread_rng;
    use std::ops::RangeInclusive;

    // Helper to create basic dimensions for testing
    fn create_test_dimensions_for_organisms(num_dims: usize, max_regions: usize) -> Dimensions {
        let bounds: Vec<RangeInclusive<f64>> = (0..num_dims).map(|_| 0.0..=10.0).collect();
        let gc = GlobalConstants::new(100, max_regions);
        Dimensions::new(&bounds, &gc)
    }

    #[test]
    fn given_no_organisms_when_update_all_region_keys_then_ok() {
        let mut organisms = Organisms::new_from_phenotypes(vec![]);
        let dimensions = create_test_dimensions_for_organisms(2, 4);
        assert_eq!(organisms.update_all_region_keys(&dimensions), Ok(()));
    }

    #[test]
    fn given_organisms_all_update_successfully_when_update_all_region_keys_then_ok() {
        let mut rng = thread_rng();
        let param_bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, 0.0..=1.0]; // For problem space
        let full_bounds =
            crate::parameters::parameter_enhancement::enhance_parameters(&param_bounds);

        let p1 = Phenotype::new_random_phenotype(&mut rng, &full_bounds);
        let p2 = Phenotype::new_random_phenotype(&mut rng, &full_bounds);
        let mut organisms = Organisms::new_from_phenotypes(vec![p1, p2]);

        // Dimensions where all default random phenotypes (0.0 to 1.0) should fit.
        let dimensions = create_test_dimensions_for_organisms(param_bounds.len(), 4);

        let result = organisms.update_all_region_keys(&dimensions);
        assert_eq!(result, Ok(()));
        for organism in organisms.organisms.iter() {
            assert!(organism.region_key().is_some());
        }
    }

    #[test]
    fn given_one_organism_fails_update_when_update_all_region_keys_then_err() {
        let mut rng = thread_rng();
        let problem_param_bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0, 0.0..=10.0];
        let full_bounds =
            crate::parameters::parameter_enhancement::enhance_parameters(&problem_param_bounds);

        // Create a phenotype that should pass most dimension checks.
        let mut test_phenotypes = vec![Phenotype::new_random_phenotype(&mut rng, &full_bounds)];

        // Create a phenotype that will be made to fail.
        let failing_p_phenotype = Phenotype::new_random_phenotype(&mut rng, &full_bounds);
        let mut failing_organism =
            crate::world::organisms::organism::Organism::new(failing_p_phenotype.clone());

        // To make failing_organism fail, we create dimensions where its naturally-expressed value is out of bounds.
        // `new_random_phenotype` with default enhancement creates problem parameter values between 0.0 and 1.0.
        // These dimensions will make such a value fail for the first problem parameter.
        let specific_bounds_for_failure = vec![100.0..=110.0, 0.0..=1.0];
        let gc = GlobalConstants::new(100, 4);
        let dimensions_that_cause_failure = Dimensions::new(&specific_bounds_for_failure, &gc);

        // Pre-check: ensure our setup is correct and failing_organism does fail.
        assert!(
            failing_organism
                .update_region_key(&dimensions_that_cause_failure)
                .is_err()
        );

        test_phenotypes.push(failing_p_phenotype); // Add the failing phenotype.
        test_phenotypes.push(Phenotype::new_random_phenotype(&mut rng, &full_bounds)); // Add another that should pass.

        let mut organisms = Organisms::new_from_phenotypes(test_phenotypes);
        let result = organisms.update_all_region_keys(&dimensions_that_cause_failure);

        // The main assertion: the collection-level update should fail.
        assert!(result.is_err());
        // And it should report the index of the failing dimension (0 in this case).
        assert_eq!(result, Err(0));

        // Check state: The organism that was set up to fail should have no key.
        assert_eq!(organisms.organisms[1].region_key(), None);
    }
}
