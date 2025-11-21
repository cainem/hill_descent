//! Simple 2D optimization example demonstrating basic hill_descent_lib usage.
//!
//! This example optimizes the Sphere function (sum of squares), which has a
//! single global minimum at the origin (0, 0) with a value of 0.
//!
//! Run with:
//! ```
//! cargo run --example simple_optimization
//! ```

use hill_descent_lib::{GlobalConstants, SingleValuedFunction, TrainingData, setup_world};

/// Sphere function: f(x, y) = x² + y²
///
/// This is one of the simplest optimization problems with a single global
/// minimum at (0, 0).
#[derive(Debug)]
struct Sphere;

impl SingleValuedFunction for Sphere {
    fn single_run(&self, params: &[f64]) -> f64 {
        params.iter().map(|x| x * x).sum()
    }
}

fn main() {
    println!("=== Hill Descent Simple Optimization Example ===\n");
    println!("Optimizing Sphere function: f(x, y) = x² + y²");
    println!("Global minimum: (0, 0) with value 0\n");

    // Define the search space: [-10, 10] for both x and y
    let param_range = vec![-10.0..=10.0; 2];

    // Configure the genetic algorithm
    // - Population size: 200 organisms
    // - Target regions: 20 spatial partitions
    let constants = GlobalConstants::new(200, 20);

    // Create the optimization world
    let mut world = setup_world(&param_range, constants, Box::new(Sphere));

    println!("Configuration:");
    println!("  Population size: 200");
    println!("  Target regions: 20");
    println!("  Parameter range: [-10.0, 10.0] for each dimension\n");

    // Run optimization for 100 epochs
    println!("Running optimization...\n");
    let epochs = 100;

    for epoch in 0..epochs {
        // For SingleValuedFunction, use TrainingData::None
        world.training_run(TrainingData::None { floor_value: 0.0 });

        // Print progress every 20 epochs
        if (epoch + 1) % 20 == 0 {
            let best_score = world.get_best_score();
            println!("Epoch {:3}: Best score = {:.6}", epoch + 1, best_score);
        }
    }

    // Extract final results
    let best_score = world.get_best_score();

    println!("\n=== Results ===");
    println!("Best solution found:");
    println!("  f(x, y) = {:.6}", best_score);

    // Check if we found a good solution
    if best_score < 0.01 {
        println!("\n✓ Successfully converged to near-optimal solution!");
    } else {
        println!("\n⚠ Solution quality could be improved - try more epochs or larger population");
    }
}
