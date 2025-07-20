pub mod gamete;
pub mod gen_hybrid_range;
pub mod locus;
pub mod parameters;
pub mod phenotype;
pub mod world;

#[cfg(test)]
pub mod test_utils;

#[cfg(feature = "enable-tracing")]
pub mod tracing_init;

#[cfg(feature = "enable-tracing")]
pub use tracing_init::init as init_tracing;

#[cfg(target_arch = "wasm32")]
pub mod wasm_interface;

pub const NUM_SYSTEM_PARAMETERS: usize = 7;
pub const E0: f64 = f64::MIN_POSITIVE;

use std::ops::RangeInclusive;

pub use gamete::Gamete;
pub use locus::Locus;
pub use parameters::GlobalConstants;
pub use phenotype::Phenotype;
pub use world::World;
pub use world::world_function::WorldFunction;

// this will take a list of parameters and return a world
pub fn setup_world(
    params: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    // The new function parameter
    function: Box<dyn WorldFunction>,
) -> World {
    World::new(params, global_constants, function)
}
