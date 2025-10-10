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

/// Number of system-wide evolvable parameters (m1, m2, m3, m4, m5, m6, m6_sigma, max_age, crossover_points)
pub const NUM_SYSTEM_PARAMETERS: usize = 9;
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
