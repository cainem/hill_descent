use hill_descent_lib::world::single_valued_function::SingleValuedFunction;
use hill_descent_lib::WorldFunction;
use std::ops::RangeInclusive;

/// Trait for benchmark algorithms that can be tested
pub trait BenchmarkAlgorithm {
    /// Get the name of the algorithm for file naming
    fn name(&self) -> &'static str;

    /// Get the parameter ranges for this algorithm
    fn param_ranges(&self) -> Vec<RangeInclusive<f64>>;

    /// Get the function implementation
    fn function(&self) -> Box<dyn WorldFunction>;
}

/// Himmelblau's function - standard test function with four identical local minima
/// f(x, y) = (x^2 + y - 11)^2 + (x + y^2 - 7)^2
/// Global minimum is 0, typically tested on domain [-5, 5] × [-5, 5]
pub struct HimmelblauAlgorithm;

impl BenchmarkAlgorithm for HimmelblauAlgorithm {
    fn name(&self) -> &'static str {
        "himmelblau"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-5.0, 5.0),
            RangeInclusive::new(-5.0, 5.0),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(Himmelblau)
    }
}

/// Ackley function - widely-used multimodal benchmark for optimization
/// f(x, y) = -20 * exp(-0.2 * sqrt(0.5 * (x² + y²))) - exp(0.5 * (cos(2πx) + cos(2πy))) + e + 20
/// Global minimum at (0,0) with f = 0.0, typically tested on domain [-5, 5] × [-5, 5]
pub struct AckleyAlgorithm;

impl BenchmarkAlgorithm for AckleyAlgorithm {
    fn name(&self) -> &'static str {
        "ackley"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-5.0, 5.0),
            RangeInclusive::new(-5.0, 5.0),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(Ackley)
    }
}

// Function implementations
#[derive(Debug)]
struct Himmelblau;

impl SingleValuedFunction for Himmelblau {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        let term1 = (x.powi(2) + y - 11.0).powi(2);
        let term2 = (x + y.powi(2) - 7.0).powi(2);
        term1 + term2
    }
}

#[derive(Debug)]
struct Ackley;

impl SingleValuedFunction for Ackley {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        let two_pi = 2.0 * std::f64::consts::PI;
        let e = std::f64::consts::E;

        let term1 = -20.0 * (-0.2 * (0.5 * (x * x + y * y)).sqrt()).exp();
        let term2 = -(0.5 * (two_pi * x).cos() + 0.5 * (two_pi * y).cos()).exp();

        term1 + term2 + e + 20.0
    }
}
