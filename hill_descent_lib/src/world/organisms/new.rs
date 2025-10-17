use super::generate_random_phenotypes;
use crate::parameters::global_constants::GlobalConstants;
use crate::parameters::parameter_enhancement::enhance_parameters;
use crate::world::organisms::Organisms;
use crate::world::organisms::organism::Organism;
use rand::Rng;
use std::ops::RangeInclusive;
use std::sync::Arc;

impl Organisms {
    /// Creates a new `Organisms` collection with a specified population size.
    ///
    /// This function generates a set of random phenotypes based on the provided bounds
    /// and global constants. Each resulting organism is initialized with a random age,
    /// determined by the `max_age` system parameter from its phenotype.
    ///
    /// # Arguments
    ///
    /// * `initial_value_bounds` - The bounds for the problem-specific parameters.
    /// * `global_constants` - Global constants, including the population size.
    /// * `rng` - A mutable reference to a random number generator.
    ///
    /// # Returns
    ///
    /// A new `Organisms` instance populated with newly created organisms.
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
                .map(|p| {
                    let max_age = p.system_parameters().max_age();
                    let upper_bound = if max_age > 0.0 { max_age as usize } else { 0 };
                    let age = if upper_bound > 0 {
                        rng.random_range(0..=upper_bound)
                    } else {
                        0
                    };

                    Arc::new(Organism::new(Arc::new(p), age, (None, None)))
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::parameters::GlobalConstants;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn given_valid_inputs_when_new_called_then_creates_organisms_correctly() {
        let initial_value_bounds = vec![0.0..=1.0, 0.0..=1.0];
        let global_constants = GlobalConstants::new(10, 4);
        let mut rng = SmallRng::seed_from_u64(0);

        let organisms = Organisms::new(&initial_value_bounds, &global_constants, &mut rng);

        assert_eq!(organisms.organisms.len(), 10);
        assert_eq!(
            organisms.organisms[0].phenotype().gamete1().len(),
            initial_value_bounds.len() + NUM_SYSTEM_PARAMETERS
        );
        // This is implicitly tested by Phenotype::new_random_phenotype, which expects enhanced_parameter_bounds
        for organism in organisms.organisms {
            // Check that the age is within the valid range.
            let max_age = organism.phenotype().system_parameters().max_age();
            assert!(
                (organism.age() as f64) <= max_age,
                "Organism age {} should be less than or equal to max_age {}",
                organism.age(),
                max_age
            );
        }
    }
    // The following test was removed because GlobalConstants::new panics if population_size is 0,
    // so Organisms::new will not be called with such a GlobalConstants instance.
    // The behavior of generating zero phenotypes is tested in
    // world::organisms::generate_random_phenotypes::tests::given_zero_population_size_when_called_then_returns_empty_vector.
}
