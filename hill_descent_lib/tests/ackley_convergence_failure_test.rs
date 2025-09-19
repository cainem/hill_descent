use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Ackley function (2D) implementation for testing convergence failure
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
#[ignore] // This test demonstrates proper seeded convergence and should be run explicitly
pub fn given_ackley_with_small_population_when_properly_seeded_then_converges() {
    // Test that Ackley function converges with proper seeding:
    // Population: 100, Regions: 10, Seed: 2
    let param_range = vec![
        RangeInclusive::new(-5.0, 5.0),
        RangeInclusive::new(-5.0, 5.0),
    ];

    // Use one of the prime seeds that failed to converge (seed 2)
    let seed = 2;
    let global_constants = GlobalConstants::new_with_seed(100, 10, seed);

    let mut world = setup_world(&param_range, global_constants, Box::new(Ackley));

    let mut best_score = f64::MAX;
    let mut hit_resolution_limit = false;
    let max_rounds = 1000;

    println!("Starting Ackley convergence failure test:");
    println!(
        "Population: 100, Regions: 10, Seed: 2, Max rounds: {}",
        max_rounds
    );

    // Run the optimization and track progress
    for round in 0..max_rounds {
        let at_resolution_limit = world.training_run(&[], &[]);
        let current_best = world.get_best_score();

        if current_best < best_score {
            best_score = current_best;
            println!("Round {}: New best score: {:.6e}", round + 1, best_score);
        }

        if at_resolution_limit {
            hit_resolution_limit = true;
            println!("Resolution limit reached at round {}", round + 1);
            break;
        }

        // Print periodic updates
        if (round + 1) % 100 == 0 {
            println!("Round {}: Best score so far: {:.6e}", round + 1, best_score);
        }
    }

    println!("Final results:");
    println!("Hit resolution limit: {}", hit_resolution_limit);
    println!("Final best score: {:.6e}", best_score);
    println!(
        "Rounds completed: {}",
        if hit_resolution_limit {
            "< 1000"
        } else {
            "1000"
        }
    );

    // With proper seeding, the algorithm should converge for most reasonable seeds
    // This test verifies that the Ackley function can be optimized successfully
    assert!(
        hit_resolution_limit,
        "Expected Ackley with seed 2 to converge, but it failed"
    );
    assert!(
        best_score < 1e-100,
        "Expected very small final score, got {:.6e}",
        best_score
    );
}
