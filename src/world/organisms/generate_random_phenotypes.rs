use crate::phenotype::Phenotype;
use rand::Rng;
use std::ops::RangeInclusive;

/// Generates a vector of random phenotypes.
///
/// This function creates a specified number of `Phenotype` instances, each initialized
/// randomly based on the provided enhanced parameter bounds. The `enhanced_parameter_bounds`
/// are expected to have already been processed (e.g., by `enhance_parameters`) to include
/// any necessary system-level parameters.
///
/// # Arguments
///
/// * `rng`: A mutable reference to a random number generator.
/// * `enhanced_parameter_bounds`: A slice of `RangeInclusive<f64>` representing the
///   bounds for each locus in the phenotypes. These bounds should already include
///   any system-specific parameters.
/// * `population_size`: The number of phenotypes to generate.
///
/// # Returns
///
/// A `Vec<Phenotype>` containing the newly generated random phenotypes.
pub fn generate_random_phenotypes(
    rng: &mut impl Rng,
    enhanced_parameter_bounds: &[RangeInclusive<f64>],
    population_size: usize,
) -> Vec<Phenotype> {
    let mut phenotypes = Vec::with_capacity(population_size);
    for _ in 0..population_size {
        phenotypes.push(Phenotype::new_random_phenotype(
            rng,
            enhanced_parameter_bounds,
        ));
    }
    phenotypes
}

#[cfg(test)]
mod tests {
    use super::*;
    // Phenotype is brought in by use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use rand::rngs::mock::StepRng;
    use std::ops::RangeInclusive; // Expected to be 7

    // Helper to create parameter bounds for testing
    fn create_test_enhanced_bounds(num_additional_params: usize) -> Vec<RangeInclusive<f64>> {
        let total_params = NUM_SYSTEM_PARAMETERS + num_additional_params;
        vec![0.0..=1.0; total_params]
    }

    #[test]
    fn given_valid_inputs_when_called_then_returns_correct_number_of_phenotypes() {
        let mut rng = StepRng::new(0, 1);
        let enhanced_bounds = create_test_enhanced_bounds(3); // 7 system + 3 additional
        let population_size = 5;

        let phenotypes = generate_random_phenotypes(&mut rng, &enhanced_bounds, population_size);

        assert_eq!(
            phenotypes.len(),
            population_size,
            "Should return the specified number of phenotypes"
        );
        // Further checks could involve inspecting properties of the phenotypes
        // if Phenotype had public fields or getters, and if new_random_phenotype
        // had more deterministic behavior with StepRng for its internal locus generation.
        // For now, we trust that new_random_phenotype works as tested elsewhere.
    }

    #[test]
    fn given_zero_population_size_when_called_then_returns_empty_vector() {
        let mut rng = StepRng::new(0, 1);
        let enhanced_bounds = create_test_enhanced_bounds(3);
        let population_size = 0;

        let phenotypes = generate_random_phenotypes(&mut rng, &enhanced_bounds, population_size);

        assert!(
            phenotypes.is_empty(),
            "Should return an empty vector for zero population size"
        );
    }

    #[test]
    fn given_bounds_with_only_system_parameters_when_called_then_succeeds() {
        let mut rng = StepRng::new(0, 1);
        // These bounds only contain the minimum required system parameters.
        let enhanced_bounds = create_test_enhanced_bounds(0);
        assert_eq!(
            enhanced_bounds.len(),
            NUM_SYSTEM_PARAMETERS,
            "Enhanced bounds should have exactly NUM_SYSTEM_PARAMETERS elements"
        );
        let population_size = 2;

        let phenotypes = generate_random_phenotypes(&mut rng, &enhanced_bounds, population_size);

        assert_eq!(
            phenotypes.len(),
            population_size,
            "Should generate phenotypes even with only system parameters in bounds"
        );
    }
}
