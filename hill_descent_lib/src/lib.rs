//! # hill_descent_lib
//!
//! A genetic algorithm library for n-dimensional optimization problems.
//!
//! This library implements a spatial genetic algorithm that divides the search space into
//! adaptive regions, allowing efficient exploration of complex fitness landscapes. It's
//! particularly well-suited for optimization problems where gradient information is unavailable
//! or unreliable.
//!
//! ## Quick Start
//!
//! ```rust
//! use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
//! use std::ops::RangeInclusive;
//!
//! // Define your fitness function (lower is better)
//! #[derive(Debug)]
//! struct Quadratic;
//!
//! impl SingleValuedFunction for Quadratic {
//!     fn single_run(&self, params: &[f64]) -> f64 {
//!         // Minimize: x² + y²
//!         params[0].powi(2) + params[1].powi(2)
//!     }
//! }
//!
//! // Set up and run optimization
//! let bounds = vec![-10.0..=10.0, -10.0..=10.0];
//! let constants = GlobalConstants::new(100, 10);
//! let mut world = setup_world(&bounds, constants, Box::new(Quadratic));
//!
//! for _ in 0..100 {
//!     world.training_run(&[], &[0.0]);
//! }
//!
//! println!("Best score: {}", world.get_best_score());
//! ```
//!
//! ## Core Concepts
//!
//! - **World**: The main optimization container that manages the population and search space
//! - **Organisms**: Individual solutions with genetic material (DNA) that evolve over generations
//! - **Regions**: Spatial partitions of the search space that adapt based on organism distribution
//! - **Fitness Function**: User-defined function to minimize (implement [`SingleValuedFunction`])
//!
//! ## Features
//!
//! - N-dimensional optimization (tested with 100+ dimensions)
//! - Adaptive spatial regions for efficient exploration
//! - Deterministic results via seeded RNG
//! - Parallel processing with Rayon
//! - Optional tracing support (feature: `enable-tracing`)
//!
//! ## Algorithm Overview
//!
//! 1. **Initialization**: Random population within specified bounds
//! 2. **Evaluation**: Each organism scored by fitness function
//! 3. **Regional Competition**: Organisms compete within their spatial regions
//! 4. **Reproduction**: Better organisms get more offspring via sexual reproduction
//! 5. **Mutation**: Genetic variation through adaptive mutation
//! 6. **Adaptation**: Regions dynamically adjust based on population distribution
//!
//! The algorithm automatically manages carrying capacity, organism aging, mutation rates,
//! and search space expansion.

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

/// Creates and initializes a new optimization world.
///
/// This is the primary entry point for setting up an optimization problem. It creates
/// a [`World`] instance with a random initial population distributed according to the
/// specified parameter bounds.
///
/// # Arguments
///
/// * `params` - Slice of `RangeInclusive<f64>` defining the bounds for each dimension.
///   The length of this slice determines the dimensionality of the problem. Each range
///   specifies the minimum and maximum allowed values for that parameter.
///
/// * `global_constants` - Configuration for the genetic algorithm including population
///   size, number of regions, and random seed. See [`GlobalConstants`] for details.
///
/// * `function` - Boxed trait object implementing [`WorldFunction`]. This defines the
///   fitness function to optimize. Typically you implement [`SingleValuedFunction`] which
///   automatically provides the [`WorldFunction`] implementation.
///
/// # Returns
///
/// A fully initialized [`World`] ready to begin optimization via [`World::training_run`].
///
/// # Examples
///
/// Basic 2D optimization:
///
/// ```rust
/// use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
/// use std::ops::RangeInclusive;
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
/// let bounds = vec![-5.0..=5.0, -5.0..=5.0];
/// let constants = GlobalConstants::new(100, 10);
/// let world = setup_world(&bounds, constants, Box::new(Sphere));
/// ```
///
/// Higher dimensional problem:
///
/// ```rust
/// use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
/// use std::ops::RangeInclusive;
/// # #[derive(Debug)]
/// # struct Sphere;
/// # impl SingleValuedFunction for Sphere {
/// #     fn single_run(&self, params: &[f64]) -> f64 {
/// #         params.iter().map(|x| x * x).sum()
/// #     }
/// # }
///
/// // Optimize in 50 dimensions
/// let bounds = vec![-10.0..=10.0; 50];
/// let constants = GlobalConstants::new(500, 25);
/// let world = setup_world(&bounds, constants, Box::new(Sphere));
/// ```
///
/// # Panics
///
/// Panics if `global_constants` has a population size of zero or target regions of zero.
/// See [`GlobalConstants::new`] for details.
pub fn setup_world(
    params: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    function: Box<dyn WorldFunction>,
) -> World {
    World::new(params, global_constants, function)
}
