use super::dimension::Dimension;

/// Represents the result of calculating a dimension key.
#[derive(Debug, PartialEq, Clone)]
pub enum CalculateDimensionsKeyResult {
    /// The key was successfully calculated.
    Success(Vec<usize>),
    /// The calculation failed because a value was out of bounds for a dimension.
    Failure {
        /// The index of the dimension where the failure occurred.
        dimension_index: usize,
        /// The value that was out of bounds.
        value: f64,
    },
}

/// Calculates a dimension key for a set of expressed values.
///
/// The key is a vector of interval indices, one for each dimension, that
/// corresponds to the region where the expressed values fall.
///
/// # Parameters
/// * `dimensions`: A slice of `Dimension`s that define the space.
/// * `expressed_values`: A slice of `f64` values to be mapped to the dimensions.
///
/// # Returns
/// * `CalculateDimensionsKeyResult`: Either a `Success` with the dimension key
///   or a `Failure` indicating which value was out of bounds for which dimension.
///
/// # Panics
/// * Panics if the number of dimensions does not match the number of expressed values.
pub fn calculate_dimensions_key(
    dimensions: &[Dimension],
    expressed_values: &[f64],
) -> CalculateDimensionsKeyResult {
    if dimensions.len() != expressed_values.len() {
        panic!("The number of dimensions must match the number of expressed values.");
    }

    let mut key = Vec::with_capacity(dimensions.len());

    for (i, (dimension, &value)) in dimensions.iter().zip(expressed_values.iter()).enumerate() {
        match dimension.get_interval(value) {
            Some(interval) => key.push(interval),
            None => {
                return CalculateDimensionsKeyResult::Failure {
                    dimension_index: i,
                    value,
                };
            }
        }
    }

    CalculateDimensionsKeyResult::Success(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_calculate_dimensions_key_is_called_then_success_is_returned() {
        let dimensions = vec![
            Dimension::new(0.0..=10.0, 2),  // 2^2 = 4 intervals of size 2.5 each
            Dimension::new(0.0..=100.0, 4), // 2^4 = 16 intervals of size 6.25 each
        ];
        let expressed_values = vec![7.5, 60.0];
        let result = calculate_dimensions_key(&dimensions, &expressed_values);
        // 7.5 falls into interval 3 (7.5/2.5 = 3.0), 60.0 falls into interval 9 (60.0/6.25 = 9.6)
        assert_eq!(result, CalculateDimensionsKeyResult::Success(vec![3, 9]));
    }

    #[test]
    fn given_values_on_boundaries_when_calculate_dimensions_key_is_called_then_success_is_returned()
    {
        let dimensions = vec![
            Dimension::new(0.0..=10.0, 2),
            Dimension::new(0.0..=100.0, 4),
        ];
        let expressed_values = vec![10.0, 100.0];
        let result = calculate_dimensions_key(&dimensions, &expressed_values);
        // 10.0 is in interval 3 (the last interval, 0-indexed)
        // 100.0 is in interval 15 (the last interval, 0-indexed)
        assert_eq!(result, CalculateDimensionsKeyResult::Success(vec![3, 15]));
    }

    #[test]
    fn given_value_out_of_bounds_when_calculate_dimensions_key_is_called_then_failure_is_returned()
    {
        let dimensions = vec![
            Dimension::new(0.0..=10.0, 2),
            Dimension::new(0.0..=100.0, 4),
        ];
        let expressed_values = vec![7.5, 101.0]; // 101.0 is out of bounds for the second dimension
        let result = calculate_dimensions_key(&dimensions, &expressed_values);
        assert_eq!(
            result,
            CalculateDimensionsKeyResult::Failure {
                dimension_index: 1,
                value: 101.0
            }
        );
    }

    #[test]
    fn given_first_value_out_of_bounds_when_calculate_dimensions_key_is_called_then_failure_is_returned()
     {
        let dimensions = vec![
            Dimension::new(0.0..=10.0, 2),
            Dimension::new(0.0..=100.0, 4),
        ];
        let expressed_values = vec![-1.0, 50.0]; // -1.0 is out of bounds
        let result = calculate_dimensions_key(&dimensions, &expressed_values);
        assert_eq!(
            result,
            CalculateDimensionsKeyResult::Failure {
                dimension_index: 0,
                value: -1.0
            }
        );
    }

    #[test]
    #[should_panic(
        expected = "The number of dimensions must match the number of expressed values."
    )]
    fn given_mismatched_lengths_when_calculate_dimensions_key_is_called_then_it_panics() {
        let dimensions = vec![Dimension::new(0.0..=10.0, 2)];
        let expressed_values = vec![5.0, 50.0];
        calculate_dimensions_key(&dimensions, &expressed_values);
    }

    #[test]
    fn given_empty_inputs_when_calculate_dimensions_key_is_called_then_success_with_empty_vec_is_returned()
     {
        let dimensions: Vec<Dimension> = vec![];
        let expressed_values: Vec<f64> = vec![];
        let result = calculate_dimensions_key(&dimensions, &expressed_values);
        assert_eq!(result, CalculateDimensionsKeyResult::Success(vec![]));
    }
}
