use crate::runner::AlgorithmResults;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Generate a timestamped filename for the algorithm results
/// Generate a year-month subdirectory and timestamped filename for the algorithm results
pub fn generate_subdir_and_filename(algorithm_name: &str) -> (String, String) {
    let now = chrono::Local::now();
    let subdir = now.format("%Y-%m").to_string();
    let filename = format!("{}_{}.md", now.format("%Y%m%d_%H%M%S"), algorithm_name);
    (subdir, filename)
}

/// Write algorithm results to a markdown file in the run_stats directory
pub fn write_results_to_file(
    results: &AlgorithmResults,
    run_stats_dir: &Path,
) -> Result<(), std::io::Error> {
    // Determine year-month subdirectory and filename
    let (subdir, filename) = generate_subdir_and_filename(&results.algorithm_name);
    let subdir_path = run_stats_dir.join(subdir);
    fs::create_dir_all(&subdir_path)?;
    let filepath = subdir_path.join(filename);
    let mut file = fs::File::create(&filepath)?;

    // Write header with timestamp and algorithm info
    let now = chrono::Local::now();
    writeln!(
        file,
        "# {} Algorithm Benchmark Results",
        results.algorithm_name.to_uppercase()
    )?;
    writeln!(file)?;
    writeln!(file, "**Run Started:** {}", now.format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(file, "**Algorithm:** {}", results.algorithm_name)?;
    writeln!(file)?;

    // Write configuration summary
    writeln!(file, "## Configuration")?;
    writeln!(
        file,
        "- **Runs per configuration:** {}",
        crate::runner::RUNS_PER_CONFIG
    )?;
    writeln!(file, "- **Maximum rounds:** {}", crate::runner::MAX_ROUNDS)?;
    writeln!(
        file,
        "- **Seeds used:** {:?}",
        &crate::runner::PRIME_SEEDS[..crate::runner::RUNS_PER_CONFIG]
    )?;
    writeln!(file)?;

    // Write results table
    writeln!(file, "## Results")?;
    writeln!(file)?;
    writeln!(file, "| Runs | Population | Regions | Max Rounds | Avg Rounds | Hit Resolution Limit | Best Score | Avg Time (s) |")?;
    writeln!(file, "|------|------------|---------|------------|------------|---------------------|------------|--------------|")?;

    for config in &results.configurations {
        writeln!(
            file,
            "| {} | {} | {} | {} | {:.1} | {} | {:.6e} | {:.3} |",
            crate::runner::RUNS_PER_CONFIG,
            config.population,
            config.regions,
            crate::runner::MAX_ROUNDS,
            config.average_rounds(),
            config.resolution_limit_hits(),
            config.best_score(),
            config.average_time_secs()
        )?;
    }

    writeln!(file)?;

    // Write detailed run information
    writeln!(file, "## Detailed Run Information")?;
    writeln!(file)?;

    for (config_idx, config) in results.configurations.iter().enumerate() {
        writeln!(
            file,
            "### Configuration {} (Pop: {}, Regions: {})",
            config_idx + 1,
            config.population,
            config.regions
        )?;
        writeln!(file)?;
        writeln!(
            file,
            "| Run | Seed | Rounds | Hit Limit | Score | Time (s) |"
        )?;
        writeln!(
            file,
            "|-----|------|--------|-----------|-------|----------|"
        )?;

        for (run_idx, run) in config.runs.iter().enumerate() {
            let seed = crate::runner::PRIME_SEEDS[run_idx % crate::runner::PRIME_SEEDS.len()];
            writeln!(
                file,
                "| {} | {} | {} | {} | {:.6e} | {:.3} |",
                run_idx + 1,
                seed,
                run.rounds_taken,
                if run.hit_resolution_limit {
                    "✓"
                } else {
                    "✗"
                },
                run.best_score,
                run.duration_secs
            )?;
        }
        writeln!(file)?;
    }

    println!("Results written to: {}", filepath.display());
    Ok(())
}
