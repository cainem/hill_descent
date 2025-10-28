# hill_descent_lib

[![Crates.io](https://img.shields.io/crates/v/hill_descent_lib.svg)](https://crates.io/crates/hill_descent_lib)
[![Documentation](https://docs.rs/hill_descent_lib/badge.svg)](https://docs.rs/hill_descent_lib)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/cainem/hill_descent#license)

A Rust genetic algorithm library for n-dimensional optimization problems.

## Quick Start

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
use std::ops::RangeInclusive;

// Define your fitness function (lower scores are better)
#[derive(Debug)]
struct Quadratic;

impl SingleValuedFunction for Quadratic {
    fn single_run(&self, params: &[f64]) -> f64 {
        // Minimize: x² + y²
        // Optimal solution: (0, 0) with score 0
        params[0].powi(2) + params[1].powi(2)
    }
}

fn main() {
    // Define parameter bounds
    let bounds = vec![-10.0..=10.0, -10.0..=10.0];
    
    // Configure the algorithm (population size, number of regions)
    let constants = GlobalConstants::new(100, 10);
    
    // Create the optimization world
    let mut world = setup_world(&bounds, constants, Box::new(Quadratic));
    
    // Run optimization for 100 generations
    for _ in 0..100 {
        world.training_run(&[], &[0.0]);
    }
    
    println!("Best score: {}", world.get_best_score());
}
```

## Features

- **N-dimensional optimization** - Handles any number of parameters
- **Spatial regions** - Divides search space into adaptive regions for efficient exploration
- **Genetic algorithm** - Uses crossover, mutation, and selection for evolution
- **Deterministic** - Seeded RNG ensures reproducible results
- **Zero-copy parallelism** - Leverages Rayon for efficient multi-core processing
- **Flexible fitness functions** - Easy-to-implement trait for custom optimization problems
- **Optional tracing** - Built-in logging support for debugging (feature: `enable-tracing`)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hill_descent_lib = "0.1.0"
```

## Usage Examples

### Basic 2D Optimization

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
use std::ops::RangeInclusive;

#[derive(Debug)]
struct Himmelblau;

impl SingleValuedFunction for Himmelblau {
    fn single_run(&self, params: &[f64]) -> f64 {
        let x = params[0];
        let y = params[1];
        // Himmelblau's function has four identical local minima
        (x.powi(2) + y - 11.0).powi(2) + (x + y.powi(2) - 7.0).powi(2)
    }
}

fn main() {
    let bounds = vec![-5.0..=5.0, -5.0..=5.0];
    let constants = GlobalConstants::new(500, 10);
    let mut world = setup_world(&bounds, constants, Box::new(Himmelblau));
    
    for generation in 0..1000 {
        world.training_run(&[], &[0.0]);
        
        if generation % 100 == 0 {
            println!("Generation {}: Best score = {}", generation, world.get_best_score());
        }
    }
}
```

### Higher Dimensional Problems

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
use std::ops::RangeInclusive;

#[derive(Debug)]
struct NDimSphere;

impl SingleValuedFunction for NDimSphere {
    fn single_run(&self, params: &[f64]) -> f64 {
        // Sphere function: sum of squares
        params.iter().map(|&x| x * x).sum()
    }
}

fn main() {
    // Optimize in 20 dimensions
    let bounds = vec![-10.0..=10.0; 20];
    let constants = GlobalConstants::new(1000, 50);
    let mut world = setup_world(&bounds, constants, Box::new(NDimSphere));
    
    for _ in 0..500 {
        world.training_run(&[], &[0.0]);
    }
    
    println!("Best score: {}", world.get_best_score());
}
```

### Custom Function Floor

By default, the algorithm assumes the optimal fitness is 0. For functions with different optima:

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
use std::ops::RangeInclusive;

#[derive(Debug)]
struct CustomFloor;

impl SingleValuedFunction for CustomFloor {
    fn single_run(&self, params: &[f64]) -> f64 {
        // Your function that has a minimum of -100
        -100.0 + params[0].powi(2)
    }
    
    fn function_floor(&self) -> f64 {
        -100.0  // Tell the algorithm the expected minimum
    }
}
```

### Using the Tracing Feature

For debugging and analysis, enable the optional tracing feature:

```toml
[dependencies]
hill_descent_lib = { version = "0.1.0", features = ["enable-tracing"] }
```

```rust
use hill_descent_lib::{init_tracing, GlobalConstants, SingleValuedFunction, setup_world};

fn main() {
    // Initialize tracing (only available with feature enabled)
    #[cfg(feature = "enable-tracing")]
    init_tracing();
    
    // Your optimization code...
}
```

## How It Works

The library implements a genetic algorithm with spatial partitioning:

1. **Initialization** - Creates a population of random organisms within specified bounds
2. **Regions** - Divides the search space into adaptive regions based on organism distribution
3. **Evaluation** - Each organism is scored using your fitness function
4. **Selection** - Better-performing organisms in each region get more reproduction opportunities
5. **Reproduction** - Sexual reproduction with crossover and mutation creates offspring
6. **Adaptation** - Regions dynamically adjust based on population distribution and fitness landscape

The algorithm automatically manages:
- Region boundaries and subdivision
- Carrying capacity per region based on fitness
- Organism aging and death
- Mutation rates and adjustment parameters
- Search space expansion when needed

## API Overview

### Core Types

- **`setup_world()`** - Initialize the optimization environment
- **`GlobalConstants`** - Configuration (population size, region count, seed)
- **`World`** - Main optimization container with methods:
  - `training_run()` - Run one generation
  - `get_best_score()` - Get current best fitness
  - `get_best_organism()` - Get the best organism
  - `get_state()` - Get JSON representation of world state

### Traits to Implement

- **`SingleValuedFunction`** - Define your fitness function
  - `single_run(&self, params: &[f64]) -> f64` - Required
  - `function_floor(&self) -> f64` - Optional (default: 0.0)

## Performance Characteristics

- **Parallelism**: Region processing uses Rayon for multi-core efficiency
- **Memory**: Allocates based on population size and dimensionality
- **Determinism**: Seeded RNG ensures reproducible runs with same configuration
- **Scalability**: Tested with 100+ dimensions and populations of 10,000+

## Common Use Cases

- Parameter optimization for machine learning models
- Neural network weight initialization and tuning
- Function minimization/maximization problems
- Engineering design optimization
- Scientific computing and simulation tuning
- Hyperparameter search

## Comparison to Other Libraries

Unlike gradient-based optimizers, hill_descent_lib:
- ✅ Doesn't require differentiable functions
- ✅ Handles discrete and continuous parameters
- ✅ Explores multiple regions simultaneously
- ✅ Less likely to get stuck in local minima
- ❌ Generally requires more function evaluations
- ❌ Not as fast for smooth, convex problems

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/hill_descent_lib).

## Examples

See the `examples/` directory for more usage patterns:
- `simple_optimization.rs` - Basic 2D example
- `custom_function.rs` - Implementing custom fitness functions
- `multi_dimensional.rs` - High-dimensional optimization

Run examples with:
```bash
cargo run --example simple_optimization
```

## Minimum Supported Rust Version (MSRV)

Rust 2024 edition (Rust 1.82.0+)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

Developed on Windows with cross-platform compatibility in mind.

## Repository

Source code: [https://github.com/cainem/hill_descent](https://github.com/cainem/hill_descent)
