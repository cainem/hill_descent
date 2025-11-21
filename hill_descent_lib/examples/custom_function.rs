//! Example demonstrating how to implement a custom optimization function.
//!
//! This example shows different optimization functions and how to implement them
//! using the SingleValuedFunction trait.
//!
//! Run with:
//! ```
//! cargo run --example custom_function
//! ```

use hill_descent_lib::{GlobalConstants, SingleValuedFunction, TrainingData, setup_world};

/// Rosenbrock function: f(x, y) = (1 - x)² + 100(y - x²)²
///
/// Also known as the "banana function" due to its curved valley shape.
/// Global minimum at (1, 1) with value 0.
///
/// This function is challenging because:
/// - The global minimum lies in a narrow, parabolic valley
/// - The function is non-convex
/// - It has a very steep descent around the minimum
#[derive(Debug)]
struct Rosenbrock;

impl SingleValuedFunction for Rosenbrock {
    fn single_run(&self, params: &[f64]) -> f64 {
        let x = params[0];
        let y = params[1];
        (1.0 - x).powi(2) + 100.0 * (y - x.powi(2)).powi(2)
    }
}

/// Himmelblau's function: f(x, y) = (x² + y - 11)² + (x + y² - 7)²
///
/// This function has four identical local minima, making it interesting for
/// testing whether the algorithm can find multiple solutions.
///
/// Global minima at:
/// - (3.0, 2.0)
/// - (-2.805118, 3.131312)
/// - (-3.779310, -3.283186)
/// - (3.584428, -1.848126)
/// All with value 0.
#[derive(Debug)]
struct Himmelblau;

impl SingleValuedFunction for Himmelblau {
    fn single_run(&self, params: &[f64]) -> f64 {
        let x = params[0];
        let y = params[1];
        (x.powi(2) + y - 11.0).powi(2) + (x + y.powi(2) - 7.0).powi(2)
    }
}

fn run_rosenbrock_example() {
    println!("\n{}", "=".repeat(60));
    println!("Optimizing: Rosenbrock (banana function)");
    println!("{}\n", "=".repeat(60));

    let param_range = vec![-5.0..=5.0; 2];
    let constants = GlobalConstants::new(500, 50);
    let mut world = setup_world(&param_range, constants, Box::new(Rosenbrock));

    let epochs = 1000;
    println!("Running {} epochs...", epochs);

    for epoch in 0..epochs {
        world.training_run(TrainingData::None { floor_value: 0.0 });

        // Print progress every 200 epochs
        if (epoch + 1) % 200 == 0 || epoch == 0 {
            let best_score = world.get_best_score();
            println!("  Epoch {:4}: Best score = {:.6}", epoch + 1, best_score);
        }
    }

    // Extract results
    let best_score = world.get_best_score();

    println!("\nResults:");
    println!("  Best fitness: {:.6}", best_score);
    println!("  Expected optimal: (1.0, 1.0)");

    if best_score < 0.01 {
        println!("  ✓ Excellent convergence!");
    } else if best_score < 1.0 {
        println!("  ✓ Good convergence");
    } else {
        println!("  ⚠ Moderate convergence - try more epochs");
    }
}

fn run_himmelblau_example() {
    println!("\n{}", "=".repeat(60));
    println!("Optimizing: Himmelblau (four minima)");
    println!("{}\n", "=".repeat(60));

    let param_range = vec![-5.0..=5.0; 2];
    let constants = GlobalConstants::new(500, 50);
    let mut world = setup_world(&param_range, constants, Box::new(Himmelblau));

    let epochs = 1000;
    println!("Running {} epochs...", epochs);

    for epoch in 0..epochs {
        world.training_run(TrainingData::None { floor_value: 0.0 });

        if (epoch + 1) % 200 == 0 || epoch == 0 {
            let best_score = world.get_best_score();
            println!("  Epoch {:4}: Best score = {:.6}", epoch + 1, best_score);
        }
    }

    let best_score = world.get_best_score();

    println!("\nResults:");
    println!("  Best fitness: {:.6}", best_score);
    println!("  Known minima: (3.0, 2.0), (-2.805, 3.131), (-3.779, -3.283), (3.584, -1.848)");

    if best_score < 0.01 {
        println!("  ✓ Excellent convergence!");
    } else if best_score < 1.0 {
        println!("  ✓ Good convergence");
    } else {
        println!("  ⚠ Moderate convergence - try more epochs");
    }
}

fn main() {
    println!("=== Hill Descent Custom Function Examples ===");
    println!("\nThis example demonstrates implementing custom optimization functions");
    println!("and optimizing different problem types.\n");

    run_rosenbrock_example();
    run_himmelblau_example();

    println!("\n{}", "=".repeat(60));
    println!("Tip: Try implementing your own function by:");
    println!("  1. Create a struct for your function");
    println!("  2. Implement SingleValuedFunction trait");
    println!("  3. Define your fitness calculation in single_run()");
    println!("{}\n", "=".repeat(60));
}
