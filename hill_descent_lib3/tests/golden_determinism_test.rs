//! Golden determinism test.
//!
//! This locks down the exact best score/params for a fixed configuration.
//! It runs the training inside a single-thread Rayon pool so the execution order
//! is deterministic.
//!
//! Note: The fill-to-capacity reproduction strategy with dimension subdivision
//! produces different convergence characteristics than earlier versions.

use hill_descent_lib3::{GlobalConstants, SingleValuedFunction, TrainingData, setup_world};
use rayon::ThreadPoolBuilder;
use std::ops::RangeInclusive;

/// Sphere function: f(x) = Σxᵢ²
/// Global minimum is 0 at origin.
#[derive(Debug)]
struct Sphere;

impl SingleValuedFunction for Sphere {
    fn single_run(&self, params: &[f64]) -> f64 {
        params.iter().map(|x| x * x).sum()
    }

    fn function_floor(&self) -> f64 {
        0.0
    }
}

#[test]
fn given_fixed_seed_when_training_run_in_single_thread_pool_then_best_is_golden() {
    // Updated golden values after adding dimension subdivision and fill-to-capacity reproduction
    const EXPECTED_BEST_SCORE: f64 = 0.0015707753011852058;
    const FLOAT_TOLERANCE: f64 = 1e-10;

    let pool = ThreadPoolBuilder::new().num_threads(1).build().unwrap();

    pool.install(|| {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];

        let seed = 456;
        let population_size = 100;
        let regions = 10;
        let constants = GlobalConstants::new_with_seed(population_size, regions, seed);

        let mut world = setup_world(&bounds, constants, Box::new(Sphere));

        let epochs = 50;
        for _ in 0..epochs {
            world.training_run(TrainingData::None { floor_value: 0.0 });
        }

        let final_score = world.get_best_score();
        let best_params = world.get_best_params();

        // Print for debugging if test fails
        println!("Final score: {final_score:?}");
        println!("Best params: {best_params:?}");
        println!("Best id: {:?}", world.get_best_organism_id());

        assert!(
            (final_score - EXPECTED_BEST_SCORE).abs() <= FLOAT_TOLERANCE,
            "best_score mismatch: got {final_score:?}, expected {EXPECTED_BEST_SCORE:?}"
        );
    });
}
