use std::fmt::Debug;

pub trait WorldFunction: Debug {
    fn run(&self, phenotype: &[f64]) -> Vec<f64>;
}
