use super::World;

impl World {
    /// Returns the best (lowest) fitness score found in the current population.
    ///
    /// This is the primary method for monitoring optimization progress. Call this
    /// after [`training_run`](World::training_run) to check if the algorithm is
    /// converging toward an optimal solution.
    ///
    /// # Returns
    ///
    /// The lowest fitness score among all evaluated organisms in the current generation.
    /// Returns `f64::MAX` if no organisms have been scored yet (should not occur in
    /// normal usage after calling `training_run`).
    ///
    /// # Examples
    ///
    /// ## Basic Progress Monitoring
    ///
    /// ```
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
    /// let param_range = vec![-10.0..=10.0; 2];
    /// let constants = GlobalConstants::new(100, 10);
    /// let mut world = setup_world(&param_range, constants, Box::new(Sphere));
    ///
    /// world.training_run(&[], &[0.0]);
    /// let initial_score = world.get_best_score();
    ///
    /// for _ in 0..100 {
    ///     world.training_run(&[], &[0.0]);
    /// }
    ///
    /// let final_score = world.get_best_score();
    /// assert!(final_score < initial_score);  // Should improve
    /// assert!(final_score < 0.1);            // Should be near optimal
    /// ```
    ///
    /// ## Early Stopping with Convergence Threshold
    ///
    /// ```
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
    ///
    /// #[derive(Debug)]
    /// struct Quadratic;
    ///
    /// impl SingleValuedFunction for Quadratic {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         params[0].powi(2)
    ///     }
    /// }
    ///
    /// let param_range = vec![-10.0..=10.0];
    /// let constants = GlobalConstants::new(50, 5);
    /// let mut world = setup_world(&param_range, constants, Box::new(Quadratic));
    ///
    /// let target_fitness = 0.001;
    /// let max_epochs = 500;
    ///
    /// for epoch in 0..max_epochs {
    ///     world.training_run(&[], &[0.0]);
    ///     
    ///     if world.get_best_score() < target_fitness {
    ///         println!("Target fitness reached after {} epochs", epoch + 1);
    ///         break;
    ///     }
    /// }
    /// ```
    ///
    /// ## Comparing Multiple Runs
    ///
    /// ```
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
    ///
    /// #[derive(Debug)]
    /// struct TestFunction;
    ///
    /// impl SingleValuedFunction for TestFunction {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         params.iter().map(|x| x * x).sum()
    ///     }
    /// }
    ///
    /// let param_range = vec![-5.0..=5.0; 3];
    /// let epochs = 200;
    ///
    /// // Test different population sizes
    /// for pop_size in [50, 100, 200] {
    ///     let constants = GlobalConstants::new(pop_size, pop_size / 10);
    ///     let mut world = setup_world(&param_range, constants, Box::new(TestFunction));
    ///     
    ///     for _ in 0..epochs {
    ///         world.training_run(&[], &[0.0]);
    ///     }
    ///     
    ///     println!("Pop size {}: Final score = {}", pop_size, world.get_best_score());
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// This is an O(n) operation where n is the population size, but typically very
    /// fast (microseconds for populations of 1000s). Safe to call every epoch for
    /// monitoring without significant performance impact.
    ///
    /// # See Also
    ///
    /// - [`training_run`](World::training_run) - Run optimization epochs
    /// - [`get_best_organism`](World::get_best_organism) - Get the parameters for the best organism
    /// - [`get_state`](World::get_state) - Full population state for detailed analysis
    pub fn get_best_score(&self) -> f64 {
        self.organisms
            .iter()
            .filter_map(|o| o.score())
            .fold(f64::MAX, f64::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::world::world_function::WorldFunction;
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct TestFn;
    impl WorldFunction for TestFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![1.5] // Returns value above floor of 1.0
        }
    }

    #[test]
    fn given_world_with_scored_organisms_when_get_best_score_then_returns_lowest() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(10, 5);
        let mut world = World::new(&bounds, gc, Box::new(TestFn));

        // Run training to score organisms, floor = 1.0
        world.training_run(&[0.5], &[1.0]);

        let best_score = world.get_best_score();
        assert!(best_score < f64::MAX, "Should have a valid score");
        assert!(
            best_score >= 0.0,
            "Score should be non-negative (distance metric)"
        );
    }

    #[test]
    fn given_world_with_no_scored_organisms_when_get_best_score_then_returns_max() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(10, 5);
        let world = World::new(&bounds, gc, Box::new(TestFn));

        let best_score = world.get_best_score();
        assert_eq!(
            best_score,
            f64::MAX,
            "Should return MAX when no organisms are scored"
        );
    }
}
