//! Training data configuration for optimization runs.
//!
//! This module provides the [`TrainingData`] enum which clarifies the different ways
//! to pass data to training methods in the genetic algorithm.

/// Specifies the training data for a genetic algorithm optimization run.
///
/// This enum makes the API clearer by explicitly distinguishing between:
/// - Standard optimization where the fitness function is self-contained ([`TrainingData::None`])
/// - Advanced supervised learning scenarios with external data ([`TrainingData::Supervised`])
///
/// # Use Cases
///
/// ## Standard Optimization (SingleValuedFunction)
///
/// Most optimization problems use [`SingleValuedFunction`](crate::SingleValuedFunction) where
/// the fitness function internally computes the score for given parameters. In this case,
/// no external training data is needed - only a floor value (theoretical minimum).
///
/// Use [`TrainingData::None`] for:
/// - Mathematical optimization (minimizing Rosenbrock, Sphere, etc.)
/// - Parameter tuning where fitness is computed internally
/// - Black-box optimization
/// - Any self-contained fitness function
///
/// ## Supervised Learning (WorldFunction)
///
/// Advanced use cases may require external input/output pairs for evaluation. This is
/// rare but supported for scenarios like:
/// - Training models with batch data
/// - Fitness functions that need reference data
/// - Supervised learning optimization
///
/// Use [`TrainingData::Supervised`] for these cases.
///
/// # Examples
///
/// ## Standard Optimization
///
/// ```rust
/// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction, TrainingData};
///
/// #[derive(Debug)]
/// struct Sphere;
///
/// impl SingleValuedFunction for Sphere {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         params.iter().map(|x| x * x).sum()
///     }
/// }
///
/// let bounds = vec![-10.0..=10.0; 2];
/// let constants = GlobalConstants::new(100, 10);
/// let mut world = setup_world(&bounds, constants, Box::new(Sphere));
///
/// // Optimize with no external data - just specify the theoretical minimum (floor)
/// for _ in 0..100 {
///     world.training_run(TrainingData::None { floor_value: 0.0 });
/// }
///
/// let best_params = world.get_best_params();
/// println!("Optimized parameters: {:?}", best_params);
/// ```
///
/// ## Large-Scale Optimization (Neural Network Weights)
///
/// ```rust
/// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction, TrainingData};
/// use std::sync::Arc;
///
/// # // Mock training data structure
/// # struct TrainingSet { data: Vec<Vec<f64>>, labels: Vec<usize> }
/// #
/// /// Fitness function that evaluates neural network performance
/// #[derive(Debug)]
/// struct NetworkFitness {
///     training_data: Arc<TrainingSet>,
/// }
///
/// impl SingleValuedFunction for NetworkFitness {
///     fn single_run(&self, weights: &[f64]) -> f64 {
///         // Internally sample from training_data
///         // Evaluate network with these weights
///         // Return validation loss
///         # 0.5 // Placeholder
///         // ... your implementation ...
///     }
/// }
///
/// # let training_data = Arc::new(TrainingSet { data: vec![], labels: vec![] });
/// // Optimize 50,000 neural network weights
/// let param_count = 50_000;
/// let bounds = vec![-1.0..=1.0; param_count];
/// let constants = GlobalConstants::new(500, 50);
///
/// let fitness = NetworkFitness { training_data };
/// let mut world = setup_world(&bounds, constants, Box::new(fitness));
///
/// // Training loop - no external data passed to training_run
/// for generation in 0..1000 {
///     world.training_run(TrainingData::None { floor_value: 0.0 });
///     
///     if generation % 100 == 0 {
///         println!("Generation {}: Loss = {:.6}", generation, world.get_best_score());
///     }
/// }
///
/// // Extract optimized weights
/// let best_weights = world.get_best_params();
/// ```
///
/// ## Supervised Learning (Advanced)
///
/// ```rust
/// use hill_descent_lib::{setup_world, GlobalConstants, WorldFunction, TrainingData};
///
/// # #[derive(Debug)]
/// # struct CustomFunction;
/// # impl WorldFunction for CustomFunction {
/// #     fn run(&self, params: &[f64], inputs: &[f64]) -> Vec<f64> {
/// #         vec![0.0]
/// #     }
/// # }
/// #
/// let bounds = vec![-5.0..=5.0; 3];
/// let constants = GlobalConstants::new(200, 20);
/// let mut world = setup_world(&bounds, constants, Box::new(CustomFunction));
///
/// // External training data
/// let inputs = vec![
///     vec![1.0, 2.0, 3.0],
///     vec![4.0, 5.0, 6.0],
/// ];
/// let targets = vec![
///     vec![10.0],
///     vec![20.0],
/// ];
///
/// // Train with external data
/// for _ in 0..100 {
///     world.training_run(TrainingData::Supervised {
///         inputs: &inputs,
///         outputs: &targets,
///     });
/// }
/// ```
///
/// # Design Rationale
///
/// Previous API used `&[[f64]]` parameters which caused confusion:
/// - Type signatures unclear (`&[[f64]]` vs `&[&[f64]]`)
/// - Empty slice patterns not obvious (`&[]` vs `&[0.0]`)
/// - Common case (no external data) looked like an error
///
/// The enum approach:
/// - Makes intent explicit at call sites
/// - Provides clear documentation for each case
/// - Eliminates "magic" empty slice patterns
/// - Type-safe distinction between use cases
#[derive(Debug, Clone, Copy)]
pub enum TrainingData<'a> {
    /// Standard optimization with no external training data.
    ///
    /// Use this variant when your fitness function is self-contained and computes
    /// scores internally (typical for [`SingleValuedFunction`](crate::SingleValuedFunction)).
    ///
    /// # Fields
    ///
    /// * `floor_value` - The theoretical minimum (floor) value of your fitness function.
    ///   This helps the algorithm calibrate fitness scores. For most functions, use `0.0`.
    ///   If your function has a known minimum (e.g., `-5.0`), specify that value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hill_descent_lib::TrainingData;
    ///
    /// // For functions with minimum at 0 (Sphere, Rosenbrock, etc.)
    /// let data = TrainingData::None { floor_value: 0.0 };
    ///
    /// // For functions with known different minimum
    /// let data = TrainingData::None { floor_value: -5.0 };
    /// ```
    None {
        /// The theoretical minimum value your fitness function can return.
        /// Used for fitness score calibration. Typically `0.0`.
        floor_value: f64,
    },

    /// Advanced supervised learning with external input/output pairs.
    ///
    /// Use this variant when your fitness function requires external reference data
    /// for evaluation. This is uncommon - most users should use [`TrainingData::None`].
    ///
    /// # Fields
    ///
    /// * `inputs` - 2D array of input examples: `&[[input1_val1, input1_val2], [input2_val1, ...]]`
    /// * `outputs` - 2D array of target outputs: `&[[target1], [target2], ...]`
    ///
    /// # Constraints
    ///
    /// - `inputs.len()` must equal `outputs.len()`
    /// - Each inner slice must be non-empty
    /// - All values must be finite (not NaN or infinite)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hill_descent_lib::TrainingData;
    ///
    /// let inputs = vec![
    ///     vec![1.0, 2.0],
    ///     vec![3.0, 4.0],
    /// ];
    /// let outputs = vec![
    ///     vec![5.0],
    ///     vec![10.0],
    /// ];
    ///
    /// let data = TrainingData::Supervised {
    ///     inputs: &inputs,
    ///     outputs: &outputs,
    /// };
    /// ```
    Supervised {
        /// Input data examples as a 2D array.
        inputs: &'a [Vec<f64>],
        /// Target output values as a 2D array.
        outputs: &'a [Vec<f64>],
    },
}

impl<'a> TrainingData<'a> {
    /// Returns the floor value for fitness calibration.
    ///
    /// For [`TrainingData::None`], returns the specified floor value.
    /// For [`TrainingData::Supervised`], returns `0.0` as a default.
    pub fn floor_value(&self) -> f64 {
        match self {
            TrainingData::None { floor_value } => *floor_value,
            TrainingData::Supervised { .. } => 0.0,
        }
    }

    /// Returns true if this is the supervised learning variant.
    pub fn is_supervised(&self) -> bool {
        matches!(self, TrainingData::Supervised { .. })
    }

    /// Returns true if this is the standard (no external data) variant.
    pub fn is_none(&self) -> bool {
        matches!(self, TrainingData::None { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_none_variant_when_floor_value_then_returns_specified_value() {
        let data = TrainingData::None { floor_value: 5.0 };
        assert_eq!(data.floor_value(), 5.0);
    }

    #[test]
    fn given_none_variant_when_is_none_then_returns_true() {
        let data = TrainingData::None { floor_value: 0.0 };
        assert!(data.is_none());
        assert!(!data.is_supervised());
    }

    #[test]
    fn given_supervised_variant_when_floor_value_then_returns_zero() {
        let inputs = vec![vec![1.0, 2.0]];
        let outputs = vec![vec![3.0]];
        let data = TrainingData::Supervised {
            inputs: &inputs,
            outputs: &outputs,
        };
        assert_eq!(data.floor_value(), 0.0);
    }

    #[test]
    fn given_supervised_variant_when_is_supervised_then_returns_true() {
        let inputs = vec![vec![1.0]];
        let outputs = vec![vec![2.0]];
        let data = TrainingData::Supervised {
            inputs: &inputs,
            outputs: &outputs,
        };
        assert!(data.is_supervised());
        assert!(!data.is_none());
    }

    #[test]
    fn given_none_with_negative_floor_when_created_then_stores_correctly() {
        let data = TrainingData::None { floor_value: -10.5 };
        assert_eq!(data.floor_value(), -10.5);
    }

    #[test]
    fn given_supervised_with_multiple_examples_when_created_then_valid() {
        let inputs = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let outputs = vec![vec![7.0], vec![8.0], vec![9.0]];

        let data = TrainingData::Supervised {
            inputs: &inputs,
            outputs: &outputs,
        };

        assert!(data.is_supervised());
        assert_eq!(data.floor_value(), 0.0);
    }
}
