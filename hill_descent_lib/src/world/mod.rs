//! The main simulation container and fitness evaluation interfaces.
//!
//! This module contains the [`World`] struct, which is the primary entry point for
//! running genetic algorithm optimizations, as well as the traits for defining
//! optimization functions.
//!
//! # Key Types
//!
//! - [`World`] - The main optimization container created by [`setup_world`](crate::setup_world)
//! - [`single_valued_function::SingleValuedFunction`] - Trait for defining single-output optimization functions (most common)
//! - [`world_function::WorldFunction`] - Advanced trait for multi-output functions with external inputs
//!
//! # Basic Usage
//!
//! ```
//! use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
//!
//! #[derive(Debug)]
//! struct MyFunction;
//!
//! impl SingleValuedFunction for MyFunction {
//!     fn single_run(&self, params: &[f64]) -> f64 {
//!         params.iter().map(|x| x * x).sum()
//!     }
//! }
//!
//! use hill_descent_lib::TrainingData;
//! let param_range = vec![-10.0..=10.0; 2];
//! let constants = GlobalConstants::new(100, 10);
//! let mut world = setup_world(&param_range, constants, Box::new(MyFunction));
//!
//! for _ in 0..100 {
//!     world.training_run(TrainingData::None { floor_value: 0.0 });
//! }
//! println!("Best score: {}", world.get_best_score());
//! ```

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
mod get_best_params;
mod get_best_score;
mod get_state;
mod get_state_for_web;
pub mod organisms;
mod regions;
mod remove_dead;
pub mod single_valued_function;
mod training_run;
pub mod world_function;

/// The main optimization container managing population evolution and fitness evaluation.
///
/// `World` is created by [`setup_world`](crate::setup_world) and contains all the state
/// needed for genetic algorithm optimization, including:
///
/// - **Population**: A collection of organisms (candidate solutions)
/// - **Spatial regions**: Adaptive partitioning of the parameter space
/// - **Evaluation function**: Your fitness function implementing [`single_valued_function::SingleValuedFunction`] or [`world_function::WorldFunction`]
/// - **Configuration**: Global constants controlling population size, regions, etc.
///
/// # Core Workflow
///
/// 1. Create world with [`setup_world`](crate::setup_world)
/// 2. Run optimization with [`training_run`](World::training_run)
/// 3. Extract results with [`get_best_score`](World::get_best_score) and [`get_best_organism`](World::get_best_organism)
///
/// # Examples
///
/// ## Basic Optimization
///
/// ```
/// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
///
/// #[derive(Debug)]
/// struct Sphere;
///
/// impl SingleValuedFunction for Sphere {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         params.iter().map(|x| x * x).sum()
///     }
/// }
///
/// use hill_descent_lib::TrainingData;
/// let param_range = vec![-5.0..=5.0; 3];  // 3D problem
/// let constants = GlobalConstants::new(200, 20);
/// let mut world = setup_world(&param_range, constants, Box::new(Sphere));
///
/// // Run 100 epochs of evolution
/// for _ in 0..100 {
///     world.training_run(TrainingData::None { floor_value: 0.0 });
/// }
///
/// // Get results
/// let best_score = world.get_best_score();
/// assert!(best_score < 0.01);  // Should find near-zero minimum
/// ```
///
/// ## Progressive Optimization with Monitoring
///
/// ```
/// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
///
/// #[derive(Debug)]
/// struct Rosenbrock;
///
/// impl SingleValuedFunction for Rosenbrock {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         let x = params[0];
///         let y = params[1];
///         (1.0 - x).powi(2) + 100.0 * (y - x.powi(2)).powi(2)
///     }
/// }
///
/// use hill_descent_lib::TrainingData;
/// let param_range = vec![-5.0..=5.0; 2];
/// let constants = GlobalConstants::new(500, 50);
/// let mut world = setup_world(&param_range, constants, Box::new(Rosenbrock));
///
/// // Run optimization in stages, monitoring progress
/// for stage in 0..10 {
///     for _ in 0..50 {
///         world.training_run(TrainingData::None { floor_value: 0.0 });
///     }
///     println!("Stage {}: Best score = {}", stage, world.get_best_score());
///     
///     if world.get_best_score() < 0.001 {
///         println!("Converged after {} total epochs", (stage + 1) * 50);
///         break;
///     }
/// }
/// ```
///
/// # Performance Characteristics
///
/// - **Parallel Evaluation**: Fitness evaluations run concurrently across CPU cores
/// - **Memory Efficient**: Organism storage grows with population size, not parameter dimensionality
/// - **Scalability**: Handles 2D to 100D+ problems; larger populations for higher dimensions
///
/// # Thread Safety
///
/// `World` is **not** thread-safe and should not be shared across threads. However,
/// the fitness function implementations must be `Sync` since they're called concurrently
/// during each epoch.
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
    /// **Note**: Use [`setup_world`](crate::setup_world) instead of calling this directly.
    /// This constructor is public to support advanced use cases but most users should
    /// use the convenience function.
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
