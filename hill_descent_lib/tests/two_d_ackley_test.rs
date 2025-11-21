use hill_descent_lib::{
    GlobalConstants, TrainingData, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Ackley function (2D) is a widely-used multimodal benchmark for optimization.
// f(x, y) = -20 * exp(-0.2 * sqrt(0.5 * (x² + y²))) - exp(0.5 * (cos(2πx) + cos(2πy))) + e + 20
// Global minimum at (0,0) with f = 0.0.
// Characterized by nearly flat outer region and central peak with many local minima.
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

#[test]
#[ignore]
pub fn execute() {
    // Typical Ackley search domain is [-5, 5] for each dimension.
    let param_range = vec![
        RangeInclusive::new(-5.0, 5.0),
        RangeInclusive::new(-5.0, 5.0),
    ];

    // Use larger population due to Ackley's challenging landscape with many local minima
    let global_constants = GlobalConstants::new(500, 10);

    let mut world = setup_world(&param_range, global_constants, Box::new(Ackley));

    let mut best_score = f64::MAX;

    // Allow sufficient epochs to navigate the complex multimodal landscape
    for epoch in 0..3000 {
        let at_resolution_limit = world.training_run(TrainingData::None {
            floor_value: Ackley.function_floor(),
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
