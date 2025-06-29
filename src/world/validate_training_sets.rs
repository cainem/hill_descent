use std::fmt::Debug;

/// Validates that the provided training inputs and known outputs form a
/// well-structured data set.
///
/// Invariant rules (derived from PDD §5.3.2):
/// 1. Both slices must be non-empty.
/// 2. They must have equal length (same number of examples).
/// 3. For every example `i`, the input vector and output vector must have the
///    same arity.
///
/// If any invariant is violated the function panics with a descriptive
/// message.  The function is intentionally simple so that callers do not have
/// to deal with a `Result` type and because these errors are programmer
/// mistakes rather than runtime conditions.
pub fn validate_training_sets<I, O>(training_data: &[I], known_outputs: &[O])
where
    I: AsRef<[f64]> + Debug,
    O: AsRef<[f64]> + Debug,
{
    if training_data.is_empty() {
        panic!("Training data cannot be empty");
    }
    if training_data.len() != known_outputs.len() {
        panic!(
            "Training data and known outputs must contain the same number of examples ({} != {})",
            training_data.len(),
            known_outputs.len()
        );
    }

    for (idx, (input, output)) in training_data.iter().zip(known_outputs).enumerate() {
        let in_len = input.as_ref().len();
        let out_len = output.as_ref().len();
        if in_len != out_len {
            panic!("Mismatch at index {idx}: input length {in_len} != output length {out_len}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_training_sets;

    #[test]
    fn given_valid_sets_when_validate_then_no_panic() {
        let inputs = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let outputs = vec![vec![0.5, 0.5], vec![0.1, 0.9]];
        validate_training_sets(&inputs, &outputs);
    }

    #[test]
    #[should_panic(expected = "Training data cannot be empty")]
    fn given_empty_inputs_when_validate_then_panic() {
        let inputs: Vec<Vec<f64>> = Vec::new();
        let outputs: Vec<Vec<f64>> = Vec::new();
        validate_training_sets(&inputs, &outputs);
    }

    #[test]
    #[should_panic(expected = "same number of examples")]
    fn given_mismatched_lengths_when_validate_then_panic() {
        let inputs = vec![vec![1.0]];
        let outputs = vec![vec![1.0], vec![2.0]]; // Extra example
        validate_training_sets(&inputs, &outputs);
    }

    #[test]
    #[should_panic(expected = "input length 2 != output length 1")]
    fn given_row_length_mismatch_when_validate_then_panic() {
        let inputs = vec![vec![1.0, 2.0]]; // length 2
        let outputs = vec![vec![0.3]]; // length 1
        validate_training_sets(&inputs, &outputs);
    }
}
