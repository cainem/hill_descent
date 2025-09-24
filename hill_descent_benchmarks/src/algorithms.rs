/// Bukin N.6 function - challenging optimization test with a narrow curved valley
/// f(x, y) = 100*sqrt(|y - 0.01x^2|) + 0.01*|x + 10|
/// Global minimum is 0 at (-10, 1), domain x ∈ [-15, -5], y ∈ [-3, 3]
pub struct BukinN6Algorithm;

impl BenchmarkAlgorithm for BukinN6Algorithm {
    fn name(&self) -> &'static str {
        "bukin_n6"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-15.0, -5.0),
            RangeInclusive::new(-3.0, 3.0),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(BukinN6)
    }
}

/// Levi N.13 function - multimodal test function
/// f(x, y) = sin^2(3πx) + (x-1)^2[1 + sin^2(3πy)] + (y-1)^2[1 + sin^2(2πy)]
/// Global minimum is 0 at (1, 1), domain x, y ∈ [-10, 10]
pub struct LeviN13Algorithm;

impl BenchmarkAlgorithm for LeviN13Algorithm {
    fn name(&self) -> &'static str {
        "levi_n13"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-10.0, 10.0),
            RangeInclusive::new(-10.0, 10.0),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(LeviN13)
    }
}

/// Rastrigin function - standard multimodal benchmark
/// f(x, y) = 20 + (x^2 - 10 cos(2πx)) + (y^2 - 10 cos(2πy))
/// Global minimum at (0,0) with f = 0.0, domain x, y ∈ [-5.12, 5.12]
pub struct RastriginAlgorithm;

impl BenchmarkAlgorithm for RastriginAlgorithm {
    fn name(&self) -> &'static str {
        "rastrigin"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-5.12, 5.12),
            RangeInclusive::new(-5.12, 5.12),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(Rastrigin)
    }
}

/// Schaffer N.2 function - multimodal with smooth areas and ripples
/// f(x, y) = 0.5 + (sin^2(x^2 - y^2) - 0.5) / (1 + 0.001(x^2 + y^2))^2
/// Global minimum is 0 at (0, 0), domain x, y ∈ [-100, 100]
pub struct SchafferN2Algorithm;

impl BenchmarkAlgorithm for SchafferN2Algorithm {
    fn name(&self) -> &'static str {
        "schaffer_n2"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-100.0, 100.0),
            RangeInclusive::new(-100.0, 100.0),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(SchafferN2)
    }
}

/// Styblinski–Tang function - multimodal with many local minima
/// f(x,y) = (x^4 - 16x^2 + 5x)/2 + (y^4 - 16y^2 + 5y)/2, shifted so min is 0
/// Global minimum at (-2.903534, -2.903534), domain x, y ∈ [-5, 5]
pub struct StyblinskiTangAlgorithm;

impl BenchmarkAlgorithm for StyblinskiTangAlgorithm {
    fn name(&self) -> &'static str {
        "styblinski_tang"
    }

    fn param_ranges(&self) -> Vec<RangeInclusive<f64>> {
        vec![
            RangeInclusive::new(-5.0, 5.0),
            RangeInclusive::new(-5.0, 5.0),
        ]
    }

    fn function(&self) -> Box<dyn WorldFunction> {
        Box::new(StyblinskiTang)
    }
}
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

#[derive(Debug)]
struct BukinN6;

impl SingleValuedFunction for BukinN6 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let term1 = 100.0 * (y - 0.01 * x * x).abs().sqrt();
        let term2 = 0.01 * (x + 10.0).abs();
        term1 + term2
    }
}

#[derive(Debug)]
struct LeviN13;

impl SingleValuedFunction for LeviN13 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let pi = std::f64::consts::PI;
        let term1 = (3.0 * pi * x).sin().powi(2);
        let term2 = (x - 1.0).powi(2) * (1.0 + (3.0 * pi * y).sin().powi(2));
        let term3 = (y - 1.0).powi(2) * (1.0 + (2.0 * pi * y).sin().powi(2));
        term1 + term2 + term3
    }
}

#[derive(Debug)]
struct Rastrigin;

impl SingleValuedFunction for Rastrigin {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let two_pi = 2.0 * std::f64::consts::PI;
        20.0 + (x * x - 10.0 * (two_pi * x).cos()) + (y * y - 10.0 * (two_pi * y).cos())
    }
}

#[derive(Debug)]
struct SchafferN2;

impl SingleValuedFunction for SchafferN2 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let x2_plus_y2 = x * x + y * y;
        let x2_minus_y2 = x * x - y * y;
        let numerator = x2_minus_y2.sin().powi(2) - 0.5;
        let denominator = (1.0 + 0.001 * x2_plus_y2).powi(2);
        0.5 + numerator / denominator
    }
}

#[derive(Debug)]
struct StyblinskiTang;

impl SingleValuedFunction for StyblinskiTang {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let term_x = (x.powi(4) - 16.0 * x.powi(2) + 5.0 * x) / 2.0;
        let term_y = (y.powi(4) - 16.0 * y.powi(2) + 5.0 * y) / 2.0;
        let original_value = term_x + term_y;
        // Shift so global min is 0 (original min for 2D is about -78.33233)
        original_value + 78.33233
    }
}

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
