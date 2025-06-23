use std::fmt::Debug;

/// Defines the interface for fitness functions used within the `World`.
///
/// This trait allows for a flexible, swappable fitness function implementation. It requires
/// that any fitness function be debuggable.
pub trait WorldFunction: Debug {
    /// Executes the fitness function against a given phenotype.
    ///
    /// # Arguments
    ///
    /// * `phenotype` - A slice of `f64` representing the expressed genetic traits of an organism.
    ///
    /// # Returns
    ///
    /// A `Vec<f64>` containing the results of the fitness calculation.
    fn run(&self, phenotype: &[f64]) -> Vec<f64>;

    /// Configures the fitness function's internal state.
    ///
    /// This method allows the fitness function to be dynamically adjusted. For example,
    /// it could be configured based on the phenotype of another organism, enabling co-evolution
    /// scenarios.
    ///
    /// # Arguments
    ///
    /// * `phenotype_values` - A slice of `f64` used to configure the function.
    fn configure(&mut self, phenotype_values: &[f64]);
}
