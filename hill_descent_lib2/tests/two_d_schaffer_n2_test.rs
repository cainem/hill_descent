use hill_descent_lib2::{GlobalConstants, SingleValuedFunction, TrainingData, World};
use std::ops::RangeInclusive;

// Schaffer function N.2 is a multimodal test function with smooth areas and ripples.
// It has a complex surface that tests an algorithm's ability to navigate
// between smooth regions and areas with high-frequency oscillations.
// f(x, y) = 0.5 + (sin²(x² - y²) - 0.5) / (1 + 0.001(x² + y²))²
// The global minimum is 0 at (0, 0).
#[derive(Debug)]
struct SchafferN2;

impl SingleValuedFunction for SchafferN2 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // This function is 2-dimensional.
        assert_eq!(2, phenotype_expressed_values.len());

        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        // f(x, y) = 0.5 + (sin²(x² - y²) - 0.5) / (1 + 0.001(x² + y²))²
        let x2_plus_y2 = x * x + y * y;
        let x2_minus_y2 = x * x - y * y;

        let numerator = x2_minus_y2.sin().powi(2) - 0.5;
        let denominator = (1.0 + 0.001 * x2_plus_y2).powi(2);

        0.5 + numerator / denominator
    }
}

#[test]
#[ignore] // This test is long-running and should be run explicitly.
pub fn execute() {
    // Schaffer N.2 is typically evaluated on [-100, 100] for both variables
    let param_range = vec![
        RangeInclusive::new(-100.0, 100.0),
        RangeInclusive::new(-100.0, 100.0),
    ];
    let global_constants = GlobalConstants::new(500, 10); // Larger population for 2D search

    let mut world = World::new(&param_range, global_constants, Box::new(SchafferN2));

    let mut best_score = f64::MAX;

    // Run for a number of epochs to allow the system to find the minimum.
    for i in 0..3000 {
        // Objective-function mode: use TrainingData::None with function floor
        let at_resolution_limit = world.training_run(TrainingData::None {
            floor_value: SchafferN2.function_floor(),
        });

        // Get the current best score from organisms
        let current_best = world.get_best_score();

        if current_best < best_score {
            best_score = current_best;
        }

        // Note: at_resolution_limit in lib2 just means dimensions didn't change,
        // not that we've reached maximum precision. We don't break early.
        let _ = at_resolution_limit;

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
