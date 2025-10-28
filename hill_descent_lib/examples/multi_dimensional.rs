//! Example demonstrating high-dimensional optimization.
//!
//! This example shows how the library scales to problems with many parameters,
//! using the Rastrigin function which becomes increasingly difficult with more dimensions.
//!
//! Run with:
//! ```
//! cargo run --example multi_dimensional
//! ```

use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
use std::f64::consts::PI;

/// Rastrigin function in n dimensions.
///
/// This is a highly multimodal function with many local minima,
/// making it a challenging optimization problem. The global minimum
/// is at the origin with value 0.
///
/// Formula: f(x) = 10n + Σ(x_i² - 10cos(2πx_i))
#[derive(Debug)]
struct Rastrigin {
    dimensions: usize,
}

impl Rastrigin {
    fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl SingleValuedFunction for Rastrigin {
    fn single_run(&self, params: &[f64]) -> f64 {
        let n = self.dimensions as f64;
        10.0 * n
            + params
                .iter()
                .map(|&x| x.powi(2) - 10.0 * (2.0 * PI * x).cos())
                .sum::<f64>()
    }
}

/// Sphere function in n dimensions (simpler baseline).
///
/// This is the sum of squares: f(x) = Σ(x_i²)
/// Global minimum at origin with value 0.
#[derive(Debug)]
#[allow(dead_code)]
struct Sphere;

impl SingleValuedFunction for Sphere {
    fn single_run(&self, params: &[f64]) -> f64 {
        params.iter().map(|x| x * x).sum()
    }
}

fn optimize_dimension(dimensions: usize, epochs: usize, population_size: usize) {
    println!("\n{}", "=".repeat(70));
    println!("Optimizing {}-dimensional Rastrigin function", dimensions);
    println!("{}", "=".repeat(70));

    // Rastrigin typically uses [-5.12, 5.12] range
    let param_range = vec![-5.12..=5.12; dimensions];

    // Scale regions with population
    let target_regions = population_size / 10;
    let constants = GlobalConstants::new(population_size, target_regions);

    let function = Box::new(Rastrigin::new(dimensions));
    let mut world = setup_world(&param_range, constants, function);

    println!("Configuration:");
    println!("  Dimensions: {}", dimensions);
    println!("  Population: {}", population_size);
    println!("  Regions: {}", target_regions);
    println!("  Epochs: {}\n", epochs);

    println!("Running optimization...");

    let report_interval = epochs / 5;
    for epoch in 0..epochs {
        world.training_run(&[], &[0.0]);

        if (epoch + 1) % report_interval == 0 || epoch == 0 {
            let best_score = world.get_best_score();
            println!("  Epoch {:5}: Best score = {:.6}", epoch + 1, best_score);
        }
    }

    // Results
    let best_score = world.get_best_score();

    println!("\nResults:");
    println!("  Best fitness: {:.6}", best_score);

    // Quality assessment
    let quality = if best_score < dimensions as f64 * 0.1 {
        "✓ Excellent - very close to global optimum"
    } else if best_score < dimensions as f64 * 0.5 {
        "✓ Good - near global optimum region"
    } else if best_score < dimensions as f64 * 2.0 {
        "○ Moderate - found a reasonable solution"
    } else {
        "⚠ Fair - may need more epochs or larger population"
    };

    println!("  Quality: {}", quality);
}

fn main() {
    println!("=== Hill Descent Multi-Dimensional Optimization Example ===");
    println!("\nThis example demonstrates how the library scales to high-dimensional");
    println!("optimization problems using the challenging Rastrigin function.\n");

    // Test with increasing dimensions
    optimize_dimension(2, 500, 200);
    optimize_dimension(5, 800, 400);
    optimize_dimension(10, 1000, 800);

    // Optional: Uncomment for even higher dimensions (takes longer)
    // optimize_dimension(20, 2000, 1600);
    // optimize_dimension(50, 5000, 4000);

    println!("\n{}", "=".repeat(70));
    println!("Key Insights:");
    println!("  • Higher dimensions require larger populations");
    println!("  • Target regions scale with population (typically 10% ratio)");
    println!("  • More dimensions need more epochs for convergence");
    println!("  • Rastrigin is particularly challenging due to many local minima");
    println!("{}", "=".repeat(70));

    println!("\n{}", "=".repeat(70));
    println!("Try it yourself:");
    println!("  • Uncomment the 20D and 50D examples above");
    println!("  • Experiment with different population sizes");
    println!("  • Compare Rastrigin vs Sphere (swap function in code)");
    println!("{}\n", "=".repeat(70));
}
