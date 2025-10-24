use crate::world::{organisms::Organism, world_function::WorldFunction};

impl Organism {
    /// Runs the organism's phenotype with the provided function and inputs.
    ///
    /// The fitness score is calculated as the Euclidean distance between the world-function
    /// outputs and the `known_outputs` (target values or function floor):
    ///
    /// `score = sqrt(Σ(output_i - known_output_i)²)`
    ///
    /// **For `SingleValuedFunction` (objective-function mode):**
    /// - `known_outputs` should contain a single value: the function's theoretical minimum (floor)
    /// - The score becomes the distance from the floor: `sqrt((output - floor)²) = |output - floor|`
    /// - This allows minimization of functions with any minimum value, not just zero
    ///
    /// **For supervised learning:**
    /// - `known_outputs` contains the target output values
    /// - The score is the Euclidean distance from actual to target outputs
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The known_outputs slice is empty
    /// - Any known_outputs contain non-finite numbers (NaN or Infinity)
    /// - The number of outputs from the world function does not match the number of known outputs
    /// - Any output is below its corresponding known_output (floor violation)
    /// - The computed fitness score is non-finite (NaN or Infinity)
    pub fn run(&self, function: &dyn WorldFunction, inputs: &[f64], known_outputs: &[f64]) {
        // Validate known_outputs
        debug_assert!(!known_outputs.is_empty(), "known_outputs must not be empty");
        debug_assert!(
            known_outputs.iter().all(|&x| x.is_finite()),
            "known_outputs must only contain finite numbers"
        );

        // Run the world function with the input for each phenotype
        let phenotype = self.phenotype();
        let phenotype_expressed_values = phenotype.expression_problem_values();
        let outputs = function.run(phenotype_expressed_values, inputs);

        // Validate output count matches known_outputs
        if outputs.len() != known_outputs.len() {
            panic!(
                "The number of outputs ({}) must match the number of known outputs ({}).",
                outputs.len(),
                known_outputs.len()
            );
        }

        // Validate that outputs are not below their corresponding floors
        for (i, (&output, &floor)) in outputs.iter().zip(known_outputs.iter()).enumerate() {
            assert!(
                output >= floor,
                "Output[{i}] = {output} is below the function floor {floor}. This indicates a bug in the function implementation."
            );
        }

        // Calculate fitness as Euclidean distance: sqrt(Σ(output_i - known_output_i)²)
        let sum_of_squares: f64 = outputs
            .iter()
            .zip(known_outputs.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();

        let score = sum_of_squares.sqrt();

        // Validate that the score is finite
        assert!(
            score.is_finite(),
            "Fitness score must be finite, got: {score}. This indicates a bug in the fitness function implementation."
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
    use std::{ops::RangeInclusive, sync::Arc};

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
        let phenotype = Arc::new(Phenotype::new_for_test(expressed_values));
        Organism::new(phenotype, 0, (None, None))
    }

    #[test]
    fn given_valid_inputs_when_run_is_called_then_score_is_updated_correctly() {
        // Arrange
        let organism = create_test_organism();
        let inputs = vec![1.0, 2.0];
        let known_outputs = vec![0.5, 0.0]; // Floor values
        let test_fn = TestFn {
            output_values: vec![1.0, 0.5], // Both values are >= their respective floors
        };
        // Euclidean distance = sqrt((1.0 - 0.5)^2 + (0.5 - 0.0)^2) = sqrt(0.25 + 0.25) = sqrt(0.5)
        let expected_score = 0.5_f64.sqrt();

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
        // Euclidean distance = sqrt(0.0) = 0.0
        let expected_score = 0.0;

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
    fn given_single_value_function_when_run_then_score_is_distance_from_floor() {
        // This simulates a SingleValuedFunction with floor = 0.0
        let organism = create_test_organism();
        let inputs = vec![1.0, 2.0];
        let test_fn = TestFn {
            output_values: vec![2.5], // Function output
        };
        let floor = 0.0;
        // Expected score = sqrt((2.5 - 0.0)^2) = 2.5
        let expected = 2.5;
        organism.run(&test_fn, &inputs, &[floor]);
        assert_eq!(organism.score(), Some(expected));
    }

    #[test]
    #[should_panic(
        expected = "The number of outputs (0) must match the number of known outputs (1)."
    )]
    fn given_no_outputs_when_run_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![0.0];
        let test_fn = TestFn {
            output_values: vec![],
        };
        organism.run(&test_fn, &inputs, &[0.0]);
    }

    #[test]
    #[should_panic(expected = "known_outputs must not be empty")]
    fn given_empty_known_outputs_slice_when_run_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let empty_outputs: &[f64] = &[];
        let test_fn = TestFn {
            output_values: vec![1.0],
        };
        organism.run(&test_fn, &inputs, empty_outputs);
    }

    #[test]
    #[should_panic(expected = "known_outputs must only contain finite numbers")]
    fn given_infinite_known_outputs_when_run_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let infinite_outputs = vec![1.0, f64::INFINITY, 2.0];
        let test_fn = TestFn {
            output_values: vec![1.0, 2.0, 3.0],
        };
        organism.run(&test_fn, &inputs, &infinite_outputs);
    }

    #[test]
    #[should_panic(expected = "known_outputs must only contain finite numbers")]
    fn given_nan_known_outputs_when_run_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let nan_outputs = vec![1.0, f64::NAN];
        let test_fn = TestFn {
            output_values: vec![1.0, 2.0],
        };
        organism.run(&test_fn, &inputs, &nan_outputs);
    }

    #[test]
    #[should_panic(expected = "is below the function floor")]
    fn given_output_below_floor_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let floor = 0.0;
        let test_fn = TestFn {
            output_values: vec![-2.5], // Output below floor
        };
        organism.run(&test_fn, &inputs, &[floor]);
    }

    #[test]
    #[should_panic(expected = "Fitness score must be finite")]
    fn given_infinite_output_when_run_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![f64::INFINITY], // Infinite output produces infinite distance
        };
        organism.run(&test_fn, &inputs, &[0.0]);
    }

    #[test]
    #[should_panic(expected = "is below the function floor")]
    fn given_nan_output_when_run_then_panics() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let test_fn = TestFn {
            output_values: vec![f64::NAN], // NaN fails floor validation (NaN < x is always false)
        };
        organism.run(&test_fn, &inputs, &[0.0]);
    }

    #[test]
    fn given_output_at_floor_when_run_then_score_is_zero() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let floor = 0.0;
        let test_fn = TestFn {
            output_values: vec![floor], // Output exactly at floor
        };
        organism.run(&test_fn, &inputs, &[floor]);
        assert_eq!(organism.score(), Some(0.0));
    }

    #[test]
    fn given_output_above_floor_when_run_then_score_is_distance() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let floor = 0.0;
        let test_fn = TestFn {
            output_values: vec![42.0], // Output above floor
        };
        // Expected score = sqrt((42.0 - 0.0)^2) = 42.0
        organism.run(&test_fn, &inputs, &[floor]);
        assert_eq!(organism.score(), Some(42.0));
    }

    #[test]
    fn given_negative_floor_when_run_then_works_correctly() {
        let organism = create_test_organism();
        let inputs = vec![1.0];
        let floor = -10.0;
        let test_fn = TestFn {
            output_values: vec![-5.0], // Output above negative floor
        };
        // Expected score = sqrt((-5.0 - (-10.0))^2) = sqrt(25.0) = 5.0
        organism.run(&test_fn, &inputs, &[floor]);
        assert_eq!(organism.score(), Some(5.0));
    }
}
