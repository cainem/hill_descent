//! Setup world - creates a new World with initial population.

use std::ops::RangeInclusive;

use super::{World, WorldFunction};
use crate::parameters::GlobalConstants;

/// Creates a new World ready for optimization.
pub fn setup_world(
    param_range: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    world_function: Box<dyn WorldFunction>,
) -> World {
    World::new(param_range, global_constants, world_function)
}
