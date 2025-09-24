mod algorithms;
mod output;
mod runner;

use algorithms::{
    AckleyAlgorithm, BenchmarkAlgorithm, BukinN6Algorithm, HimmelblauAlgorithm, LeviN13Algorithm,
    RastriginAlgorithm, SchafferN2Algorithm, StyblinskiTangAlgorithm,
};
use output::write_results_to_file;
use runner::benchmark_algorithm;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hill Descent Algorithm Benchmarking Tool");
    println!("========================================");
    println!();

    // Create run_stats directory
    let run_stats_dir = Path::new("run_stats");

    // List of algorithms to benchmark
    let algorithms: Vec<Box<dyn BenchmarkAlgorithm>> = vec![
        Box::new(AckleyAlgorithm),
        Box::new(HimmelblauAlgorithm),
        Box::new(BukinN6Algorithm),
        Box::new(LeviN13Algorithm),
        Box::new(RastriginAlgorithm),
        Box::new(SchafferN2Algorithm),
        Box::new(StyblinskiTangAlgorithm),
    ];

    // Run benchmarks for each algorithm
    for algorithm in algorithms.iter() {
        println!("Running benchmarks for {} algorithm...", algorithm.name());
        println!(
            "Configurations to test: {:?}",
            runner::POPULATION_REGION_CONFIGS
        );
        println!("Runs per configuration: {}", runner::RUNS_PER_CONFIG);
        println!("Maximum rounds per run: {}", runner::MAX_ROUNDS);
        println!();

        let results = benchmark_algorithm(algorithm.as_ref());

        // Print summary to console
        println!("Results for {}:", algorithm.name());
        for config in &results.configurations {
            println!(
                "  Pop: {}, Regions: {}, Avg Rounds: {:.1}, Resolution Hits: {}/{}, Best Score: {:.6}, Avg Time: {:.3}s",
                config.population,
                config.regions,
                config.average_rounds(),
                config.resolution_limit_hits(),
                runner::RUNS_PER_CONFIG,
                config.best_score(),
                config.average_time_secs()
            );
        }
        println!();

        // Write results to file
        match write_results_to_file(&results, run_stats_dir) {
            Ok(()) => println!("✓ Results successfully written to file"),
            Err(e) => eprintln!("✗ Error writing results: {}", e),
        }
        println!();
    }

    println!("Benchmarking completed!");
    Ok(())
}
