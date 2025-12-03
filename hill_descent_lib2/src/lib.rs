//! # hill_descent_lib2
//!
//! A re-architected genetic algorithm library for n-dimensional optimization problems.
//!
//! This library uses a message-passing concurrency model based on `messaging_thread_pool`
//! to achieve better performance for complex fitness functions. Each organism lives on
//! a dedicated thread, eliminating shared mutable state and lock contention.
//!
//! ## Architecture
//!
//! - **Organisms**: Pool items that process messages on their assigned thread
//! - **Regions**: Spatial partitions managed via Rayon parallelism
//! - **World**: Coordinator that orchestrates training runs via message batches
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use hill_descent_lib2::{GlobalConstants, SingleValuedFunction, setup_world};
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
//! use hill_descent_lib2::TrainingData;
//! let bounds = vec![-10.0..=10.0, -10.0..=10.0];
//! let constants = GlobalConstants::new(100, 10);
//! let mut world = setup_world(&bounds, constants, Box::new(Quadratic));
//!
//! for _ in 0..100 {
//!     world.training_run(TrainingData::None { floor_value: 0.0 });
//! }
//!
//! println!("Best score: {}", world.get_best_score());
//! ```
//!
//! ## Key Differences from hill_descent_lib
//!
//! - Uses `messaging_thread_pool` for organism management
//! - Parallel dimension bound checking in a single pass
//! - No `Arc<Organism>` or `Mutex` overhead
//! - Better CPU cache locality through thread affinity
//!
//! ## Features
//!
//! - N-dimensional optimization (tested with 100+ dimensions)
//! - Adaptive spatial regions for efficient exploration
//! - Deterministic results via seeded RNG
//! - Parallel processing with message passing
//! - Optional tracing support (feature: `enable-tracing`)

// Stage 1 - Copied from hill_descent_lib
pub mod gamete;
pub(crate) mod gen_hybrid_range;
pub mod locus;
pub mod parameters;
pub mod phenotype;
pub mod training_data;
pub mod world;

// Stage 2 - New architecture
pub mod organism;

// Public API re-exports
pub use parameters::GlobalConstants;
pub use training_data::TrainingData;
pub use world::World;
pub use world::setup_world::setup_world;
pub use world::single_valued_function::SingleValuedFunction;
pub use world::world_function::WorldFunction;

/// Number of system parameters used by the genetic algorithm.
/// These are the first 7 expressed values from a phenotype.
pub(crate) const NUM_SYSTEM_PARAMETERS: usize = 7;

/// Minimum positive value used in fitness calculations to avoid division by zero.
pub(crate) const E0: f64 = f64::MIN_POSITIVE;

#[cfg(test)]
mod tests {
    #[test]
    fn given_crate_when_built_then_compiles_successfully() {
        // Placeholder test to verify crate compiles
        assert!(true);
    }
}
