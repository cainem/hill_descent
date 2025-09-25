use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Styblinski–Tang function is a multimodal test function with many local minima.
// Original formula: f(x,y) = (x⁴ - 16x² + 5x)/2 + (y⁴ - 16y² + 5y)/2
// The original 1D global minimum is approximately -39.16616570377142 at x ≈ -2.903534.
// For 2D the global minimum is twice that. We shift exactly so the minimum is 0.0.
#[derive(Debug)]
struct StyblinskiTang;

impl SingleValuedFunction for StyblinskiTang {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // This function is 2-dimensional.
        assert_eq!(2, phenotype_expressed_values.len());

        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        const ST_1D_MIN: f64 = -39.166_165_703_771_42;
        let term_x = (x.powi(4) - 16.0 * x.powi(2) + 5.0 * x) / 2.0;
        let term_y = (y.powi(4) - 16.0 * y.powi(2) + 5.0 * y) / 2.0;
        let original_value = term_x + term_y;
        let shifted = original_value - 2.0 * ST_1D_MIN; // exact shift
        debug_assert!(
            shifted >= -1e-9,
            "Shifted Styblinski-Tang value unexpectedly negative: {shifted}"
        );
        if shifted < 0.0 { 0.0 } else { shifted }
    }
}

#[test]
fn given_styblinski_tang_when_evaluated_at_global_min_then_returns_zero() {
    let st = StyblinskiTang;
    let v = st.single_run(&[-2.903534, -2.903534]);
    assert!(v.abs() < 1e-6, "Expected ~0 at global minimum, got {v}");
}

#[test]
#[ignore] // This test is long-running and should be run explicitly.
pub fn execute() {
    // #[cfg(feature = "enable-tracing")]
    // hill_descent_lib::init_tracing();

    // Styblinski–Tang is typically evaluated on [-5, 5] for both variables
    let param_range = vec![
        RangeInclusive::new(-5.0, 5.0),
        RangeInclusive::new(-5.0, 5.0),
    ];
    let global_constants = GlobalConstants::new(500, 10); // Larger population for 2D search

    let mut world = setup_world(&param_range, global_constants, Box::new(StyblinskiTang));

    let mut best_score = f64::MAX;

    // Run for a number of epochs to allow the system to find the minimum.
    for i in 0..3000 {
        // Objective-function mode: no known outputs
        let at_resolution_limit = world.training_run(&[], None);

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
