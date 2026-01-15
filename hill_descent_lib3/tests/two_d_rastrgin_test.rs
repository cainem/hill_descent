use hill_descent_lib3::{
    GlobalConstants, TrainingData, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Rastrigin function (2D) is a standard multimodal benchmark for optimization.
// f(x, y) = 20 + (x^2 - 10 cos(2πx)) + (y^2 - 10 cos(2πy))
// Global minimum at (0,0) with f = 0.0.
// Many regularly spaced local minima make it a good test of exploration.
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

#[test]
#[ignore]
pub fn execute() {
    // Typical Rastrigin search domain is [-5.12, 5.12] for each dimension.
    let param_range = vec![
        RangeInclusive::new(-5.12, 5.12),
        RangeInclusive::new(-5.12, 5.12),
    ];

    // Population & regions tuned similarly to the Himmelblau test; adjust if convergence slow.
    let global_constants = GlobalConstants::new(500, 10);

    let mut world = setup_world(&param_range, global_constants, Box::new(Rastrigin));

    let mut best_score = f64::MAX;

    // Allow sufficient epochs to traverse multiple local minima basins.
    for epoch in 0..3000 {
        let at_resolution_limit = world.training_run(TrainingData::None {
            floor_value: Rastrigin.function_floor(),
        });
        let current_best = world.get_best_score();
        if current_best < best_score {
            best_score = current_best;
        }
        if at_resolution_limit {
            println!("Resolution limit reached at epoch {epoch}");
            break;
        }
        if epoch % 100 == 0 {
            println!("Epoch {epoch}: Best score so far: {best_score}");
        }
    }

    println!("Final best score: {best_score}");
    assert!(
        best_score < 0.01,
        "Final score {best_score} was not close enough to 0.0"
    );
}
