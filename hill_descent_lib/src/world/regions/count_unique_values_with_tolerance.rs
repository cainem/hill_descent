use std::cmp::Ordering;

/// Counts unique values in a sorted slice using relative tolerance for floating point comparison.
///
/// This function handles floating point precision issues by using relative tolerance
/// based on the magnitude of values being compared, preventing false diversity detection
/// in optimization algorithms.
///
/// # Arguments
/// * `values` - A slice of f64 values to count unique values from
///
/// # Returns
/// The count of unique values considering floating point tolerance
pub fn count_unique_values_with_tolerance(values: &[f64]) -> usize {
    if values.is_empty() {
        return 0;
    }

    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    // Use relative tolerance based on f64 machine epsilon
    const RELATIVE_TOLERANCE: f64 = 100.0 * f64::EPSILON; // ≈ 2.22 × 10^-14
    const ABSOLUTE_MIN_TOLERANCE: f64 = 1000.0 * f64::EPSILON; // ≈ 2.22 × 10^-13

    let mut unique_count = 0;
    let mut last_value: Option<f64> = None;

    for value in sorted_values {
        if let Some(prev) = last_value {
            // Use relative tolerance based on the larger magnitude
            let magnitude = value.abs().max(prev.abs());
            let tolerance = magnitude.max(ABSOLUTE_MIN_TOLERANCE) * RELATIVE_TOLERANCE;

            if (value - prev).abs() < tolerance {
                // Values are too close relative to their magnitude, treat as same
                continue;
            }
        }
        unique_count += 1;
        last_value = Some(value);
    }

    unique_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_values_when_count_unique_then_returns_zero() {
        assert_eq!(count_unique_values_with_tolerance(&[]), 0);
    }

    #[test]
    fn given_single_value_when_count_unique_then_returns_one() {
        assert_eq!(count_unique_values_with_tolerance(&[5.0]), 1);
    }

    #[test]
    fn given_identical_values_when_count_unique_then_returns_one() {
        assert_eq!(count_unique_values_with_tolerance(&[1.0, 1.0, 1.0]), 1);
    }

    #[test]
    fn given_different_values_when_count_unique_then_returns_correct_count() {
        assert_eq!(count_unique_values_with_tolerance(&[1.0, 2.0, 3.0]), 3);
    }

    #[test]
    fn given_close_values_when_count_unique_then_treats_as_same() {
        // Values within tolerance should be treated as same
        assert_eq!(
            count_unique_values_with_tolerance(&[1.0, 1.000000000000001]),
            1
        );
    }

    #[test]
    fn given_distant_values_when_count_unique_then_treats_as_different() {
        // Values outside tolerance should be treated as different
        assert_eq!(count_unique_values_with_tolerance(&[1.0, 1.01]), 2);
    }

    #[test]
    fn given_mixed_magnitudes_when_count_unique_then_handles_correctly() {
        // Test with very small and very large values
        let values = vec![1e-15, 1.000000000000001e-15, 1e15, 1.000000000000001e15];
        assert_eq!(count_unique_values_with_tolerance(&values), 2);
    }
}
