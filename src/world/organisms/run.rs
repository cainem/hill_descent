use crate::world::{organisms::Organisms, world_function::WorldFunction};

impl Organisms {
    /// Runs all organisms in the population against the provided world function.
    ///
    /// This method iterates through each organism and calls its respective `run` method,
    /// updating each one's fitness score based on the provided inputs and known outputs.
    pub fn run(&mut self, function: &dyn WorldFunction, inputs: &[f64], known_outputs: &[f64]) {
        // run the world function with the input for each phenotype
        for organism in self.iter_mut() {
            organism.run(function, inputs, known_outputs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{E0, parameters::GlobalConstants, world::world_function::WorldFunction};
    use rand::rngs::mock::StepRng;

    // A mock WorldFunction for testing purposes.
    #[derive(Debug)]
    struct TestFn {
        output_values: Vec<f64>,
    }
    impl WorldFunction for TestFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            self.output_values.clone()
        }
    }

    #[test]
    fn given_organisms_when_run_is_called_then_all_organisms_are_run() {
        // Arrange
        let initial_value_bounds = vec![0.0..=1.0];
        let global_constants = GlobalConstants::new(5, 10); // 5 organisms
        let mut rng = StepRng::new(0, 1);
        let mut organisms = Organisms::new(&initial_value_bounds, &global_constants, &mut rng);

        let inputs = vec![1.0, 2.0];
        let known_outputs = vec![0.5, 0.5];
        let test_fn = TestFn {
            output_values: vec![1.0, 0.0], // These will produce a known error
        };
        // Sum of squared errors = (1.0 - 0.5)^2 + (0.0 - 0.5)^2 = 0.25 + 0.25 = 0.5
        let expected_score = 1.0 / (0.5 + E0);

        // Act
        organisms.run(&test_fn, &inputs, &known_outputs);

        // Assert
        assert_eq!(organisms.count(), 5);
        for organism in organisms.iter() {
            assert!(organism.score().is_some(), "Organism score should be Some");
            assert_eq!(
                organism.score().unwrap(),
                expected_score,
                "Organism score did not match expected score"
            );
        }
    }
}
