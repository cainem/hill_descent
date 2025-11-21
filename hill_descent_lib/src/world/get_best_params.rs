use super::World;

impl World {
    /// Returns the best organism's problem parameters without running any training.
    ///
    /// This is a non-mutating convenience method that provides quick access to the
    /// parameter values of the current best-performing organism. Unlike
    /// [`get_best_organism`](World::get_best_organism), this method does not trigger
    /// a training epoch and returns only the problem-specific parameters (excluding
    /// system parameters).
    ///
    /// # Returns
    ///
    /// Returns a vector containing the problem-specific parameter values of the organism
    /// with the lowest score. Returns an empty vector if the population has no scored
    /// organisms (which should not occur during normal usage).
    ///
    /// # Example: Simple Optimization
    ///
    /// ```
    /// use hill_descent_lib::{GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction, TrainingData};
    /// use std::ops::RangeInclusive;
    ///
    /// // Define a simple quadratic function to minimize
    /// #[derive(Debug)]
    /// struct Quadratic;
    /// impl SingleValuedFunction for Quadratic {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         params.iter().map(|x| x * x).sum()  // Minimize x²
    ///     }
    /// }
    ///
    /// let param_range = vec![-10.0..=10.0];
    /// let constants = GlobalConstants::new(50, 10);
    /// let mut world = setup_world(&param_range, constants, Box::new(Quadratic));
    ///
    /// // Run optimization
    /// for _ in 0..10 {
    ///     world.training_run(TrainingData::None { floor_value: 0.0 });
    /// }
    ///
    /// // Get best parameters without additional training
    /// let best_params = world.get_best_params();
    /// println!("Best parameters: {:?}", best_params);
    /// ```
    ///
    /// # Example: Multi-Dimensional Problem
    ///
    /// ```
    /// use hill_descent_lib::{GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction, TrainingData};
    ///
    /// #[derive(Debug)]
    /// struct Rosenbrock;
    /// impl SingleValuedFunction for Rosenbrock {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         let x = params[0];
    ///         let y = params[1];
    ///         100.0 * (y - x * x).powi(2) + (1.0 - x).powi(2)
    ///     }
    /// }
    ///
    /// let param_range = vec![-2.0..=2.0, -2.0..=2.0];
    /// let constants = GlobalConstants::new(100, 20);
    /// let mut world = setup_world(&param_range, constants, Box::new(Rosenbrock));
    ///
    /// // Run optimization
    /// for _ in 0..10 {
    ///     world.training_run(TrainingData::None { floor_value: 0.0 });
    /// }
    ///
    /// // Extract solution
    /// let params = world.get_best_params();
    /// let score = world.get_best_score();
    /// println!("Solution: x={:.6}, y={:.6}, score={:.6}", params[0], params[1], score);
    /// ```
    ///
    /// # Performance
    ///
    /// This is a very fast operation (O(1) lookup + O(n) copy where n is the number of
    /// parameters) as it only accesses the best organism from the already-evaluated
    /// population. It does not trigger any fitness evaluation or training.
    ///
    /// # See Also
    ///
    /// - [`get_best_organism`](World::get_best_organism) - Run one epoch and get full organism
    /// - [`get_best_score`](World::get_best_score) - Get just the fitness value
    /// - [`get_state`](World::get_state) - Full population snapshot
    pub fn get_best_params(&self) -> Vec<f64> {
        self.organisms
            .best()
            .map(|org| org.phenotype().expression_problem_values().to_vec())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TrainingData,
        parameters::global_constants::GlobalConstants,
        world::{World, WorldFunction},
    };
    use std::ops::RangeInclusive;

    // Simple deterministic world function for testing
    #[derive(Debug)]
    struct MockFn;
    impl WorldFunction for MockFn {
        fn run(&self, p: &[f64], _v: &[f64]) -> Vec<f64> {
            // Return sum of absolute values + 1.5 to ensure above floor of 1.0
            vec![p.iter().map(|x| x.abs()).sum::<f64>() + 1.5]
        }
    }

    #[test]
    fn given_optimized_world_when_get_best_params_then_returns_params() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, 0.0..=1.0];
        let gc = GlobalConstants::new(20, 5);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));

        // Run a few training epochs
        for _ in 0..3 {
            world.training_run(TrainingData::None { floor_value: 1.0 });
        }

        let params = world.get_best_params();

        // Should have 2 parameters (matching bounds)
        assert_eq!(params.len(), 2);

        // All parameters should be within bounds
        assert!(params[0] >= 0.0 && params[0] <= 1.0);
        assert!(params[1] >= 0.0 && params[1] <= 1.0);
    }

    #[test]
    fn given_single_dimension_when_get_best_params_then_returns_single_value() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let gc = GlobalConstants::new(15, 10);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));

        world.training_run(TrainingData::None { floor_value: 1.0 });

        let params = world.get_best_params();
        assert_eq!(params.len(), 1);
        assert!(params[0] >= -5.0 && params[0] <= 5.0);
    }

    #[test]
    fn given_world_with_supervised_data_when_get_best_params_then_returns_params() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(15, 10);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));

        // Single-example supervised dataset
        let inputs = vec![vec![1.0]];
        let outputs = vec![vec![1.0]];

        world.training_run(TrainingData::Supervised {
            inputs: &inputs,
            outputs: &outputs,
        });

        let params = world.get_best_params();
        assert_eq!(params.len(), 1);
        assert!(params[0] >= 0.0 && params[0] <= 1.0);
    }
}
