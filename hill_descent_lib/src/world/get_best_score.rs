use super::World;

impl World {
    /// Returns the best (lowest) fitness score in the current population.
    ///
    /// # Returns
    ///
    /// Returns the lowest score among all organisms that have been scored,
    /// or `f64::MAX` if no organisms have scores.
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
            vec![0.5]
        }
    }

    #[test]
    fn given_world_with_scored_organisms_when_get_best_score_then_returns_lowest() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(10, 5);
        let mut world = World::new(&bounds, gc, Box::new(TestFn));

        // Run training to score organisms
        world.training_run(&[0.5], &[1.0]);

        let best_score = world.get_best_score();
        assert!(best_score < f64::MAX, "Should have a valid score");
        assert!(
            best_score > 0.0,
            "Score should be positive for this test function"
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
