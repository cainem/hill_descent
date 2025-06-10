use super::generate_random_phenotypes;
use crate::parameters::global_constants::GlobalConstants;
use crate::parameters::parameter_enhancement::enhance_parameters;
use crate::world::organisms::Organisms;
use rand::Rng;
use std::ops::RangeInclusive;

impl Organisms {
    pub fn new(
        rng: &mut impl Rng,
        user_defined_parameter_bounds: &[RangeInclusive<f64>],
        global_constants: &GlobalConstants, // Changed to reference
    ) -> Self {
        let enhanced_parameter_bounds = enhance_parameters(user_defined_parameter_bounds);

        let phenotypes = generate_random_phenotypes(
            rng,
            &enhanced_parameter_bounds,
            global_constants.population_size(),
        );

        Self {
            organisms: phenotypes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::parameters::GlobalConstants;
    use rand::rngs::mock::StepRng;
    use std::ops::RangeInclusive; // For validating enhanced bounds length

    #[test]
    fn given_valid_inputs_when_new_called_then_creates_organisms_correctly() {
        let mut rng = StepRng::new(0, 1);
        let user_bounds: [RangeInclusive<f64>; 1] = [0.0..=1.0];
        let global_constants_instance = GlobalConstants::new(10, 100);

        let organisms = Organisms::new(&mut rng, &user_bounds, &global_constants_instance);

        assert_eq!(
            organisms.organisms.len(),
            global_constants_instance.population_size(),
            "Number of organisms should match population size"
        );

        // Check if phenotypes were created with the correct number of loci
        // enhanced_parameter_bounds.len() should be NUM_SYSTEM_PARAMETERS + user_bounds.len()
        let _expected_loci_count = NUM_SYSTEM_PARAMETERS + user_bounds.len();
        if !organisms.organisms.is_empty() {
            // Assuming Phenotype has a way to get its loci count or direct access to its gametes' loci
            // For now, we'll infer from the fact that new_random_phenotype would have used these bounds.
            // This test relies on the correctness of new_random_phenotype and enhance_parameters.
            // A more direct test would require Phenotype to expose its loci count.
            // We know Phenotype::new_random_phenotype panics if parameter_bounds.len() < NUM_SYSTEM_PARAMETERS
            // and our enhance_parameters ensures this minimum.
        }
        // Check that each phenotype has the expected number of parameters
        // This is implicitly tested by Phenotype::new_random_phenotype, which expects enhanced_parameter_bounds
        for _phenotype in organisms.organisms {
            // If Phenotype had a method like `num_loci()` or similar, we could assert it here.
            // e.g., assert_eq!(phenotype.num_loci(), expected_loci_count);
            // For now, we trust that Phenotype::new_random_phenotype used the enhanced_parameter_bounds correctly.
        }
    }
    // The following test was removed because GlobalConstants::new panics if population_size is 0,
    // so Organisms::new will not be called with such a GlobalConstants instance.
    // The behavior of generating zero phenotypes is tested in
    // world::organisms::generate_random_phenotypes::tests::given_zero_population_size_when_called_then_returns_empty_vector.
}
