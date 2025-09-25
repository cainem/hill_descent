use crate::algorithms::BenchmarkAlgorithm;
use hill_descent_lib::{setup_world, GlobalConstants};
use std::time::Instant;

// Configuration constants
pub const POPULATION_REGION_CONFIGS: &[(u32, u32)] = &[
    (100, 10),
    (250, 10),
    (500, 20),
    (750, 50),
    (1000, 100),
    (10_000, 1_000),
];

// Prime seeds for reproducible runs
pub const PRIME_SEEDS: &[u64] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
];

pub const MAX_ROUNDS: u32 = 1000;
pub const RUNS_PER_CONFIG: usize = 20;

/// Results from a single benchmark run
#[derive(Debug)]
pub struct SingleRunResult {
    pub rounds_taken: u32,
    pub resolution_limit_count: u32,
    pub best_score: f64,
    pub duration_secs: f64,
}

/// Aggregated results for a configuration across multiple runs
#[derive(Debug)]
pub struct ConfigurationResults {
    pub population: u32,
    pub regions: u32,
    pub runs: Vec<SingleRunResult>,
}

impl ConfigurationResults {
    pub fn new(population: u32, regions: u32) -> Self {
        Self {
            population,
            regions,
            runs: Vec::new(),
        }
    }

    /// Calculate average rounds taken across all runs
    pub fn average_rounds(&self) -> f64 {
        let total: u32 = self.runs.iter().map(|r| r.rounds_taken).sum();
        total as f64 / self.runs.len() as f64
    }

    /// Calculate total resolution limit hits across all runs
    pub fn resolution_limit_hits(&self) -> u32 {
        self.runs.iter().map(|r| r.resolution_limit_count).sum()
    }

    /// Calculate average time taken across all runs
    pub fn average_time_secs(&self) -> f64 {
        let total: f64 = self.runs.iter().map(|r| r.duration_secs).sum();
        total / self.runs.len() as f64
    }

    /// Get the best score achieved across all runs
    pub fn best_score(&self) -> f64 {
        self.runs
            .iter()
            .map(|r| r.best_score)
            .fold(f64::INFINITY, f64::min)
    }
}

/// Results for all configurations of a single algorithm
#[derive(Debug)]
pub struct AlgorithmResults {
    pub algorithm_name: String,
    pub configurations: Vec<ConfigurationResults>,
}

/// Run benchmarks for a single algorithm across all configurations
pub fn benchmark_algorithm(algorithm: &dyn BenchmarkAlgorithm) -> AlgorithmResults {
    let mut algorithm_results = AlgorithmResults {
        algorithm_name: algorithm.name().to_string(),
        configurations: Vec::new(),
    };

    for &(population, regions) in POPULATION_REGION_CONFIGS {
        let mut config_results = ConfigurationResults::new(population, regions);

        for i in 0..RUNS_PER_CONFIG {
            let seed = PRIME_SEEDS[i % PRIME_SEEDS.len()];
            let run_result = run_single_benchmark(algorithm, population, regions, seed);
            config_results.runs.push(run_result);
        }

        algorithm_results.configurations.push(config_results);
    }

    algorithm_results
}

/// Run a single benchmark with specific parameters
fn run_single_benchmark(
    algorithm: &dyn BenchmarkAlgorithm,
    population: u32,
    regions: u32,
    seed: u64,
) -> SingleRunResult {
    let param_ranges = algorithm.param_ranges();
    let function = algorithm.function();
    let global_constants =
        GlobalConstants::new_with_seed(population as usize, regions as usize, seed);

    let mut world = setup_world(&param_ranges, global_constants, function);
    let start_time = Instant::now();

    let rounds_taken = MAX_ROUNDS;
    let mut resolution_limit_count = 0u32;

    // Run the optimization for all MAX_ROUNDS
    for _round in 0..MAX_ROUNDS {
        if _round == 564 {
            // put breakpoint here to stop at round problem exists.
            resolution_limit_count += 1;
            resolution_limit_count -= 1;
        }

        // Run training and count resolution limit hits
        if world.training_run(&[], None) {
            resolution_limit_count += 1;
        }
    }

    let duration = start_time.elapsed();
    let best_score = world.get_best_score();

    SingleRunResult {
        rounds_taken,
        resolution_limit_count,
        best_score,
        duration_secs: duration.as_secs_f64(),
    }
}
