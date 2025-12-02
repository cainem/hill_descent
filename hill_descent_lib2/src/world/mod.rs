//! World module containing fitness function traits and world orchestration.
//!
//! This module provides:
//! - [`SingleValuedFunction`] - The primary trait for optimization functions
//! - [`WorldFunction`] - Advanced trait for multi-output functions
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

pub mod single_valued_function;
pub mod world_function;

pub use single_valued_function::SingleValuedFunction;
pub use world_function::WorldFunction;
