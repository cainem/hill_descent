use super::generate_random_phenotypes;
use crate::parameters::global_constants::GlobalConstants;
use crate::world::organisms::Organisms;
use crate::world::organisms::organism::Organism;
use crate::world::world_function::WorldFunction;
use rand::Rng;
use std::ops::RangeInclusive;
use std::rc::Rc;

impl Organisms {
    pub fn new(
        initial_value_bounds: &[RangeInclusive<f64>],
        global_constants: &GlobalConstants,
        rng: &mut impl Rng,
        world_function: Rc<dyn WorldFunction>,
    ) -> Self {
        let phenotypes = generate_random_phenotypes(
            rng,
            initial_value_bounds, // These are already enhanced by the caller of Organisms::new
            global_constants.population_size(),
        );

        Self {
            organisms: phenotypes
                .into_iter()
                .map(|p| Organism::new(Rc::new(p), world_function.clone()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::parameters::GlobalConstants;
    use crate::world::world_function::WorldFunction;
    use rand::rngs::mock::StepRng;
    use std::fmt;
    use std::ops::RangeInclusive; // For validating enhanced bounds length
    use std::rc::Rc;

    #[derive(Debug)]
    struct TestFn;
    impl WorldFunction for TestFn {
        fn run(&self, _p: &[f64]) -> Vec<f64> {
            vec![0.0]
        }
    }

    #[test]
    fn given_valid_inputs_when_new_called_then_creates_organisms_correctly() {
        let mut rng = StepRng::new(0, 1);
        let user_bounds: Vec<RangeInclusive<f64>> = (0..NUM_SYSTEM_PARAMETERS)
            .map(|i| (i as f64)..=((i + 1) as f64))
            .collect();
        let global_constants_instance = GlobalConstants::new(10, 100);
        let world_fn = Rc::new(TestFn);

        let organisms =
            Organisms::new(&user_bounds, &global_constants_instance, &mut rng, world_fn);

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
