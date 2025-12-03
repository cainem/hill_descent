//! Setup world - creates a new World with initial population.

use std::ops::RangeInclusive;
use std::sync::Arc;

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
    todo!("Implement setup_world")
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_valid_params_when_setup_world_then_world_created() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_setup_world_when_organism_count_then_equals_population_size() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_setup_world_when_dimension_version_then_is_zero() {
        todo!()
    }
}
