use crate::world::validate_training_sets::validate_training_sets;

impl super::World {
    /// Runs a single training epoch over the provided training data.
    ///
    /// This function processes each input/output pair in the training data once,
    /// updating the world state after each example. It validates the training
    /// data before processing.
    ///
    /// # Panics
    ///
    /// This function will panic if the training data is invalid (see
    /// `validate_training_sets` for validation rules).
    ///
    /// # Parameters
    ///
    /// * `training_data`: Slice of input vectors for training
    /// * `known_outputs`: Slice of expected output vectors corresponding to the training data
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, training_data, known_outputs))
    )]
    pub fn run_epoch(&mut self, training_data: &[&[f64]], known_outputs: &[&[f64]]) {
        validate_training_sets(training_data, known_outputs);

        for (input, output) in training_data.iter().zip(known_outputs) {
            self.training_run(input, Some(output));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parameters::global_constants::GlobalConstants,
        world::{World, world_function::WorldFunction},
    };
    use std::ops::RangeInclusive;

    // Mock WorldFunction that returns a constant output for testing
    #[derive(Debug)]
    struct MockWorldFunction;
    impl WorldFunction for MockWorldFunction {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![0.5, 0.5] // Consistent output for testing
        }
    }

    fn create_test_world() -> World {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(10, 5); // population_size=10 > target_regions=5
        World::new(&bounds, gc, Box::new(MockWorldFunction))
    }

    #[test]
    fn given_valid_training_data_when_run_epoch_then_no_panic() {
        let mut world = create_test_world();
        let inputs = [vec![1.0, 2.0], vec![3.0, 4.0]];
        let outputs = [vec![0.5, 0.5], vec![0.5, 0.5]];

        // Convert Vec<Vec<f64>> to Vec<&[f64]>
        let input_refs: Vec<&[f64]> = inputs.iter().map(|v| v.as_slice()).collect();
        let output_refs: Vec<&[f64]> = outputs.iter().map(|v| v.as_slice()).collect();

        world.run_epoch(&input_refs, &output_refs);
        // If we get here, no panic occurred
    }

    #[test]
    #[should_panic(expected = "Training data cannot be empty")]
    fn given_empty_training_data_when_run_epoch_then_panic() {
        let mut world = create_test_world();
        let inputs: Vec<&[f64]> = Vec::new();
        let outputs: Vec<&[f64]> = Vec::new();

        world.run_epoch(&inputs, &outputs);
    }

    #[test]
    fn given_training_data_when_run_epoch_then_scores_are_updated() {
        let mut world = create_test_world();
        let inputs = [vec![1.0, 2.0], vec![3.0, 4.0]];
        let outputs = [vec![0.5, 0.5], vec![0.5, 0.5]];

        // Convert Vec<Vec<f64>> to Vec<&[f64]>
        let input_refs: Vec<&[f64]> = inputs.iter().map(|v| v.as_slice()).collect();
        let output_refs: Vec<&[f64]> = outputs.iter().map(|v| v.as_slice()).collect();

        // All scores should be None initially
        assert!(world.organisms.iter().all(|o| o.score().is_none()));

        world.run_epoch(&input_refs, &output_refs);

        // After running epoch, at least some organisms should have scores
        // (reproduction may create new organisms without scores, so we don't require all to have scores)
        assert!(world.organisms.iter().any(|o| o.score().is_some()));
    }
}
