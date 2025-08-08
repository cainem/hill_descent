use crate::E0;
use crate::world::{organisms::Organism, world_function::WorldFunction};

impl Organism {
    /// Runs the organism's phenotype with the provided function and inputs.
    ///
    /// Behaviour depends on `known_outputs`:
    /// 1. **Supervised mode** – when `known_outputs` is **non-empty** the score is the
    ///    sum-of-squared-errors between the world-function outputs and the `known_outputs`, plus `E0`.
    /// 2. **Objective-function mode** – when `known_outputs` is **empty** the world function is assumed
    ///    to return a single scalar that is to be *minimised*. The first element of the output vector
    ///    is taken directly as the fitness (again with `E0` added). This lets callers minimise an
    ///    arbitrary n-dimensional function without knowing its true minimum.
    ///
    /// # Panics
    ///
    /// This function will panic if the number of outputs from the world function does not match
    /// the number of known outputs provided.
    pub fn run(&self, function: &dyn WorldFunction, inputs: &[f64], known_outputs: &[f64]) {
        // Run the world function with the input for each phenotype
        let phenotype = self.phenotype();
        let phenotype_expressed_values = phenotype.expression_problem_values();
        let outputs = function.run(phenotype_expressed_values, inputs);

        debug_assert!(
            outputs.iter().all(|&x| x.is_finite()),
            "output must only contain finite numbers"
        );

        // Determine the fitness score depending on the scoring mode.
        let score = if known_outputs.is_empty() {
            // Objective-function mode.
            if outputs.is_empty() {
                panic!(
                    "World function returned no outputs, cannot evaluate objective-function mode."
                );
            }
            outputs[0] + E0
        } else {
            // Supervised mode – minimise squared error to known outputs.
            if outputs.len() != known_outputs.len() {
                panic!(
                    "The number of outputs ({}) must match the number of known outputs ({}).",
                    outputs.len(),
                    known_outputs.len()
                );
            }

            outputs
                .iter()
                .zip(known_outputs.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                + E0
        };

        self.set_score(Some(score));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        E0, parameters::parameter_enhancement::enhance_parameters, phenotype::Phenotype,
        world::world_function::WorldFunction,
    };
    use std::{ops::RangeInclusive, rc::Rc};

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

    // Helper to create a default organism for tests.
    fn create_test_organism() -> Organism {
        let user_defined_parameters: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let all_params = enhance_parameters(&user_defined_parameters);
        let expressed_values: Vec<f64> = all_params.iter().map(|p| *p.start()).collect();
        let phenotype = Rc::new(Phenotype::new_for_test(expressed_values));
        Organism::new(phenotype, 0)
    }

    #[test]
    fn given_valid_inputs_when_run_is_called_then_score_is_updated_correctly() {
        // Arrange
        let organism = create_test_organism();
        let inputs = vec![1.0, 2.0];
        let known_outputs = vec![0.5, 0.5];
        let test_fn = TestFn {
            output_values: vec![1.0, 0.0], // These will produce a known error
        };
        // Sum of squared errors = (1.0 - 0.5)^2 + (0.0 - 0.5)^2 = 0.25 + 0.25 = 0.5
        let expected_score = 0.5 + E0;

        // Act
        organism.run(&test_fn, &inputs, &known_outputs);

        // Assert
        assert_eq!(organism.score(), Some(expected_score));
    }

    #[test]
    fn given_perfect_match_when_run_is_called_then_score_is_max() {
        // Arrange
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let known_outputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![1.0], // Perfect match
        };
        // Sum of squared errors = 0.0
        let expected_score = E0;

        // Act
        organism.run(&test_fn, &inputs, &known_outputs);

        // Assert
        assert_eq!(organism.score(), Some(expected_score));
    }

    #[test]
    #[should_panic(
        expected = "The number of outputs (1) must match the number of known outputs (2)."
    )]
    fn given_mismatched_output_lengths_when_run_is_called_then_it_panics() {
        // Arrange
        let organism = create_test_organism();
        let inputs = vec![1.0, 2.0];
        let known_outputs = vec![0.5, 0.5]; // Expects 2 outputs
        let test_fn = TestFn {
            output_values: vec![1.0], // But function provides only 1
        };

        // Act & Assert
        organism.run(&test_fn, &inputs, &known_outputs);
    }

    #[test]
    fn given_empty_known_outputs_when_run_then_score_is_first_output_plus_e0() {
        let organism = create_test_organism();
        let inputs = vec![1.0, 2.0];
        let test_fn = TestFn {
            output_values: vec![2.5],
        };
        let expected = 2.5 + E0;
        organism.run(&test_fn, &inputs, &[]);
        assert_eq!(organism.score(), Some(expected));
    }

    #[test]
    #[should_panic(
        expected = "World function returned no outputs, cannot evaluate objective-function mode."
    )]
    fn given_empty_known_outputs_and_no_outputs_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![0.0];
        let test_fn = TestFn {
            output_values: vec![],
        };
        organism.run(&test_fn, &inputs, &[]);
    }
}
