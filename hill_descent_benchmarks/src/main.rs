mod algorithms;
mod git_info;
mod output;
mod runner;

use algorithms::{
    AckleyAlgorithm, BenchmarkAlgorithm, BukinN6Algorithm, HimmelblauAlgorithm, LeviN13Algorithm,
    RastriginAlgorithm, SchafferN2Algorithm, StyblinskiTangAlgorithm,
};
use git_info::get_hash_prefix_for_directory;
use output::write_results_to_file;
use runner::benchmark_algorithm;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the current run directory path (YYYY-MM/<hashprefix>-DD)
fn get_current_run_directory(run_stats_dir: &Path) -> PathBuf {
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let hash_prefix = get_hash_prefix_for_directory();
    let day = now.format("%d").to_string();
    let hash_day_subdir = format!("{}-{}", hash_prefix, day);

    run_stats_dir.join(year_month).join(hash_day_subdir)
}

/// Clean the current run directory if it exists
fn clean_run_directory(run_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if run_dir.exists() {
        println!("Cleaning existing run directory: {}", run_dir.display());
        fs::remove_dir_all(run_dir)?;
        println!("✓ Directory cleaned");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hill Descent Algorithm Benchmarking Tool");
    println!("========================================");
    println!();

    // Create run_stats directory
    let run_stats_dir = Path::new("run_stats");

    // Clean the current run directory before starting
    let current_run_dir = get_current_run_directory(run_stats_dir);
    clean_run_directory(&current_run_dir)?;
    println!();

    // List of algorithms to benchmark
    let algorithms: Vec<Box<dyn BenchmarkAlgorithm>> = vec![
        Box::new(StyblinskiTangAlgorithm),
        Box::new(AckleyAlgorithm),
        Box::new(HimmelblauAlgorithm),
        Box::new(BukinN6Algorithm),
        Box::new(LeviN13Algorithm),
        Box::new(RastriginAlgorithm),
        Box::new(SchafferN2Algorithm),
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
                "  Pop: {}, Regions: {}, Total Rounds: {:.0}, Total Resolution Hits: {}, Best Score: {:.6}, Avg Score: {:.6}, Std Dev: {:.6}, Avg Time: {:.3}s",
                config.population,
                config.regions,
                config.average_rounds(),
                config.resolution_limit_hits(),
                config.best_score(),
                config.average_best_score(),
                config.std_dev_best_score(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn given_run_stats_dir_when_get_current_run_directory_then_valid_path() {
        let run_stats_dir = Path::new("run_stats");
        let run_dir = get_current_run_directory(run_stats_dir);

        // Path should start with run_stats
        assert!(run_dir.starts_with("run_stats"));

        // Should have 3 components: run_stats, YYYY-MM, <hash>-DD
        let components: Vec<_> = run_dir.components().collect();
        assert_eq!(components.len(), 3);
    }

    #[test]
    fn given_run_directory_path_when_formatted_then_contains_hash_and_day() {
        let run_stats_dir = Path::new("run_stats");
        let run_dir = get_current_run_directory(run_stats_dir);

        let path_str = run_dir.to_string_lossy();

        // Should contain the year-month directory
        let now = chrono::Local::now();
        let year_month = now.format("%Y-%m").to_string();
        assert!(path_str.contains(&year_month));

        // Should contain the day
        let day = now.format("%d").to_string();
        assert!(path_str.contains(&day));
    }

    #[test]
    fn given_nonexistent_directory_when_clean_run_directory_then_no_error() {
        let temp_dir = Path::new("test_temp_nonexistent");

        // Should not error if directory doesn't exist
        let result = clean_run_directory(temp_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn given_existing_directory_when_clean_run_directory_then_removed() {
        let temp_dir = Path::new("test_temp_cleanup");

        // Create a temporary directory with a file
        fs::create_dir_all(temp_dir).unwrap();
        fs::write(temp_dir.join("test.txt"), "test content").unwrap();

        // Verify it exists
        assert!(temp_dir.exists());

        // Clean it
        let result = clean_run_directory(temp_dir);
        assert!(result.is_ok());

        // Verify it's removed
        assert!(!temp_dir.exists());
    }
}
