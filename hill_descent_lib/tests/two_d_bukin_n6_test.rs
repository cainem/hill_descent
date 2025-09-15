use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Bukin function N.6 is a challenging optimization test function with a narrow curved valley.
// It has asymmetric bounds and is known for its difficulty due to the valley's shape.
// f(x, y) = 100√|y - 0.01x²| + 0.01|x + 10|
// The global minimum is 0 at (-10, 1).
// Domain: x ∈ [-15, -5], y ∈ [-3, 3]
#[derive(Debug)]
struct BukinN6;

impl SingleValuedFunction for BukinN6 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // This function is 2-dimensional.
        assert_eq!(2, phenotype_expressed_values.len());

        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        // f(x, y) = 100√|y - 0.01x²| + 0.01|x + 10|
        let term1 = 100.0 * (y - 0.01 * x * x).abs().sqrt();
        let term2 = 0.01 * (x + 10.0).abs();

        term1 + term2
    }
}

#[test]
#[ignore] // This test is long-running and should be run explicitly.
pub fn execute() {
    // #[cfg(feature = "enable-tracing")]
    // hill_descent_lib::init_tracing();

    // Bukin N.6 has asymmetric domain: x ∈ [-15, -5], y ∈ [-3, 3]
    let param_range = vec![
        RangeInclusive::new(-15.0, -5.0),
        RangeInclusive::new(-3.0, 3.0),
    ];
    let global_constants = GlobalConstants::new(500, 10); // Larger population for 2D search

    let mut world = setup_world(&param_range, global_constants, Box::new(BukinN6));

    let mut best_score = f64::MAX;

    // Run for a number of epochs to allow the system to find the minimum.
    for i in 0..2000 {
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
    // Bukin N.6 is extremely challenging due to its narrow curved valley.
    // A tolerance of 0.1 is reasonable for this function.
    assert!(
        best_score < 0.1,
        "Final score {best_score} was not close enough to 0.0"
    );
}
