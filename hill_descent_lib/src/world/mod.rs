use crate::parameters::global_constants::GlobalConstants;
use crate::world::dimensions::Dimensions;
use organisms::Organisms;
use rand::SeedableRng;
use rand::rngs::StdRng;
use regions::Regions; // Required for StdRng::from_seed
use std::ops::RangeInclusive;

use world_function::WorldFunction;

mod dimensions;
mod get_best_organism;
mod get_best_score;
mod get_state;
mod get_state_for_web;
pub mod organisms;
mod regions;
mod remove_dead;
mod run_epoch;
pub mod single_valued_function;
mod training_run;
mod validate_training_sets;
pub mod world_function;

// Top-level simulation container holding dimensions, organisms, regions, RNG, and the evaluation function.

#[derive(Debug)]
pub struct World {
    dimensions: Dimensions,
    organisms: Organisms,
    regions: Regions,
    #[allow(dead_code)] // Kept for backward compatibility; replaced by per-region RNG
    rng: StdRng,
    world_function: Box<dyn WorldFunction>,
    global_constants: GlobalConstants,
}

impl World {
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
    ///   and `target_regions`.
    ///
    /// # Returns
    ///
    /// A fully initialized `World` object ready for the simulation to begin.
    ///
    /// # Process
    ///
    /// 1.  **Seeded RNG:** A deterministic pseudo-random number generator (`StdRng`) is created
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
        function: Box<dyn WorldFunction>,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(global_constants.world_seed());
        let mut organisms =
            Organisms::new(user_defined_parameter_bounds, &global_constants, &mut rng);

        let spacial_limits = organisms.find_spacial_limits();
        let mut dimensions = Dimensions::new(&spacial_limits);
        let mut regions = Regions::new(&global_constants);

        // This call performs the initial region division and organism assignment.
        regions.update(&mut organisms, &mut dimensions);

        World {
            dimensions,
            organisms,
            regions,
            rng,
            world_function: function,
            global_constants,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::world::world_function::WorldFunction;
    use std::ops::RangeInclusive;

    // Mock WorldFunction that returns 0.0 to validate World initialization logic.
    #[derive(Debug)]
    struct TestFn;
    impl WorldFunction for TestFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![0.0]
        }
    }

    #[test]
    fn given_valid_inputs_when_new_is_called_then_world_is_initialized_correctly() {
        let num_problem_dims = 2;
        let bounds: Vec<RangeInclusive<f64>> = (0..num_problem_dims)
            .map(|i| ((i + 10) as f64)..=((i + 11) as f64))
            .collect();
        let gc = GlobalConstants::new(100, 10);
        let world_fn: Box<dyn WorldFunction> = Box::new(TestFn);

        let world = World::new(&bounds, gc, world_fn);

        assert_eq!(world.organisms.len(), 100);
        assert_eq!(gc.population_size(), 100);
        assert_eq!(gc.target_regions(), 10);

        // Verify that the number of dimensions matches the problem space dimensions.
        assert_eq!(world.dimensions.num_dimensions(), num_problem_dims);

        // Check that regions have been created and organisms assigned.
        // With a small population and large target_regions, we expect at least one region.
        assert!(!world.regions.is_empty());
        // Every organism should have a region key after initialization.
        for organism in world.organisms.iter() {
            assert!(organism.region_key().is_some());
        }
    }

    #[test]
    #[should_panic(expected = "Max regions cannot be zero.")]
    fn given_zero_target_regions_when_new_is_called_then_it_panics() {
        let bounds: Vec<RangeInclusive<f64>> = Vec::new();
        let gc = GlobalConstants::new(10, 0);
        let world_fn: Box<dyn WorldFunction> = Box::new(TestFn);
        World::new(&bounds, gc, world_fn);
    }

    #[test]
    #[should_panic(expected = "Population size cannot be zero.")]
    fn given_zero_population_size_when_new_is_called_then_it_panics() {
        let bounds: Vec<RangeInclusive<f64>> = Vec::new();
        let gc = GlobalConstants::new(0, 100);
        let world_fn: Box<dyn WorldFunction> = Box::new(TestFn);
        World::new(&bounds, gc, world_fn);
    }
}
