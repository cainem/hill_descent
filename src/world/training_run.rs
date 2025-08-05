use super::World;

impl World {
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, inputs, known_outputs))
    )]
    /// Runs a single training iteration.
    ///
    /// # Returns
    ///
    /// Returns `true` if the resolution limit has been reached and no further
    /// meaningful splits are possible, `false` otherwise.
    pub fn training_run(&mut self, inputs: &[f64], known_outputs: &[f64]) -> bool {
        let _initial_population = self.organisms.len();
        crate::info!("=== TRAINING RUN START: Population = {} ===", _initial_population);

        // 1. Evaluate fitness for every organism
        self.organisms
            .run_all(self.world_function.as_ref(), inputs, known_outputs);
        crate::info!("After fitness evaluation: Population = {}", self.organisms.len());

        // 2. Generate offspring to fill regional deficits
        let mut offspring = crate::world::organisms::Organisms::new_empty();
        self.regions.repopulate(&mut self.rng, &mut offspring);
        let _offspring_count = offspring.len();
        self.organisms.extend(offspring.into_inner());
        crate::info!("After reproduction: Population = {} (added {} offspring)", self.organisms.len(), _offspring_count);

        // 3. Age organisms and cull those exceeding their max age
        self.organisms.increment_ages();
        let _pre_cull_population = self.organisms.len();
        self.remove_dead();
        let _post_cull_population = self.organisms.len();
        crate::info!("After aging/culling: Population = {} (removed {} dead)", _post_cull_population, _pre_cull_population - _post_cull_population);

        // 4. Re-evaluate spatial structure (bounding boxes, region keys, capacities).
        // This call updates region min scores and carrying capacities internally.
        // Returns true if resolution limit reached, false otherwise.
        let resolution_limit_reached = self
            .regions
            .update(&mut self.organisms, &mut self.dimensions);

        crate::info!("=== TRAINING RUN END: Population = {}, Resolution limit: {} ===", self.organisms.len(), resolution_limit_reached);
        resolution_limit_reached
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::E0;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::world::world_function::WorldFunction;
    use std::ops::RangeInclusive;

    // Mock WorldFunction that returns a single constant value (0.5) for deterministic tests.
    #[derive(Debug)]
    struct IdentityFn;
    impl WorldFunction for IdentityFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![0.5] // deterministic output
        }
    }

    // given_valid_inputs_when_training_run_then_scores_positive_and_ages_increment
    #[test]
    fn given_valid_inputs_when_training_run_then_scores_positive_and_ages_increment() {
        // Arrange
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(20, 16);
        let mut world = World::new(&bounds, gc, Box::new(IdentityFn));
        let inputs = vec![0.0];
        let known_outputs = vec![1.0];

        // Act
        let at_resolution_limit = world.training_run(&inputs, &known_outputs);

        // Assert
        assert!(
            !at_resolution_limit,
            "Should not be at resolution limit for small world"
        );
        assert!(
            world
                .organisms
                .iter()
                .all(|o| o.score().unwrap_or(0.0) > 0.0),
            "All organisms should have positive scores"
        );
        assert!(!world.organisms.is_empty());
    }

    // given_perfect_match_when_training_run_then_resolution_limit_not_reached
    #[test]
    fn given_perfect_match_when_training_run_then_resolution_limit_not_reached() {
        // Arrange
        // Mock WorldFunction that always returns the perfect matching value (1.0) for scoring tests.
        #[derive(Debug)]
        struct PerfectFn;
        impl WorldFunction for PerfectFn {
            fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
                vec![1.0]
            }
        }
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(16, 8);
        let mut world = World::new(&bounds, gc, Box::new(PerfectFn));
        let inputs = vec![0.0];
        let known_outputs = vec![1.0];

        // Act
        let at_resolution_limit = world.training_run(&inputs, &known_outputs);

        // Assert
        assert!(
            !at_resolution_limit,
            "Should not be at resolution limit for small world"
        );
        // Verify that organisms have been scored (perfect match should give E0)
        let best_score = world
            .organisms
            .iter()
            .filter_map(|o| o.score())
            .fold(f64::MAX, f64::min);
        assert!((best_score - E0).abs() < f64::EPSILON);
    }
}
