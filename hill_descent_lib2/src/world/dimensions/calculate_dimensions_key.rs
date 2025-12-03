//! Region key calculation from phenotype values and dimensions.

use super::Dimensions;
use crate::world::regions::region_key::RegionKey;

/// Result of calculating a dimensions key (region key).
#[derive(Debug, Clone, PartialEq)]
pub enum CalculateDimensionsKeyResult {
    /// Key calculated successfully
    Ok(RegionKey),
    /// Value is outside dimension bounds
    OutOfBounds {
        /// Indices of dimensions where values exceed bounds
        dimensions_exceeded: Vec<usize>,
    },
}

/// Calculates the region key for given phenotype values.
///
/// For each dimension, determines which interval the corresponding value
/// falls into. If any value is outside its dimension's bounds, returns
/// `OutOfBounds` with all the exceeded dimension indices.
///
/// # Arguments
///
/// * `expressed_values` - The expressed phenotype values (one per dimension)
/// * `dimensions` - The current dimensions
///
/// # Returns
///
/// * `Ok(RegionKey)` - The calculated region key if all values are in bounds
/// * `OutOfBounds` - If any value exceeds dimension bounds, with list of exceeded indices
///
/// # Panics
///
/// Panics if `expressed_values.len() != dimensions.num_dimensions()`.
pub fn calculate_dimensions_key(
    expressed_values: &[f64],
    dimensions: &Dimensions,
) -> CalculateDimensionsKeyResult {
    assert_eq!(
        expressed_values.len(),
        dimensions.num_dimensions(),
        "Number of expressed values ({}) must match number of dimensions ({})",
        expressed_values.len(),
        dimensions.num_dimensions()
    );

    let mut intervals = Vec::with_capacity(dimensions.num_dimensions());
    let mut exceeded_dimensions = Vec::new();

    for (idx, (&value, dimension)) in expressed_values
        .iter()
        .zip(dimensions.get_dimensions().iter())
        .enumerate()
    {
        match dimension.get_interval(value) {
            Some(interval) => intervals.push(interval),
            None => exceeded_dimensions.push(idx),
        }
    }

    if exceeded_dimensions.is_empty() {
        CalculateDimensionsKeyResult::Ok(RegionKey::new(intervals))
    } else {
        CalculateDimensionsKeyResult::OutOfBounds {
            dimensions_exceeded: exceeded_dimensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::dimensions::Dimension;

    fn create_test_dimensions() -> Dimensions {
        // 2D space: x in [0, 10], y in [-5, 5]
        Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0), Dimension::new(-5.0..=5.0)])
    }

    fn create_test_dimensions_with_doublings() -> Dimensions {
        // 2D space with subdivisions
        let mut dim_x = Dimension::new(0.0..=10.0);
        dim_x.set_number_of_doublings(2); // 4 intervals: [0,2.5), [2.5,5), [5,7.5), [7.5,10]
        let mut dim_y = Dimension::new(-5.0..=5.0);
        dim_y.set_number_of_doublings(1); // 2 intervals: [-5,0), [0,5]
        Dimensions::new_for_test(vec![dim_x, dim_y])
    }

    // ==================== Success cases ====================

    #[test]
    fn given_values_within_bounds_when_calculate_then_returns_ok() {
        let dims = create_test_dimensions();
        let values = vec![5.0, 0.0];

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::Ok(key) => {
                // With 0 doublings, all values map to interval 0
                assert_eq!(key.values(), &[0, 0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_values_with_doublings_when_calculate_then_returns_correct_intervals() {
        let dims = create_test_dimensions_with_doublings();

        // x=7.5 is in interval 3 (last of 4 intervals)
        // y=2.5 is in interval 1 (last of 2 intervals)
        let values = vec![7.5, 2.5];

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[3, 1]);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_values_at_boundaries_when_calculate_then_returns_ok() {
        let dims = create_test_dimensions();

        // Boundary values: start of x range, end of y range
        let values = vec![0.0, 5.0];

        let result = calculate_dimensions_key(&values, &dims);

        assert!(matches!(result, CalculateDimensionsKeyResult::Ok(_)));
    }

    #[test]
    fn given_values_at_end_boundaries_when_calculate_then_returns_ok() {
        let dims = create_test_dimensions();

        // End boundary values
        let values = vec![10.0, -5.0];

        let result = calculate_dimensions_key(&values, &dims);

        assert!(matches!(result, CalculateDimensionsKeyResult::Ok(_)));
    }

    // ==================== Out of bounds cases ====================

    #[test]
    fn given_value_below_min_when_calculate_then_returns_out_of_bounds() {
        let dims = create_test_dimensions();
        let values = vec![-0.1, 0.0]; // x is below min

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::OutOfBounds {
                dimensions_exceeded,
            } => {
                assert_eq!(dimensions_exceeded, vec![0]);
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_value_above_max_when_calculate_then_returns_out_of_bounds() {
        let dims = create_test_dimensions();
        let values = vec![5.0, 5.1]; // y is above max

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::OutOfBounds {
                dimensions_exceeded,
            } => {
                assert_eq!(dimensions_exceeded, vec![1]);
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    #[test]
    fn given_multiple_out_of_bounds_when_calculate_then_returns_all_exceeded() {
        let dims = create_test_dimensions();
        let values = vec![-0.1, 5.1]; // Both out of bounds

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::OutOfBounds {
                dimensions_exceeded,
            } => {
                assert_eq!(dimensions_exceeded, vec![0, 1]);
            }
            _ => panic!("Expected OutOfBounds result"),
        }
    }

    // ==================== Edge cases ====================

    #[test]
    fn given_empty_dimensions_when_calculate_then_returns_empty_key() {
        let dims = Dimensions::new_for_test(vec![]);
        let values: Vec<f64> = vec![];

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::Ok(key) => {
                assert!(key.values().is_empty());
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    #[should_panic(expected = "Number of expressed values")]
    fn given_mismatched_lengths_when_calculate_then_panics() {
        let dims = create_test_dimensions();
        let values = vec![5.0]; // Only 1 value for 2 dimensions

        calculate_dimensions_key(&values, &dims);
    }

    #[test]
    fn given_single_dimension_when_calculate_then_returns_correct_key() {
        let dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=100.0)]);
        let values = vec![50.0];

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn given_many_dimensions_when_calculate_then_returns_correct_key() {
        // 5 dimensions
        let dims = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=10.0),
            Dimension::new(0.0..=10.0),
            Dimension::new(0.0..=10.0),
            Dimension::new(0.0..=10.0),
            Dimension::new(0.0..=10.0),
        ]);
        let values = vec![5.0, 5.0, 5.0, 5.0, 5.0];

        let result = calculate_dimensions_key(&values, &dims);

        match result {
            CalculateDimensionsKeyResult::Ok(key) => {
                assert_eq!(key.values(), &[0, 0, 0, 0, 0]);
            }
            _ => panic!("Expected Ok result"),
        }
    }
}
