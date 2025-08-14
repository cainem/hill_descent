use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Himmelblau's function is a standard test function for optimization algorithms.
// It has four identical local minima, making it a good test for an algorithm's
// ability to find one of several optimal solutions.
// f(x, y) = (x^2 + y - 11)^2 + (x + y^2 - 7)^2
// The global minimum is 0.
#[derive(Debug)]
struct Himmelblau;

impl SingleValuedFunction for Himmelblau {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // This function is 2-dimensional.
        assert_eq!(2, phenotype_expressed_values.len());

        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];

        // f(x, y) = (x^2 + y - 11)^2 + (x + y^2 - 7)^2
        let term1 = (x.powi(2) + y - 11.0).powi(2);
        let term2 = (x + y.powi(2) - 7.0).powi(2);
        term1 + term2
    }
}

#[test]
#[ignore] // This test is long-running and should be run explicitly.
pub fn execute() {
    // #[cfg(feature = "enable-tracing")]
    // hill_descent_lib::init_tracing();

    // The four minima are within the range [-5.0, 5.0] for both x and y.
    let param_range = vec![
        RangeInclusive::new(-25000000.0, -5000000.0),
        RangeInclusive::new(-25000000.0, -5000000.0),
    ];
    let global_constants = GlobalConstants::new(100, 10); // Larger population for 2D search

    let mut world = setup_world(&param_range, global_constants, Box::new(Himmelblau));

    //println!("Test initial state:\n{}\n", world.get_state());

    let mut best_score = f64::MAX;

    // Run for a number of epochs to allow the system to find a minimum.
    for i in 0..1100 {
        if i == 727 {
            #[cfg(feature = "enable-tracing")]
            hill_descent_lib::init_tracing();
        }

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
            //break;
        }

        //if i % 100 == 0 {
        println!("Epoch {i}: Best score so far: {best_score}");
        //    println!("{}\n\n", world.get_state());
        //}
    }

    //println!("Final state:\n{}\n", world.get_state());
    println!("Final best score: {best_score}");

    // The goal is to get the score very close to the global minimum of 0.
    // A tolerance of 0.01 should be achievable.
    assert!(
        best_score < 0.01,
        "Final score {best_score} was not close enough to 0.0"
    );
}
