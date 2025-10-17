use std::fmt::Debug;

/// Defines the interface for the function that will be evaluated using the phenotypes
/// expressed values and a known set of inputs
///
/// The outputs of this function will then be compared against known outputs to determine the
/// fitness of the phenotype in the context of the problem being solved.
pub trait WorldFunction: Debug + Sync {
    fn run(&self, phenotype_expressed_values: &[f64], inputs: &[f64]) -> Vec<f64>;
}
