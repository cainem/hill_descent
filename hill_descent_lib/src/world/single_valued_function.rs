use crate::WorldFunction;

use std::fmt::Debug;

/// Trait for optimization functions that return a single scalar value to minimize.
///
/// This is the primary trait users implement to define their optimization problem.
/// The genetic algorithm will evolve parameters to minimize the value returned by
/// [`single_run`](SingleValuedFunction::single_run).
///
/// # Common Use Cases
///
/// - **Mathematical Functions**: Minimizing surfaces like Rosenbrock, Rastrigin, Ackley
/// - **Parameter Tuning**: Finding optimal configuration values for models
/// - **Engineering Design**: Minimizing cost, weight, or error metrics
/// - **Machine Learning**: Hyperparameter optimization
///
/// # Thread Safety
///
/// Functions must be `Sync` as they are called concurrently from multiple threads
/// during fitness evaluation.
///
/// # Examples
///
/// ## Simple Quadratic Function
///
/// ```
/// use hill_descent_lib::SingleValuedFunction;
///
/// #[derive(Debug)]
/// struct Quadratic;
///
/// impl SingleValuedFunction for Quadratic {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         // Minimize f(x, y) = x² + y²
///         // Global minimum: f(0, 0) = 0
///         params.iter().map(|x| x * x).sum()
///     }
/// }
///
/// // Use with setup_world:
/// use hill_descent_lib::{setup_world, GlobalConstants, TrainingData};
///
/// let param_range = vec![-10.0..=10.0, -10.0..=10.0];
/// let constants = GlobalConstants::new(100, 10);
/// let mut world = setup_world(&param_range, constants, Box::new(Quadratic));
///
/// for _ in 0..100 {
///     world.training_run(TrainingData::None { floor_value: 0.0 });
/// }
/// assert!(world.get_best_score() < 0.01);  // Should find near-zero minimum
/// ```
///
/// ## Function with Custom Floor
///
/// ```
/// use hill_descent_lib::SingleValuedFunction;
///
/// #[derive(Debug)]
/// struct ShiftedFunction;
///
/// impl SingleValuedFunction for ShiftedFunction {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         // Function with minimum value of -5.0
///         params.iter().map(|x| x * x).sum::<f64>() - 5.0
///     }
///
///     fn function_floor(&self) -> f64 {
///         -5.0  // Specify theoretical minimum
///     }
/// }
/// ```
///
/// ## Complex Optimization Problem
///
/// ```
/// use hill_descent_lib::SingleValuedFunction;
///
/// #[derive(Debug)]
/// struct ModelFitness {
///     target_data: Vec<f64>,
/// }
///
/// impl SingleValuedFunction for ModelFitness {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         // params[0]: learning_rate, params[1]: regularization, etc.
///         // Return mean squared error or similar metric
///         let predictions = self.run_model(params);
///         self.compute_error(&predictions)
///     }
/// }
///
/// impl ModelFitness {
///     fn run_model(&self, _params: &[f64]) -> Vec<f64> {
///         // Model execution logic
///         vec![0.0; self.target_data.len()]
///     }
///
///     fn compute_error(&self, predictions: &[f64]) -> f64 {
///         // Error calculation
///         predictions.iter()
///             .zip(&self.target_data)
///             .map(|(p, t)| (p - t).powi(2))
///             .sum::<f64>() / self.target_data.len() as f64
///     }
/// }
/// ```
///
/// # Implementation Notes
///
/// - The function should be **deterministic** - same inputs must produce same output
/// - Avoid expensive operations if possible - called millions of times during optimization
/// - Return `f64::INFINITY` for invalid parameter combinations
/// - Consider implementing [`function_floor`](SingleValuedFunction::function_floor) if your function has a known theoretical minimum
///
/// # See Also
///
/// - [`crate::WorldFunction`] - For multi-output functions (automatically implemented)
/// - [`crate::setup_world`] - Initialize optimization with your function
/// - [`super::World::training_run`] - Run optimization epochs
pub trait SingleValuedFunction: Debug + Sync {
    /// Evaluates the function for given parameter values.
    ///
    /// This is the core method that defines your optimization problem. The genetic
    /// algorithm will call this method millions of times with different parameter
    /// combinations, seeking the combination that produces the minimum return value.
    ///
    /// # Parameters
    ///
    /// * `phenotype_expressed_values` - The parameter values to evaluate. Length matches
    ///   the number of ranges provided to [`setup_world`](crate::setup_world). Values are
    ///   guaranteed to be within the bounds specified by those ranges.
    ///
    /// # Returns
    ///
    /// The fitness score to minimize. Lower values indicate better solutions.
    ///
    /// # Examples
    ///
    /// ```
    /// use hill_descent_lib::SingleValuedFunction;
    ///
    /// #[derive(Debug)]
    /// struct Distance;
    ///
    /// impl SingleValuedFunction for Distance {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         // Euclidean distance from origin
    ///         params.iter().map(|x| x * x).sum::<f64>().sqrt()
    ///     }
    /// }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method must be thread-safe as it's called concurrently. Avoid mutable
    /// state or use appropriate synchronization primitives.
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64;

    /// Returns the theoretical minimum value (floor) of this function.
    ///
    /// This is used to validate that computed values are not below the theoretical minimum,
    /// which would indicate a bug in the function implementation.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `0.0`, maintaining backward compatibility with
    /// existing functions that assume a minimum of zero.
    ///
    /// # Examples
    ///
    /// For a function with a known minimum of -5.0:
    /// ```ignore
    /// fn function_floor(&self) -> f64 {
    ///     -5.0
    /// }
    /// ```
    fn function_floor(&self) -> f64 {
        0.0
    }
}

impl<T> WorldFunction for T
where
    T: SingleValuedFunction + Debug,
{
    /// Adapts the `single_run` interface to the `WorldFunction` interface by wrapping the
    /// single scalar result in a `Vec`.
    fn run(&self, phenotype_expressed_values: &[f64], _inputs: &[f64]) -> Vec<f64> {
        vec![self.single_run(phenotype_expressed_values)]
    }

    /// Forwards the floor from SingleValuedFunction to WorldFunction.
    ///
    /// This ensures that when a SingleValuedFunction is used as a WorldFunction,
    /// its custom floor value is preserved and used for validation.
    fn function_floor(&self) -> f64 {
        SingleValuedFunction::function_floor(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock function with default floor (0.0)
    #[derive(Debug)]
    struct DefaultFloorFunction;

    impl SingleValuedFunction for DefaultFloorFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            0.5 // Returns value above default floor of 0.0
        }
    }

    /// Mock function with custom floor (1.0)
    #[derive(Debug)]
    struct CustomFloorFunction;

    impl SingleValuedFunction for CustomFloorFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            1.5 // Returns value above custom floor of 1.0
        }

        fn function_floor(&self) -> f64 {
            1.0 // Override default floor
        }
    }

    /// Mock function with negative floor (-5.0)
    #[derive(Debug)]
    struct NegativeFloorFunction;

    impl SingleValuedFunction for NegativeFloorFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            -2.0 // Returns value above floor of -5.0
        }

        fn function_floor(&self) -> f64 {
            -5.0 // Negative floor value
        }
    }

    #[test]
    fn given_default_floor_when_function_floor_called_then_returns_zero() {
        let func = DefaultFloorFunction;
        assert_eq!(SingleValuedFunction::function_floor(&func), 0.0);
    }

    #[test]
    fn given_custom_floor_when_function_floor_called_then_returns_custom_value() {
        let func = CustomFloorFunction;
        assert_eq!(SingleValuedFunction::function_floor(&func), 1.0);
    }

    #[test]
    fn given_negative_floor_when_function_floor_called_then_returns_negative_value() {
        let func = NegativeFloorFunction;
        assert_eq!(SingleValuedFunction::function_floor(&func), -5.0);
    }

    #[test]
    fn given_single_valued_function_when_used_as_world_function_then_floor_is_preserved() {
        // Test that the WorldFunction adapter preserves the floor
        let func = CustomFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        // The floor should be accessible through the WorldFunction trait
        assert_eq!(world_func.function_floor(), 1.0);
    }

    #[test]
    fn given_single_valued_function_when_run_then_returns_vec_with_single_value() {
        let func = DefaultFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        let result = world_func.run(&[1.0, 2.0], &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0.5);
    }

    #[test]
    fn given_custom_floor_function_when_run_then_output_above_floor() {
        let func = CustomFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        let result = world_func.run(&[1.0], &[]);
        assert_eq!(result[0], 1.5);
        assert!(
            result[0] >= world_func.function_floor(),
            "Output {} should be >= floor {}",
            result[0],
            world_func.function_floor()
        );
    }

    #[test]
    fn given_negative_floor_function_when_run_then_output_above_floor() {
        let func = NegativeFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        let result = world_func.run(&[], &[]);
        assert_eq!(result[0], -2.0);
        assert!(
            result[0] >= world_func.function_floor(),
            "Output {} should be >= floor {}",
            result[0],
            world_func.function_floor()
        );
    }
}
