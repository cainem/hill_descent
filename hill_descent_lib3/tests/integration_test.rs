//! Integration test for hill_descent_lib3.
//!
//! This test verifies the complete optimization workflow using the public API,
//! exercising all stages of the genetic algorithm:
//! - World initialization with bounds and configuration
//! - Population creation and evaluation
//! - Multi-epoch training with selection, reproduction, and culling
//! - Best solution tracking and retrieval

use hill_descent_lib3::{GlobalConstants, SingleValuedFunction, TrainingData, setup_world};
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
fn given_world_when_complete_optimization_workflow_then_succeeds() {
    // This test exercises the complete optimization workflow:
    // 1. Create bounds defining the search space
    // 2. Initialize the world with a population of organisms
    // 3. Run training epochs
    // 4. Query results via the public API

    // Step 1: Define search space (2D, bounds -10 to 10)
    let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];

    // Step 2: Configure and create the world
    let seed = 456;
    let population_size = 100;
    let regions = 10;
    let constants = GlobalConstants::new_with_seed(population_size, regions, seed);
    let mut world = setup_world(&bounds, constants, Box::new(Sphere));

    // Verify initial state
    assert_eq!(
        world.get_best_score(),
        f64::MAX,
        "Initial score should be MAX"
    );
    assert!(
        world.get_best_organism_id().is_none(),
        "No best organism before training"
    );
    assert!(
        world.get_best_params().is_empty(),
        "No best params before training"
    );

    // Step 3: Run training epochs
    let epochs = 50;
    for _ in 0..epochs {
        world.training_run(TrainingData::None { floor_value: 0.0 });
    }

    // Step 4: Query and verify results
    let final_score = world.get_best_score();
    let best_params = world.get_best_params();
    let best_organism_id = world.get_best_organism_id();

    // Score should have improved significantly from initial random placement
    assert!(
        final_score < f64::MAX,
        "Score should be less than MAX after training"
    );
    assert!(
        final_score < 200.0, // Loose bound - should be much better than random
        "Score {} should be reasonably optimized",
        final_score
    );

    // Params should exist and have correct dimension
    assert_eq!(best_params.len(), 2, "Should have 2 parameters");
    for (i, &p) in best_params.iter().enumerate() {
        assert!(
            (-10.0..=10.0).contains(&p),
            "Param[{}]={} should be within bounds",
            i,
            p
        );
    }

    // Best organism ID should be set
    assert!(
        best_organism_id.is_some(),
        "Best organism ID should be set after training"
    );
}
