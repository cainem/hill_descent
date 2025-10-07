use crate::git_info::{get_git_info, get_hash_prefix_for_directory};
use crate::runner::AlgorithmResults;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Generate the full directory path and filename for algorithm results
/// Returns (year_month_subdir, hash_day_subdir, filename)
/// Example: ("2025-10", "a1b2c3d4-07", "20251007_143020_rastrigin.md")
pub fn generate_subdir_and_filename(algorithm_name: &str) -> (String, String, String) {
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    let hash_prefix = get_hash_prefix_for_directory();
    let day = now.format("%d").to_string();
    let hash_day_subdir = format!("{}-{}", hash_prefix, day);
    let filename = format!("{}_{}.md", now.format("%Y%m%d_%H%M%S"), algorithm_name);
    (year_month, hash_day_subdir, filename)
}

/// Write algorithm results to a markdown file in the run_stats directory
pub fn write_results_to_file(
    results: &AlgorithmResults,
    run_stats_dir: &Path,
) -> Result<(), std::io::Error> {
    // Determine year-month subdirectory, hash-day subdirectory, and filename
    let (year_month, hash_day_subdir, filename) =
        generate_subdir_and_filename(&results.algorithm_name);
    let full_subdir_path = run_stats_dir.join(&year_month).join(&hash_day_subdir);
    fs::create_dir_all(&full_subdir_path)?;
    let filepath = full_subdir_path.join(filename);
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

    // Add git information if available
    if let Some(git_info) = get_git_info() {
        writeln!(file, "- **Git Commit:** {}", git_info.commit_hash)?;
        writeln!(file, "- **Git Branch:** {}", git_info.branch)?;
    } else {
        writeln!(
            file,
            "- **Git Info:** Not available (not in a git repository)"
        )?;
    }

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
    writeln!(file, "| Runs | Population | Regions | Total Rounds | Total Resolution Hits | Best Score | Avg Score | Std Dev | Avg Time (s) |")?;
    writeln!(file, "|------|------------|---------|--------------|----------------------|------------|-----------|---------|--------------|")?;

    for config in &results.configurations {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {:.6e} | {:.6e} | {:.6e} | {:.3} |",
            crate::runner::RUNS_PER_CONFIG,
            config.population,
            config.regions,
            crate::runner::MAX_ROUNDS,
            config.resolution_limit_hits(),
            config.best_score(),
            config.average_best_score(),
            config.std_dev_best_score(),
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
            "| Run | Seed | Rounds | Resolution Hits | Score | Time (s) |"
        )?;
        writeln!(
            file,
            "|-----|------|--------|-----------------|-------|----------|"
        )?;

        for (run_idx, run) in config.runs.iter().enumerate() {
            let seed = crate::runner::PRIME_SEEDS[run_idx % crate::runner::PRIME_SEEDS.len()];
            writeln!(
                file,
                "| {} | {} | {} | {} | {:.6e} | {:.3} |",
                run_idx + 1,
                seed,
                run.rounds_taken,
                run.resolution_limit_count,
                run.best_score,
                run.duration_secs
            )?;
        }
        writeln!(file)?;
    }

    println!("Results written to: {}", filepath.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_algorithm_name_when_generate_subdirs_then_returns_valid_format() {
        let (year_month, hash_day, filename) = generate_subdir_and_filename("test_algorithm");

        // Year-month should be in YYYY-MM format
        assert_eq!(year_month.len(), 7);
        assert!(year_month.contains('-'));

        // Hash-day should be in format: <8hexchars>-DD or nogit-DD
        let parts: Vec<&str> = hash_day.split('-').collect();
        assert_eq!(parts.len(), 2);

        // First part should be 8 hex chars or "nogit"
        if parts[0] == "nogit" {
            assert_eq!(parts[0], "nogit");
        } else {
            assert_eq!(parts[0].len(), 8);
            assert!(parts[0].chars().all(|c| c.is_ascii_hexdigit()));
        }

        // Second part should be DD (day)
        assert_eq!(parts[1].len(), 2);
        let day: u32 = parts[1].parse().unwrap();
        assert!((1..=31).contains(&day));

        // Filename should contain timestamp and algorithm name
        assert!(filename.contains("test_algorithm"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn given_different_algorithm_names_when_generate_filename_then_includes_name() {
        let (_, _, filename1) = generate_subdir_and_filename("ackley");
        let (_, _, filename2) = generate_subdir_and_filename("rastrigin");

        assert!(filename1.contains("ackley"));
        assert!(filename2.contains("rastrigin"));
        assert_ne!(filename1, filename2);
    }

    #[test]
    fn given_year_month_format_when_parsed_then_valid() {
        let (year_month, _, _) = generate_subdir_and_filename("test");

        let parts: Vec<&str> = year_month.split('-').collect();
        assert_eq!(parts.len(), 2);

        // Year should be 4 digits
        assert_eq!(parts[0].len(), 4);
        assert!(parts[0].parse::<u32>().is_ok());

        // Month should be 2 digits
        assert_eq!(parts[1].len(), 2);
        let month: u32 = parts[1].parse().unwrap();
        assert!((1..=12).contains(&month));
    }

    #[test]
    fn given_filename_when_parsed_then_contains_timestamp() {
        let (_, _, filename) = generate_subdir_and_filename("test");

        // Remove the .md extension and algorithm name
        let parts: Vec<&str> = filename.split('_').collect();

        // First part should be YYYYMMDD
        assert_eq!(parts[0].len(), 8);
        assert!(parts[0].parse::<u32>().is_ok());

        // Second part should be HHMMSS
        assert_eq!(parts[1].len(), 6);
        assert!(parts[1].parse::<u32>().is_ok());
    }

    #[test]
    fn given_hash_day_subdir_when_parsed_then_valid_format() {
        let (_, hash_day, _) = generate_subdir_and_filename("test");

        let parts: Vec<&str> = hash_day.split('-').collect();
        assert_eq!(parts.len(), 2, "Hash-day should have format: <hash>-DD");

        // Day should be 2 digits
        assert_eq!(parts[1].len(), 2);
        let day: u32 = parts[1].parse().unwrap();
        assert!((1..=31).contains(&day));
    }
}
