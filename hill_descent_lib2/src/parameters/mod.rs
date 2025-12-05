//! Configuration and parameter types for genetic algorithm control.
//!
//! This module contains [`GlobalConstants`], the main configuration struct for
//! controlling genetic algorithm behavior. Internal parameter types handle
//! system-level settings and enhancements.
//!
//! # Usage
//!
//! ```
//! use hill_descent_lib2::GlobalConstants;
//!
//! // Basic configuration
//! let constants = GlobalConstants::new(100, 10);
//!
//! // Deterministic configuration with seed
//! let constants = GlobalConstants::new_with_seed(100, 10, 42);
//! ```

pub mod global_constants;
pub(crate) mod parameter;
pub(crate) mod system_parameters;

pub use global_constants::GlobalConstants;
