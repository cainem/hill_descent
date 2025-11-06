use criterion::{Criterion, criterion_group, criterion_main};
use hill_descent::{
    parameters::GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
    TrainingData,
};
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
struct SumSquares;

impl SingleValuedFunction for SumSquares {
    /// Returns the sum of squares of the expressed phenotype values.
    /// This is intentionally simple so that the benchmark focuses on
    /// the performance of the hill descent algorithm rather than the
    /// cost of evaluating the function itself.
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        phenotype_expressed_values.iter().map(|v| v * v).sum()
    }
}

/// Benchmarks a training epoch of the hill-descent algorithm when optimising a
/// 100-dimensional function. The benchmark mirrors the structure of
/// `simple_test.rs` and `two_d_simple_test.rs`, but scales the problem up to
/// 100 parameters.
fn hill_descent_100d_benchmark(c: &mut Criterion) {
    // Parameter bounds: [-100, 100] for each of the 100 parameters
    let param_range = vec![RangeInclusive::new(-100.0, 100.0); 100];

    // Some reasonable global constants for a higher-dimensional search.
    // These mimic the style used in the existing tests.
    let global_constants = GlobalConstants::new(2000, 1000);

    // Build the world once; during the benchmark we will repeatedly call
    // `training_run` to measure the performance of an epoch.
    let mut world = setup_world(&param_range, global_constants, Box::new(SumSquares));

    c.bench_function("hill_descent_train_epoch_100d", |b| {
        b.iter(|| {
            // Objective-function mode: no external data needed
            world.training_run(TrainingData::None { floor_value: 0.0 });
        })
    });
}

criterion_group!(benches, hill_descent_100d_benchmark);
criterion_main!(benches);

