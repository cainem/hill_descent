//! Setup world - creates a new World with initial population.

use std::ops::RangeInclusive;

use super::{World, WorldFunction};
use crate::parameters::GlobalConstants;

/// Creates a new World ready for optimization.
///
/// # Arguments
///
/// * `param_range` - Bounds for each parameter dimension
/// * `global_constants` - Configuration (population size, target regions, seed)
/// * `world_function` - The fitness function to optimize
///
/// # Returns
///
/// A new World with initial random population.
///
/// # Example
///
/// ```ignore
/// use hill_descent_lib2::{setup_world, GlobalConstants, SingleValuedFunction};
///
/// #[derive(Debug)]
/// struct Sphere;
/// impl SingleValuedFunction for Sphere {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         params.iter().map(|x| x * x).sum()
///     }
/// }
///
/// let bounds = vec![-10.0..=10.0, -10.0..=10.0];
/// let constants = GlobalConstants::new(100, 10);
/// let world = setup_world(&bounds, constants, Box::new(Sphere));
/// ```
pub fn setup_world(
    param_range: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    world_function: Box<dyn WorldFunction>,
) -> World {
    World::new(param_range, global_constants, world_function)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::single_valued_function::SingleValuedFunction;

    // Mock WorldFunction for testing
    #[derive(Debug)]
    struct TestFunction;

    impl SingleValuedFunction for TestFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            0.0
        }
    }

    #[test]
    fn given_valid_params_when_setup_world_then_world_created() {
        let bounds = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new(50, 5);

        let world = setup_world(&bounds, constants, Box::new(TestFunction));

        assert_eq!(world.organism_count(), 50);
    }

    #[test]
    fn given_setup_world_when_organism_count_then_equals_population_size() {
        let bounds = vec![0.0..=100.0];
        let constants = GlobalConstants::new(100, 10);

        let world = setup_world(&bounds, constants, Box::new(TestFunction));

        assert_eq!(world.organism_count(), constants.population_size());
    }

    #[test]
    fn given_setup_world_when_dimension_version_then_is_zero() {
        let bounds = vec![-5.0..=5.0, -5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new(30, 3);

        let world = setup_world(&bounds, constants, Box::new(TestFunction));

        assert_eq!(world.dimension_version(), 0);
    }
}
