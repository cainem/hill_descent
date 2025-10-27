// Use mimalloc as the global allocator for better performance on Windows
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Internal modules - not exposed in public API
mod gamete;
mod gen_hybrid_range;
mod locus;
mod phenotype;

// Public modules containing public types and traits
pub mod parameters;
pub mod world;

#[cfg(test)]
pub mod test_utils;

#[cfg(feature = "enable-tracing")]
pub mod tracing_init;

#[cfg(feature = "enable-tracing")]
pub use tracing_init::init as init_tracing;

// Re-export log macros for unified logging interface when tracing enabled
#[cfg(feature = "enable-tracing")]
pub use log::{debug, error, info, trace, warn};

// When logging is disabled, provide no-op macros
#[cfg(not(feature = "enable-tracing"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{}};
}

#[cfg(not(feature = "enable-tracing"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{}};
}

#[cfg(not(feature = "enable-tracing"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{}};
}

#[cfg(not(feature = "enable-tracing"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{}};
}

#[cfg(not(feature = "enable-tracing"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{}};
}

// Internal constants used by the algorithm implementation
pub(crate) const NUM_SYSTEM_PARAMETERS: usize = 7;
pub(crate) const E0: f64 = f64::MIN_POSITIVE;

use std::ops::RangeInclusive;

// Re-export core public types for convenient imports
pub use parameters::GlobalConstants;
pub use world::World;
pub use world::single_valued_function::SingleValuedFunction;
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
