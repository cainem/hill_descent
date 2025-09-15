use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Lévi function N.13 is a multimodal test function for optimization algorithms.
// It has a complex surface with many local minima but one global minimum.
// f(x, y) = sin²(3πx) + (x-1)²[1 + sin²(3πy)] + (y-1)²[1 + sin²(2πy)]
// The global minimum is 0 at (1, 1).
#[derive(Debug)]
struct LeviN13;

impl SingleValuedFunction for LeviN13 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // This function is 2-dimensional.
        assert_eq!(2, phenotype_expressed_values.len());

        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        let pi = std::f64::consts::PI;

        // f(x, y) = sin²(3πx) + (x-1)²[1 + sin²(3πy)] + (y-1)²[1 + sin²(2πy)]
        let term1 = (3.0 * pi * x).sin().powi(2);
        let term2 = (x - 1.0).powi(2) * (1.0 + (3.0 * pi * y).sin().powi(2));
        let term3 = (y - 1.0).powi(2) * (1.0 + (2.0 * pi * y).sin().powi(2));

        term1 + term2 + term3
    }
}

#[test]
#[ignore] // This test is long-running and should be run explicitly.
pub fn execute() {
    // #[cfg(feature = "enable-tracing")]
    // hill_descent_lib::init_tracing();

    // Lévi N.13 is typically evaluated on [-10, 10] for both variables
    let param_range = vec![
        RangeInclusive::new(-10.0, 10.0),
        RangeInclusive::new(-10.0, 10.0),
    ];
    let global_constants = GlobalConstants::new(500, 10); // Larger population for 2D search

    let mut world = setup_world(&param_range, global_constants, Box::new(LeviN13));

    let mut best_score = f64::MAX;

    // Run for a number of epochs to allow the system to find the minimum.
    for i in 0..3000 {
        // Objective-function mode: no known outputs
        let at_resolution_limit = world.training_run(&[], &[]);

        // Get the current best score from organisms
        let current_best = world.get_best_score();

        if current_best < best_score {
            best_score = current_best;
        }

        // Break early if we've reached the resolution limit
        if at_resolution_limit {
            println!("Resolution limit reached at epoch {i}");
            break;
        }

        if i % 100 == 0 {
            println!("Epoch {i}: Best score so far: {best_score}");
        }
    }

    println!("Final best score: {best_score}");

    // The goal is to get the score very close to the global minimum of 0.
    // A tolerance of 0.01 should be achievable for this function.
    assert!(
        best_score < 0.01,
        "Final score {best_score} was not close enough to 0.0"
    );
}
