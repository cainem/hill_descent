use crate::world::{World, organisms::Organism};
use std::sync::Arc;

impl World {
    /// Runs one training epoch and returns the organism with the best (lowest) fitness score.
    ///
    /// This method combines a single training epoch with retrieval of the best solution.
    /// Use this when you want to perform one final optimization step before extracting results.
    ///
    /// **Note**: Most users should call [`training_run`](World::training_run) in a loop for
    /// optimization, then check results with [`get_best_score`](World::get_best_score). This  
    /// method is primarily useful when you need both the organism details AND want one more
    /// training epoch.
    ///
    /// # Parameters
    ///
    /// For [`SingleValuedFunction`](crate::SingleValuedFunction) optimization:
    /// * `training_data` - Pass `&[&[]]` (slice containing empty slice)
    /// * `known_outputs` - Pass `&[&[]]` (slice containing empty slice)
    ///
    /// For advanced [`WorldFunction`](crate::WorldFunction) usage:
    /// * `training_data` - Multiple input examples: `&[&[input1], &[input2], ...]`
    /// * `known_outputs` - Corresponding target outputs: `&[&[target1], &[target2], ...]`
    ///
    /// # Returns
    ///
    /// An `Arc<Organism>` pointing to the organism with the lowest fitness score after
    /// running the training epoch. Access the parameter values with
    /// `organism.phenotype().expression_problem_values()`.
    ///
    /// # Examples
    ///
    /// ## Extract Best Solution After Optimization
    ///
    /// ```no_run
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
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
    /// let param_range = vec![-10.0..=10.0; 3];
    /// let constants = GlobalConstants::new(200, 20);
    /// let mut world = setup_world(&param_range, constants, Box::new(Sphere));
    ///
    /// // Run optimization
    /// for _ in 0..500 {
    ///     world.training_run(&[], &[0.0]);
    /// }
    ///
    /// // Extract best solution (with one final epoch)
    /// let best = world.get_best_organism(&[&[]], &[&[]]);
    /// let params = best.phenotype().expression_problem_values();
    ///
    /// println!("Best parameters: {:?}", params);
    /// println!("Fitness: {:?}", best.score());
    ///
    /// // Verify near-optimal (global minimum at origin for sphere)
    /// assert!(params.iter().all(|&x| x.abs() < 0.1));
    /// ```
    ///
    /// ## Get Final Solution with Score
    ///
    /// ```no_run
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
    ///
    /// #[derive(Debug)]
    /// struct Rosenbrock;
    ///
    /// impl SingleValuedFunction for Rosenbrock {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         let x = params[0];
    ///         let y = params[1];
    ///         (1.0 - x).powi(2) + 100.0 * (y - x.powi(2)).powi(2)
    ///     }
    /// }
    ///
    /// let param_range = vec![-5.0..=5.0; 2];
    /// let constants = GlobalConstants::new(1000, 100);
    /// let mut world = setup_world(&param_range, constants, Box::new(Rosenbrock));
    ///
    /// for _ in 0..1000 {
    ///     world.training_run(&[], &[0.0]);
    /// }
    ///
    /// let best = world.get_best_organism(&[&[]], &[&[]]);
    /// let params = best.phenotype().expression_problem_values();
    /// let score = best.score().unwrap();
    ///
    /// // Global minimum at (1, 1) with score 0
    /// println!("Found x={:.3}, y={:.3}, f(x,y)={:.6}", params[0], params[1], score);
    /// assert!((params[0] - 1.0).abs() < 0.1);
    /// assert!((params[1] - 1.0).abs() < 0.1);
    /// ```
    ///
    /// ## Multiple Dimensions
    ///
    /// ```no_run
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
    ///
    /// #[derive(Debug)]
    /// struct HighDimension;
    ///
    /// impl SingleValuedFunction for HighDimension {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         // Rastrigin function in n dimensions
    ///         let n = params.len() as f64;
    ///         10.0 * n + params.iter()
    ///             .map(|&x| x.powi(2) - 10.0 * (2.0 * std::f64::consts::PI * x).cos())
    ///             .sum::<f64>()
    ///     }
    /// }
    ///
    /// let dimensions = 10;
    /// let param_range = vec![-5.12..=5.12; dimensions];
    /// let constants = GlobalConstants::new(2000, 200);
    /// let mut world = setup_world(&param_range, constants, Box::new(HighDimension));
    ///
    /// for _ in 0..2000 {
    ///     world.training_run(&[], &[0.0]);
    /// }
    ///
    /// let best = world.get_best_organism(&[&[]], &[&[]]);
    /// let params = best.phenotype().expression_problem_values();
    ///
    /// println!("10D solution: {:?}", params);
    /// println!("Quality: {:.6}", best.score().unwrap());
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The training data arrays are malformed (mismatched lengths)
    /// - The population has no scored organisms (should not occur in normal usage)
    ///
    /// # Performance
    ///
    /// This method runs one complete training epoch before returning, so it has the
    /// same performance characteristics as [`training_run`](World::training_run).
    ///
    /// # See Also
    ///
    /// - [`training_run`](World::training_run) - Run epochs without extracting results
    /// - [`get_best_score`](World::get_best_score) - Get just the fitness value
    /// - [`get_state`](World::get_state) - Full population snapshot
    pub fn get_best_organism(
        &mut self,
        training_data: &[&[f64]],
        known_outputs: &[&[f64]],
    ) -> Arc<Organism> {
        // 1. Validate inputs.
        crate::world::validate_training_sets::validate_training_sets(training_data, known_outputs);

        // 2. Run one epoch across the entire dataset.
        self.run_epoch(training_data, known_outputs);

        // 3. Return the fittest organism as Rc.
        self.organisms
            .best()
            .expect("Population contains no scored organisms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parameters::global_constants::GlobalConstants, world::world_function::WorldFunction,
    };
    use std::ops::RangeInclusive;

    // Simple deterministic world function for scoring
    #[derive(Debug)]
    struct MockFn;
    impl WorldFunction for MockFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![1.5] // Returns value above floor of 1.0
        }
    }

    #[test]
    fn given_valid_data_when_get_best_then_returns_lowest_score() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(15, 10);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));

        // Single-example dataset
        let inputs = [vec![1.0]];
        let outputs = [vec![1.0]]; // Floor value
        let input_refs: Vec<&[f64]> = inputs.iter().map(|v| v.as_slice()).collect();
        let output_refs: Vec<&[f64]> = outputs.iter().map(|v| v.as_slice()).collect();

        let best = world.get_best_organism(&input_refs, &output_refs);
        assert!(best.score().is_some());
    }

    #[test]
    #[should_panic(expected = "Training data cannot be empty")]
    fn given_empty_data_when_get_best_then_panics() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(15, 10);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));
        let inputs: Vec<&[f64]> = Vec::new();
        let outputs: Vec<&[f64]> = Vec::new();
        world.get_best_organism(&inputs, &outputs);
    }
}
