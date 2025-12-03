//! World module containing fitness function traits and world orchestration.
//!
//! This module provides:
//! - [`SingleValuedFunction`] - The primary trait for optimization functions
//! - [`WorldFunction`] - Advanced trait for multi-output functions
//! - [`World`] - The coordinator that orchestrates training runs
//! - [`Dimensions`] - Spatial bounds with versioning
//! - [`Regions`] - Spatial partitions for organism management
//!
//! # Usage
//!
//! Most users should implement [`SingleValuedFunction`]:
//!
//! ```
//! use hill_descent_lib2::SingleValuedFunction;
//!
//! #[derive(Debug)]
//! struct MyFunction;
//!
//! impl SingleValuedFunction for MyFunction {
//!     fn single_run(&self, params: &[f64]) -> f64 {
//!         params.iter().map(|x| x * x).sum()
//!     }
//! }
//! ```

pub mod dimensions;
pub mod new;
pub mod regions;
pub mod single_valued_function;
pub mod world_function;
pub mod world_struct;

// Training run steps
pub mod age_and_cull;
pub mod calculate_region_keys;
pub mod evaluate_fitness;
pub mod get_best_organism;
pub mod get_best_params;
pub mod get_best_score;
pub mod reproduction;
pub mod setup_world;
pub mod training_run;

pub use dimensions::Dimensions;
pub use regions::Regions;
pub use single_valued_function::SingleValuedFunction;
pub use world_function::WorldFunction;
pub use world_struct::World;
