use super::World;

impl World {
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, inputs, known_outputs))
    )]
    pub fn training_run(&mut self, inputs: &[f64], known_outputs: &[f64]) -> f64 {
        // 1. Evaluate fitness for every organism
        self.organisms
            .run_all(self.world_function.as_ref(), inputs, known_outputs);

        // 2. Generate offspring to fill regional deficits
        let mut offspring = crate::world::organisms::Organisms::new_empty();
        self.regions.repopulate(&mut self.rng, &mut offspring);
        self.organisms.extend(offspring.into_inner());

        // 3. Age organisms and cull those exceeding their max age
        self.organisms.increment_ages();
        self.remove_dead();

        // 4. Re-evaluate spatial structure (bounding boxes, region keys, capacities).
        // This call updates region min scores and carrying capacities internally.
        self.regions
            .update(&mut self.organisms, &mut self.dimensions);

        // Return the best (lowest) fitness score in the population for monitoring.
        self.organisms
            .iter()
            .filter_map(|o| o.score())
            .fold(f64::MAX, f64::min)
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
        let best_score = world.training_run(&inputs, &known_outputs);

        // Assert
        assert!(best_score > 0.0, "Best score should be positive");
        assert!(
            world
                .organisms
                .iter()
                .all(|o| o.score().unwrap_or(0.0) > 0.0),
            "All organisms should have positive scores"
        );
        assert!(world.organisms.len() > 0);
    }

    // given_perfect_match_when_training_run_then_best_score_equals_max
    #[test]
    fn given_perfect_match_when_training_run_then_best_score_equals_max() {
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
        let best_score = world.training_run(&inputs, &known_outputs);

        // Assert
        let expected = E0;
        assert!((best_score - expected).abs() < f64::EPSILON);
    }
}
