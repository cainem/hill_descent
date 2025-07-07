use crate::world::{World, organisms::Organism};

impl World {
    /// Applies one training epoch and returns the organism with the lowest fitness score (best fit).
    ///
    /// The function will panic if the training data/outputs slices are invalid. See
    /// `validate_training_sets` for validation rules.
    pub fn get_best_organism(
        &mut self,
        training_data: &[&[f64]],
        known_outputs: &[&[f64]],
    ) -> Organism {
        // 1. Validate inputs.
        crate::world::validate_training_sets::validate_training_sets(training_data, known_outputs);

        // 2. Run one epoch across the entire dataset.
        self.run_epoch(training_data, known_outputs);

        // 3. Return the fittest organism.
        self.organisms
            .best()
            .map(|rc| rc.as_ref().clone())
            .expect("Population contains no scored organisms")
    }
}

// unit tests below

#[cfg(test)]
#[allow(clippy::useless_vec)]
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
            vec![0.5]
        }
    }

    #[test]
    fn given_valid_data_when_get_best_then_returns_lowest_score() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(5, 10);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));

        // Single-example dataset
        let inputs = vec![vec![1.0]];
        let outputs = vec![vec![1.0]];
        let input_refs: Vec<&[f64]> = inputs.iter().map(|v| v.as_slice()).collect();
        let output_refs: Vec<&[f64]> = outputs.iter().map(|v| v.as_slice()).collect();

        let best = world.get_best_organism(&input_refs, &output_refs);
        assert!(best.score().is_some());
    }

    #[test]
    #[should_panic(expected = "Training data cannot be empty")]
    fn given_empty_data_when_get_best_then_panics() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(5, 10);
        let mut world = World::new(&bounds, gc, Box::new(MockFn));
        let inputs: Vec<&[f64]> = Vec::new();
        let outputs: Vec<&[f64]> = Vec::new();
        world.get_best_organism(&inputs, &outputs);
    }
}
