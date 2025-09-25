use crate::world::{organisms::Organism, world_function::WorldFunction};

impl Organism {
    /// Runs the organism's phenotype with the provided function and inputs.
    ///
    /// Behaviour depends on `known_outputs`:
    /// 1. **Supervised mode** – when `known_outputs` is `Some(&[f64])` the score is the
    ///    sum-of-squared-errors between the world-function outputs and the `known_outputs`.
    /// 2. **Objective-function mode** – when `known_outputs` is `None` the world function is assumed
    ///    to return a single scalar that is to be *minimised*. The first element of the output vector
    ///    is taken directly as the fitness. This lets callers minimise an arbitrary n-dimensional
    ///    function without knowing its true minimum.
    ///
    /// **Important**: Fitness functions must return non-negative values. In objective-function mode,
    /// ensure your fitness function is properly normalized (e.g., shift so global minimum ≥ 0).
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The number of outputs from the world function does not match the number of known outputs provided
    /// - In supervised mode (`Some(&[f64])`), the known_outputs slice is empty
    /// - In supervised mode (`Some(&[f64])`), any known_outputs contain non-finite numbers (NaN or Infinity)
    /// - In objective-function mode (`None`), the world function returns no outputs
    /// - The computed fitness score is negative (fitness functions must be normalized to return ≥ 0)
    /// - The computed fitness score is non-finite (NaN or Infinity)
    pub fn run(&self, function: &dyn WorldFunction, inputs: &[f64], known_outputs: Option<&[f64]>) {
        // Run the world function with the input for each phenotype
        let phenotype = self.phenotype();
        let phenotype_expressed_values = phenotype.expression_problem_values();
        let outputs = function.run(phenotype_expressed_values, inputs);

        // Determine the fitness score depending on the scoring mode.
        let score = match known_outputs {
            None => {
                // Objective-function mode.
                if outputs.is_empty() {
                    panic!(
                        "World function returned no outputs, cannot evaluate objective-function mode."
                    );
                }
                outputs[0]
            }
            Some(expected_outputs) => {
                // Supervised mode – minimise squared error to known outputs.
                debug_assert!(
                    !expected_outputs.is_empty(),
                    "known_outputs must not be empty in supervised mode"
                );
                debug_assert!(
                    expected_outputs.iter().all(|&x| x.is_finite()),
                    "known_outputs must only contain finite numbers"
                );

                if outputs.len() != expected_outputs.len() {
                    panic!(
                        "The number of outputs ({}) must match the number of known outputs ({}).",
                        outputs.len(),
                        expected_outputs.len()
                    );
                }

                outputs
                    .iter()
                    .zip(expected_outputs.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
            }
        };

        // Validate that the score is finite and non-negative
        assert!(
            score.is_finite(),
            "Fitness score must be finite, got: {score}. This indicates a bug in the fitness function implementation."
        );
        assert!(
            score >= 0.0,
            "Fitness score must be non-negative, got: {score}. This indicates the fitness function is not properly normalized for minimization."
        );

        self.set_score(Some(score));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parameters::parameter_enhancement::enhance_parameters, phenotype::Phenotype,
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
        Organism::new(phenotype, 0, (None, None))
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
        let expected_score = 0.5;

        // Act
        organism.run(&test_fn, &inputs, Some(&known_outputs));

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
        let expected_score = 0.0;

        // Act
        organism.run(&test_fn, &inputs, Some(&known_outputs));

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
        organism.run(&test_fn, &inputs, Some(&known_outputs));
    }

    #[test]
    fn given_empty_known_outputs_when_run_then_score_is_first_output() {
        let organism = create_test_organism();
        let inputs = vec![1.0, 2.0];
        let test_fn = TestFn {
            output_values: vec![2.5],
        };
        let expected = 2.5;
        organism.run(&test_fn, &inputs, None);
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
        organism.run(&test_fn, &inputs, None);
    }

    #[test]
    #[should_panic(expected = "known_outputs must not be empty in supervised mode")]
    fn given_empty_known_outputs_slice_when_supervised_mode_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let empty_outputs: &[f64] = &[];
        let test_fn = TestFn {
            output_values: vec![1.0],
        };
        organism.run(&test_fn, &inputs, Some(empty_outputs));
    }

    #[test]
    #[should_panic(expected = "known_outputs must only contain finite numbers")]
    fn given_infinite_known_outputs_when_supervised_mode_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let infinite_outputs = vec![1.0, f64::INFINITY, 2.0];
        let test_fn = TestFn {
            output_values: vec![1.0, 2.0, 3.0],
        };
        organism.run(&test_fn, &inputs, Some(&infinite_outputs));
    }

    #[test]
    #[should_panic(expected = "known_outputs must only contain finite numbers")]
    fn given_nan_known_outputs_when_supervised_mode_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let nan_outputs = vec![1.0, f64::NAN];
        let test_fn = TestFn {
            output_values: vec![1.0, 2.0],
        };
        organism.run(&test_fn, &inputs, Some(&nan_outputs));
    }

    #[test]
    #[should_panic(expected = "Fitness score must be non-negative")]
    fn given_negative_fitness_when_objective_mode_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![-2.5], // Negative fitness score
        };
        organism.run(&test_fn, &inputs, None);
    }

    #[test]
    #[should_panic(expected = "Fitness score must be finite")]
    fn given_infinite_fitness_when_objective_mode_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![f64::INFINITY], // Infinite fitness score
        };
        organism.run(&test_fn, &inputs, None);
    }

    #[test]
    #[should_panic(expected = "Fitness score must be finite")]
    fn given_nan_fitness_when_objective_mode_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![f64::NAN], // NaN fitness score
        };
        organism.run(&test_fn, &inputs, None);
    }

    #[test]
    fn given_zero_fitness_when_objective_mode_then_succeeds() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![0.0], // Zero fitness should be allowed
        };
        organism.run(&test_fn, &inputs, None);
        assert_eq!(organism.score(), Some(0.0));
    }

    #[test]
    fn given_positive_fitness_when_objective_mode_then_succeeds() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![42.0], // Positive fitness should work
        };
        organism.run(&test_fn, &inputs, None);
        assert_eq!(organism.score(), Some(42.0));
    }
}
