use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hill_descent_lib2::{GlobalConstants, SingleValuedFunction, TrainingData};
use hill_descent_lib2::world::World;
use std::ops::RangeInclusive;

/// An expensive fitness function that simulates real-world computation.
/// Each evaluation takes ~50-100ms by doing heavy floating point work.
#[derive(Debug, Clone)]
struct ExpensiveFitness {
    /// Number of iterations to simulate expensive computation
    iterations: usize,
}

impl ExpensiveFitness {
    fn new(iterations: usize) -> Self {
        Self { iterations }
    }
}

impl SingleValuedFunction for ExpensiveFitness {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // Base fitness: sum of squares
        let mut result: f64 = phenotype_expressed_values.iter().map(|v| v * v).sum();
        
        // Simulate expensive computation (e.g., neural network forward pass, 
        // physics simulation, etc.)
        for i in 0..self.iterations {
            for val in phenotype_expressed_values {
                // Heavy trig operations that can't be easily optimized away
                result += (val * (i as f64 * 0.001)).sin().abs() * 0.0001;
                result += (val * (i as f64 * 0.001)).cos().abs() * 0.0001;
                result += (result * 0.0000001).tanh();
            }
        }
        
        result
    }
}

/// Benchmark different thread counts with an expensive fitness function
fn expensive_thread_count_benchmark(c: &mut Criterion) {
    // Smaller population since each evaluation is expensive
    // 10 dimensions, 50 organisms
    let param_range = vec![RangeInclusive::new(-10.0, 10.0); 10];
    let global_constants = GlobalConstants::new(50, 10);
    
    // Calibrate iterations to get ~50-100ms per organism evaluation
    // With 50 organisms, we want the epoch to take a few seconds total
    // so we can measure thread scaling
    let iterations = 50_000; // Adjust this to get desired time per evaluation

    let thread_counts = [12, 24, 50, 75, 100];

    let mut group = c.benchmark_group("expensive_fitness");
    
    // More samples for better accuracy
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(30));

    for &threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("threads", threads),
            &threads,
            |b, &thread_count| {
                let mut world = World::new_with_thread_count(
                    &param_range,
                    global_constants,
                    Box::new(ExpensiveFitness::new(iterations)),
                    thread_count,
                );

                b.iter(|| {
                    world.training_run(TrainingData::None { floor_value: 0.0 });
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, expensive_thread_count_benchmark);
criterion_main!(benches);
