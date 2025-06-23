use crate::parameters::global_constants::GlobalConstants;
use crate::world::dimensions::Dimensions;
use organisms::Organisms;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use regions::Regions; // Required for SmallRng::from_seed
use std::ops::RangeInclusive;

const DEFAULT_WORLD_SEED: u64 = 2_147_483_647; // A Mersenne prime (2^31 - 1)

pub mod dimensions;
pub mod organisms;
pub mod regions;

#[derive(Debug, Clone)]
pub struct World<F>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    _dimensions: Dimensions,
    _organisms: Organisms,
    _regions: Regions,
    _global_constants: GlobalConstants,
    _rng: SmallRng,
    _world_function: F,
}

impl<F> World<F>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    /// Creates a new `World` instance, initializing the entire simulation environment.
    ///
    /// This function orchestrates the setup of the world based on the initial parameters,
    /// aligning with the initialization process described in the Product Definition Document (PDD),
    /// particularly section 5.1.
    ///
    /// # Arguments
    ///
    /// * `user_defined_parameter_bounds` - A slice of `RangeInclusive<f64>` defining the
    ///   bounds for the problem-specific parameters to be optimized.
    /// * `global_constants` - A struct containing system-wide constants like `population_size`
    ///   and `max_regions`.
    /// * `world_function` - The fitness function `F` that takes an organism's expressed
    ///   phenotype (`&[f64]`) and returns a vector of output values.
    ///
    /// # Returns
    ///
    /// A fully initialized `World` object ready for the simulation to begin.
    ///
    /// # Process
    ///
    /// 1.  **Seeded RNG:** A deterministic pseudo-random number generator (`SmallRng`) is created
    ///     to ensure reproducibility of the simulation.
    /// 2.  **Initial Population (PDD 5.1.3):** An initial population of `Organisms` is generated.
    ///     Each organism is created with a random phenotype, whose genetic material (loci)
    ///     is initialized within the specified `user_defined_parameter_bounds` and the standard
    ///     system parameter bounds.
    /// 3.  **Initialize Space (PDD 5.1.4):**
    ///     a. **Bounding Box:** The initial spatial limits (an n-dimensional bounding box) are
    ///     determined by finding the min/max expressed values for each dimension across the
    ///     entire initial population.
    ///     b. **Dimensions:** A `Dimensions` object is created to manage the coordinate system
    ///     based on these initial limits.
    ///     c. **Regions:** A `Regions` object is initialized.
    ///     d. **Region Division & Assignment:** The `regions.update()` method is called. This crucial
    ///     step divides the n-dimensional space into regions based on the `Dimensions` and
    ///     assigns each organism to its corresponding region by calculating its `region_key`.
    ///     It also calculates the initial carrying capacities for these new regions.
    /// 4.  **World Construction:** The final `World` struct is assembled from the created components.
    pub fn new(
        user_defined_parameter_bounds: &[RangeInclusive<f64>],
        global_constants: GlobalConstants,
        world_function: F,
    ) -> Self {
        let mut rng = SmallRng::seed_from_u64(DEFAULT_WORLD_SEED);
        let mut organisms =
            Organisms::new(user_defined_parameter_bounds, &global_constants, &mut rng);

        let spacial_limits = organisms.find_spacial_limits();
        let mut dimensions = Dimensions::new(&spacial_limits, &global_constants);
        let mut regions = Regions::new(&global_constants);

        // This call performs the initial region division and organism assignment.
        regions.update(&mut organisms, &mut dimensions);

        World {
            _dimensions: dimensions,
            _organisms: organisms,
            _regions: regions,
            _global_constants: global_constants,
            _rng: rng,
            _world_function: world_function,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parameters::parameter::Parameter;
    use crate::parameters::parameter_enhancement::enhance_parameters;

    // A simple fitness function for testing purposes.
    fn test_world_function(phenotype: &[f64]) -> Vec<f64> {
        // Return the sum of the phenotype values as a single output.
        vec![phenotype.iter().sum()]
    }

    fn get_default_test_bounds() -> Vec<RangeInclusive<f64>> {
        // Initialize with a small, valid range to avoid overflow in the random number generator.
        let problem_parameters = vec![Parameter::with_bounds(0.0, -10.0, 10.0)];
        let problem_bounds: Vec<RangeInclusive<f64>> = problem_parameters
            .iter()
            .map(|p| p.bounds().clone())
            .collect();

        enhance_parameters(&problem_bounds)
    }

    #[test]
    fn given_valid_inputs_when_new_is_called_then_world_is_initialized_correctly() {
        let bounds = get_default_test_bounds();
        let global_constants = GlobalConstants::new(10, 100);

        let world = World::new(
            bounds.as_slice(),
            global_constants.clone(),
            test_world_function,
        );

        // 1. Check if the number of organisms matches the configuration.
        assert_eq!(
            world._organisms.count(),
            global_constants.population_size(),
            "The number of organisms should match the specified population size."
        );

        // 2. Check if regions have been created and are not empty.
        assert!(
            !world._regions.regions().is_empty(),
            "Regions should be initialized and not empty."
        );

        // 3. Check if all organisms have been assigned to a region.
        let all_organisms_have_region_key =
            world._organisms.iter().all(|o| o.region_key().is_some());
        assert!(
            all_organisms_have_region_key,
            "All organisms must have a region key after world initialization."
        );

        // 4. Verify that the number of populated regions corresponds to the organisms' locations.
        let populated_region_count = world._regions.regions().len();
        assert!(
            populated_region_count > 0
                && populated_region_count <= global_constants.population_size(),
            "The number of populated regions should be greater than 0 but not more than the population size."
        );
    }

    #[test]
    #[should_panic(expected = "Max regions cannot be zero.")]
    fn given_zero_max_regions_when_new_is_called_then_it_panics() {
        let bounds = get_default_test_bounds();
        // GlobalConstants::new panics if max_regions is 0.
        let global_constants = GlobalConstants::new(10, 0);

        World::new(bounds.as_slice(), global_constants, test_world_function);
    }

    #[test]
    #[should_panic(expected = "Population size cannot be zero.")]
    fn given_zero_population_size_when_new_is_called_then_it_panics() {
        let bounds = get_default_test_bounds();
        // GlobalConstants::new panics if population_size is 0.
        let global_constants = GlobalConstants::new(0, 100);

        World::new(bounds.as_slice(), global_constants, test_world_function);
    }
}
