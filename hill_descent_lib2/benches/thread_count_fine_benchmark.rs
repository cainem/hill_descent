use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hill_descent_lib2::world::World;
use hill_descent_lib2::{GlobalConstants, SingleValuedFunction, TrainingData};
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
struct SumSquares;

impl SingleValuedFunction for SumSquares {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        phenotype_expressed_values.iter().map(|v| v * v).sum()
    }
}

/// Fine-grained benchmark around physical core count
fn thread_count_fine_benchmark(c: &mut Criterion) {
    let param_range = vec![RangeInclusive::new(-100.0, 100.0); 100];
    let global_constants = GlobalConstants::new(500, 20);

    // Test around physical core count (12) in detail
    let thread_counts = [4, 6, 8, 10, 11, 12, 13, 14, 16];

    let mut group = c.benchmark_group("thread_count_fine");

    // More samples for better accuracy
    group.sample_size(50);

    for &threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("threads", threads),
            &threads,
            |b, &thread_count| {
                let mut world = World::new_with_thread_count(
                    &param_range,
                    global_constants,
                    Box::new(SumSquares),
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

criterion_group!(benches, thread_count_fine_benchmark);
criterion_main!(benches);
