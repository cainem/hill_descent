# Hill Descent

[![Crates.io](https://img.shields.io/crates/v/hill_descent_lib.svg)](https://crates.io/crates/hill_descent_lib)
[![Documentation](https://docs.rs/hill_descent_lib/badge.svg)](https://docs.rs/hill_descent_lib)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A Rust-based genetic algorithm optimization system for solving n-dimensional optimization problems through evolutionary computation.

## Repository Structure

This is a Cargo workspace containing three main crates:

### 📚 [hill_descent_lib](hill_descent_lib/)

[![Crates.io](https://img.shields.io/crates/v/hill_descent_lib.svg)](https://crates.io/crates/hill_descent_lib)

The core genetic algorithm library, published on crates.io.

**Key Features:**
- N-dimensional optimization (2D to 100D+)
- Spatial partitioning with adaptive regions
- Sexual reproduction with genetic crossover
- Parallel fitness evaluation
- Deterministic results with seeded RNG
- Minimal public API for ease of use

**Installation:**
```bash
cargo add hill_descent_lib
```

**Quick Example:**
```rust
use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};

#[derive(Debug)]
struct Sphere;

impl SingleValuedFunction for Sphere {
    fn single_run(&self, params: &[f64]) -> f64 {
        params.iter().map(|x| x * x).sum()
    }
}

let param_range = vec![-10.0..=10.0; 2];
let constants = GlobalConstants::new(200, 20);
let mut world = setup_world(&param_range, constants, Box::new(Sphere));

for _ in 0..100 {
    world.training_run(&[], &[0.0]);
}

println!("Best score: {}", world.get_best_score());
```

[📖 Full Documentation](https://docs.rs/hill_descent_lib)

---

### 🌐 [hill_descent_server](hill_descent_server/)

A web-based visualization server for 2D optimization problems.

**Features:**
- Real-time visualization of the optimization process
- Interactive web interface showing organism distribution
- Region boundaries and fitness landscapes
- Built with Actix-web

**Usage:**
```bash
cd hill_descent_server
cargo run
```

Then open your browser to `http://localhost:8080`

The server provides endpoints for:
- `/api/init` - Initialize optimization with parameters
- `/api/step` - Run one optimization epoch
- Web interface for visualization

---

### 📊 [hill_descent_benchmarks](hill_descent_benchmarks/)

Comprehensive benchmarking tool for testing algorithm performance across different optimization functions.

**Features:**
- Multiple benchmark functions (Ackley, Himmelblau, Rastrigin, etc.)
- Configurable parameters (population size, epochs, regions)
- Statistical analysis (mean, std dev, convergence rates)
- Git integration for tracking performance across commits
- Markdown output with detailed results

**Usage:**
```bash
cd hill_descent_benchmarks
cargo run
```

Results are saved to `run_stats/YYYY-MM/` with detailed performance metrics.

**Supported Functions:**
- Styblinski-Tang
- Ackley
- Himmelblau
- Bukin N6
- Levi N13
- Rastrigin
- Schaffer N2

---

## Development

### Prerequisites
- Rust 2024 edition or later
- Cargo workspace support

### Building All Crates
```bash
cargo build --workspace
```

### Running Tests
```bash
cargo test --workspace
```

### Running Clippy
```bash
cargo clippy --workspace
```

### Running Examples
```bash
# Core library examples
cargo run --example simple_optimization --package hill_descent_lib
cargo run --example custom_function --package hill_descent_lib
cargo run --example multi_dimensional --package hill_descent_lib

# Web visualization
cd hill_descent_server && cargo run

# Benchmarking
cd hill_descent_benchmarks && cargo run
```

---

## Project Documentation

- **[AGENTS.md](AGENTS.md)** - Comprehensive development guidelines and project standards
- **[hill_descent_lib/pdd.md](hill_descent_lib/pdd.md)** - Product Definition Document with domain model
- **[hill_descent_server/web/web_pdd.md](hill_descent_server/web/web_pdd.md)** - Web interface specification

---

## Algorithm Overview

Hill Descent uses a spatial genetic algorithm with:

1. **Spatial Partitioning**: Parameter space is divided into adaptive regions
2. **Region-Based Evolution**: Each region maintains its own population
3. **Sexual Reproduction**: Organisms reproduce via genetic crossover
4. **Adaptive Mutation**: Self-adjusting mutation rates in genetic material
5. **Carrying Capacity**: Fitness-based resource allocation across regions
6. **Parallel Evaluation**: Concurrent fitness computation for performance

The algorithm excels at:
- Finding global optima in complex fitness landscapes
- Handling high-dimensional problems (100+ parameters)
- Balancing exploration vs exploitation
- Providing deterministic, reproducible results

---

## Performance Characteristics

| Dimensions | Population | Regions | Typical Epochs |
| ---------- | ---------- | ------- | -------------- |
| 2D         | 200        | 20      | 100-500        |
| 5D         | 400        | 40      | 500-1000       |
| 10D        | 800        | 80      | 1000-2000      |
| 50D        | 4000       | 400     | 5000-10000     |

Performance scales approximately linearly with population size and number of CPU cores available for parallel evaluation.

---

## Use Cases

- **Function Optimization**: Finding minima/maxima of mathematical functions
- **Hyperparameter Tuning**: Optimizing ML model parameters
- **Engineering Design**: Parameter optimization in simulations
- **Research**: Genetic algorithm experimentation and visualization

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](hill_descent_lib/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](hill_descent_lib/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

## Links

- **Crates.io**: https://crates.io/crates/hill_descent_lib
- **Documentation**: https://docs.rs/hill_descent_lib
- **Repository**: https://github.com/cainem/hill_descent
- **Issues**: https://github.com/cainem/hill_descent/issues

---

## Version History

### v0.1.0 (2025-10-28)
- Initial release to crates.io
- Core genetic algorithm implementation
- Comprehensive documentation and examples
- Web visualization server
- Benchmarking tools
