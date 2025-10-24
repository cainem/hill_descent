use std::fmt::Debug;

/// Defines the interface for the function that will be evaluated using the phenotypes
/// expressed values and a known set of inputs
///
/// The outputs of this function will then be compared against known outputs to determine the
/// fitness of the phenotype in the context of the problem being solved.
pub trait WorldFunction: Debug + Sync {
    fn run(&self, phenotype_expressed_values: &[f64], inputs: &[f64]) -> Vec<f64>;

    /// Returns the minimum possible value (floor) that the function can return.
    ///
    /// This is used for validation during fitness scoring. If the function returns a value
    /// below this floor, it indicates a bug in the function implementation and will cause
    /// a panic.
    ///
    /// # Default Implementation
    ///
    /// Returns `0.0`, which is appropriate for most optimization problems where the optimal
    /// score is zero (perfect match). Override this method if your function has a different
    /// known minimum value.
    ///
    /// # Returns
    ///
    /// The minimum value that this function can validly return.
    fn function_floor(&self) -> f64 {
        0.0
    }
}
