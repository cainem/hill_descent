use super::generate_random_phenotypes;
use crate::parameters::global_constants::GlobalConstants;
use crate::parameters::parameter_enhancement::enhance_parameters;
use crate::world::organisms::Organisms;
use crate::world::organisms::organism::Organism;
use rand::Rng;
use std::ops::RangeInclusive;
use std::rc::Rc;

impl Organisms {
    pub fn new(
        initial_value_bounds: &[RangeInclusive<f64>],
        global_constants: &GlobalConstants,
        rng: &mut impl Rng,
    ) -> Self {
        // Combine system parameter bounds with the problem-specific initial bounds.
        let parameter_bounds = enhance_parameters(initial_value_bounds);

        let phenotypes =
            generate_random_phenotypes(rng, &parameter_bounds, global_constants.population_size());

        Self {
            organisms: phenotypes
                .into_iter()
                .map(|p| Organism::new(Rc::new(p)))
                .collect(),
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
        let initial_value_bounds = vec![0.0..=1.0, 0.0..=1.0];
        let global_constants = GlobalConstants::new(10, 4);
        let mut rng = StepRng::new(0, 1);

        let organisms = Organisms::new(&initial_value_bounds, &global_constants, &mut rng);

        assert_eq!(organisms.organisms.len(), 10);
        assert_eq!(
            organisms.organisms[0].phenotype().gamete1().len(),
            initial_value_bounds.len() + NUM_SYSTEM_PARAMETERS
        );
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
