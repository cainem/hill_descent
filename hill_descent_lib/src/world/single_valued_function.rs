use crate::WorldFunction;

use std::fmt::Debug;
/// this trait represents the special case for a function that represents a
/// line or a surface (or even higher dimensions) for which we want to minimize a single
/// value
/// A simple line graph or the height of a surface say is a good example
///
/// Here the expressed values are fed into the function and a value is returned and the
/// algorithm will vary the expressed values to minimize the return value
pub trait SingleValuedFunction: Debug {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64;
}

impl<T> WorldFunction for T
where
    T: SingleValuedFunction + Debug,
{
    /// Adapts the `single_run` interface to the `WorldFunction` interface by wrapping the
    /// single scalar result in a `Vec`.
    fn run(&self, phenotype_expressed_values: &[f64], _inputs: &[f64]) -> Vec<f64> {
        vec![self.single_run(phenotype_expressed_values)]
    }
}
