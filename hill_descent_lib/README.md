# hill_descent_lib

[![Crates.io](https://img.shields.io/crates/v/hill_descent_lib.svg)](https://crates.io/crates/hill_descent_lib)
[![Documentation](https://docs.rs/hill_descent_lib/badge.svg)](https://docs.rs/hill_descent_lib)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/cainem/hill_descent#license)

A Rust genetic algorithm library for n-dimensional optimization problems.

## Quick Start

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world, TrainingData};
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
        world.training_run(TrainingData::None { floor_value: 0.0 });
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
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world, TrainingData};
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
        world.training_run(TrainingData::None { floor_value: 0.0 });
        
        if generation % 100 == 0 {
            println!("Generation {}: Best score = {}", generation, world.get_best_score());
        }
    }
}
```


### Higher Dimensional Problems

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world, TrainingData};
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
        world.training_run(TrainingData::None { floor_value: 0.0 });
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

### Machine Learning / Neural Network Optimization

For large-scale parameter optimization (e.g., neural network weights), hill_descent_lib
excels at finding good initializations or tuning thousands of parameters:

```rust
use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world, format_score, TrainingData};
use std::ops::RangeInclusive;

// Simulate a neural network with training data managed internally
#[derive(Debug)]
struct NeuralNetOptimizer {
    // Your training data stored here
    train_inputs: Vec<Vec<f64>>,
    train_labels: Vec<f64>,
    // Network structure: e.g., [input_size, hidden1, hidden2, output_size]
    layer_sizes: Vec<usize>,
}

impl NeuralNetOptimizer {
    fn new(train_inputs: Vec<Vec<f64>>, train_labels: Vec<f64>) -> Self {
        Self {
            train_inputs,
            train_labels,
            layer_sizes: vec![10, 50, 50, 1], // 10 inputs, 2x50 hidden, 1 output
        }
    }
    
    fn param_count(&self) -> usize {
        // Calculate total weights + biases
        // weights: (10*50) + (50*50) + (50*1) = 500 + 2500 + 50 = 3050
        // biases: 50 + 50 + 1 = 101
        // Total: 3151 parameters
        self.layer_sizes.windows(2)
            .map(|w| w[0] * w[1] + w[1]) // weights + biases per layer
            .sum()
    }
    
    fn forward(&self, inputs: &[f64], weights: &[f64]) -> f64 {
        // Simplified forward pass (implement your actual network logic)
        // This is just an example - use your real neural network implementation
        let mut activation = inputs.to_vec();
        let mut weight_idx = 0;
        
        for layer_idx in 0..self.layer_sizes.len() - 1 {
            let in_size = self.layer_sizes[layer_idx];
            let out_size = self.layer_sizes[layer_idx + 1];
            let mut next_activation = vec![0.0; out_size];
            
            // Apply weights
            for o in 0..out_size {
                let mut sum = 0.0;
                for i in 0..in_size {
                    sum += activation[i] * weights[weight_idx];
                    weight_idx += 1;
                }
                // Add bias
                sum += weights[weight_idx];
                weight_idx += 1;
                
                // ReLU activation (except last layer)
                next_activation[o] = if layer_idx < self.layer_sizes.len() - 2 {
                    sum.max(0.0)
                } else {
                    sum // Linear output for regression
                };
            }
            activation = next_activation;
        }
        
        activation[0] // Return single output
    }
    
    fn mse_loss(&self, weights: &[f64]) -> f64 {
        // Calculate mean squared error over all training examples
        let mut total_error = 0.0;
        for (inputs, &label) in self.train_inputs.iter().zip(&self.train_labels) {
            let prediction = self.forward(inputs, weights);
            let error = prediction - label;
            total_error += error * error;
        }
        total_error / self.train_inputs.len() as f64
    }
}

impl SingleValuedFunction for NeuralNetOptimizer {
    fn single_run(&self, params: &[f64]) -> f64 {
        // Params are the neural network weights
        self.mse_loss(params)
    }
    
    fn function_floor(&self) -> f64 {
        0.0 // Minimum possible MSE
    }
}

fn main() {
    // Generate synthetic training data (replace with your real data)
    let train_inputs: Vec<Vec<f64>> = (0..100)
        .map(|i| vec![i as f64 / 100.0; 10]) // 100 samples, 10 features each
        .collect();
    let train_labels: Vec<f64> = (0..100)
        .map(|i| (i as f64 / 100.0).sin()) // Target: sin(x)
        .collect();
    
    let optimizer = NeuralNetOptimizer::new(train_inputs, train_labels);
    let param_count = optimizer.param_count();
    
    println!("Optimizing neural network with {} parameters", param_count);
    
    // Define parameter bounds (typical weight initialization range)
    let bounds: Vec<RangeInclusive<f64>> = vec![-1.0..=1.0; param_count];
    
    // Configuration for large parameter spaces:
    // - Population size: 10-20x number of params for good coverage
    // - Regions: sqrt(population) is often a good heuristic
    let population_size = (param_count * 15).min(10000); // Scale up but cap at 10k
    let num_regions = (population_size as f64).sqrt() as usize;
    
    let constants = GlobalConstants::new(population_size, num_regions);
    let mut world = setup_world(&bounds, constants, Box::new(optimizer));
    
    println!("Population size: {}, Regions: {}", population_size, num_regions);
    println!("Initial score: {}", format_score(world.get_best_score()));
    
    // Run optimization
    let epochs = 200;
    for epoch in 1..=epochs {
        world.training_run(TrainingData::None { floor_value: 0.0 });
        
        if epoch % 20 == 0 {
            println!("Epoch {}: MSE = {}", epoch, format_score(world.get_best_score()));
        }
    }
    
    println!("\nFinal MSE: {}", format_score(world.get_best_score()));
    
    // Extract best weights
    let best = world.get_best_organism(TrainingData::None { floor_value: 0.0 });
    let best_weights = best.phenotype().expression_problem_values();
    
    println!("Optimized {} weights ready for use", best_weights.len());
}
```

**Key considerations for large-scale optimization:**

- **Parameter count**: This library has been tested with 50,000+ parameters. For neural networks,
  this is suitable for smaller architectures or as an initialization strategy before gradient descent.

- **Population scaling**: Use `10-20x` the number of parameters for population size, but cap at a
  reasonable limit (e.g., 10,000) for memory and performance.

- **Region count**: A good heuristic is `sqrt(population_size)` for balanced exploration/exploitation.

- **Hybrid approach**: Use hill_descent_lib to find a good initialization, then switch to gradient-based
  methods (SGD, Adam, etc.) for fine-tuning. Genetic algorithms excel at escaping local minima.

- **Data management**: Keep training data inside your fitness function struct to avoid passing large
  datasets through the API.

### Scaling Guidelines

Understanding how hill_descent_lib scales helps you configure it effectively for your problem size:

#### Parameter Count vs Performance

| Parameters   | Recommended Pop Size | Recommended Regions | Memory (approx) | Time per Epoch |
| ------------ | -------------------- | ------------------- | --------------- | -------------- |
| 2-10         | 100-200              | 10-15               | < 1 MB          | < 1ms          |
| 10-100       | 500-1,000            | 20-30               | < 10 MB         | 10-50ms        |
| 100-1,000    | 1,000-5,000          | 30-70               | 10-100 MB       | 50-500ms       |
| 1,000-10,000 | 5,000-10,000         | 70-100              | 100 MB - 1 GB   | 0.5-5s         |
| 10,000+      | 10,000 (capped)      | 100                 | 1-10 GB         | 5-30s          |

*Times measured on modern multi-core CPU (Ryzen/Intel i7+). Actual performance varies with fitness function complexity.*

#### Configuration Guidelines

**Population Size:**
- **Small problems (< 100 params)**: `10-20x` parameter count works well
- **Medium problems (100-1,000 params)**: Use `5-10x` parameter count
- **Large problems (1,000+ params)**: Cap at 10,000 for practical memory/time constraints
- **Rule of thumb**: More population = better exploration, but diminishing returns past 10,000

**Region Count:**
- **Formula**: `sqrt(population_size)` provides good balance
- **Minimum**: At least 10 regions for any problem
- **Maximum**: Diminishing returns past 100 regions
- **Trade-off**: More regions = finer spatial resolution but more overhead

**Epochs (Generations):**
- **Simple functions**: 100-500 epochs usually sufficient
- **Complex landscapes**: 1,000-5,000 epochs may be needed
- **Early stopping**: Monitor `get_best_score()` - stop if no improvement for 100+ epochs
- **Convergence**: Expect 70-90% of final quality within first 20% of epochs

#### Memory Usage

Memory consumption is primarily driven by:

```
Memory ≈ population_size × parameter_count × 24 bytes
       + region_count × 1KB overhead
       + function-specific data
```

**Example calculations:**
- 1,000 params, 5,000 pop: ~120 MB
- 10,000 params, 10,000 pop: ~2.4 GB
- 50,000 params, 10,000 pop: ~12 GB

**Tips for large problems:**
- Use `cargo build --release` - debug builds use significantly more memory
- Monitor with `get_state()` sparingly (serialization copies data)
- Keep fitness function data on disk/database if > 1 GB

#### When to Use vs When Not to Use

**✅ Good use cases:**
- **No gradient information available** (black-box optimization)
- **Multimodal landscapes** with many local minima
- **Discrete or mixed parameter spaces** (with custom encoding)
- **Initial parameter search** before fine-tuning with gradient methods
- **Robust optimization** where avoiding poor local minima is critical
- **Small to medium problems** (< 10,000 parameters)

**❌ Consider alternatives when:**
- **Smooth, unimodal functions**: Use gradient descent (much faster)
- **Very large parameter spaces** (> 50,000 params): Consider specialized methods
- **Real-time constraints**: Genetic algorithms need many evaluations
- **Analytical solutions exist**: Don't optimize if you can solve directly
- **Limited compute budget**: Each epoch evaluates entire population

#### Parallel Performance

hill_descent_lib uses Rayon for parallel region processing:

- **Scaling**: Near-linear speedup up to `min(num_regions, num_cores)`
- **Bottlenecks**: Fitness function evaluation time dominates
- **Sweet spot**: 4-16 cores see excellent utilization
- **Diminishing returns**: Beyond 32 cores, overhead increases

**Optimization tips:**
- Ensure fitness function is computationally intensive (> 1μs)
- Avoid I/O or locks in fitness function (breaks parallelism)
- Use `release` builds - parallelism overhead matters more in debug

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
