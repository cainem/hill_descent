pub mod gamete;
pub mod gen_hybrid_range;
pub mod locus;
pub mod parameters;
pub mod phenotype;
pub mod world;

pub const NUM_SYSTEM_PARAMETERS: usize = 7;
pub const E0: f64 = f64::MIN_POSITIVE;

use std::ops::RangeInclusive;

pub use gamete::Gamete;
pub use locus::Locus;
use parameters::GlobalConstants;
pub use phenotype::Phenotype;
use world::World;
use world::world_function::WorldFunction;

// this will take a list of parameters and return a world
pub fn setup_world(
    params: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    // The new function parameter
    function: Box<dyn WorldFunction>,
) -> World {
    World::new(params, global_constants, function)
}
