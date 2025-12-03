//! Dimensions constructor.

use std::ops::RangeInclusive;

use super::{Dimension, Dimensions};

impl Dimensions {
    /// Creates new Dimensions from parameter bounds.
    ///
    /// Each dimension starts with 0 doublings (a single interval covering the entire range).
    /// The version is initialized to 0.
    ///
    /// # Arguments
    ///
    /// * `param_range` - Slice of ranges for each parameter dimension
    ///
    /// # Returns
    ///
    /// A new Dimensions with version 0 and all dimensions having 0 doublings.
    pub fn new(param_range: &[RangeInclusive<f64>]) -> Self {
        let dimensions: Vec<Dimension> = param_range
            .iter()
            .map(|bounds| Dimension::new(bounds.clone()))
            .collect();

        Self {
            dimensions,
            version: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_parameter_bounds_when_new_then_no_dimensions_created() {
        let bounds: Vec<RangeInclusive<f64>> = vec![];
        let dims = Dimensions::new(&bounds);

        assert!(dims.get_dimensions().is_empty());
        assert_eq!(dims.version(), 0);
    }

    #[test]
    fn given_single_bound_when_new_then_one_dimension_created_with_zero_doublings() {
        let bounds = vec![0.0..=10.0];
        let dims = Dimensions::new(&bounds);

        assert_eq!(dims.num_dimensions(), 1);
        assert_eq!(*dims.get_dimension(0).range(), 0.0..=10.0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 0);
        assert_eq!(dims.get_dimension(0).num_intervals(), 1.0);
        assert_eq!(dims.version(), 0);
    }

    #[test]
    fn given_multiple_bounds_when_new_then_all_dimensions_created_with_zero_doublings() {
        let bounds = vec![0.0..=10.0, -5.0..=5.0, 100.0..=200.0];
        let dims = Dimensions::new(&bounds);

        assert_eq!(dims.num_dimensions(), 3);
        assert_eq!(dims.version(), 0);

        // Check first dimension
        assert_eq!(*dims.get_dimension(0).range().start(), 0.0);
        assert_eq!(*dims.get_dimension(0).range().end(), 10.0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 0);

        // Check second dimension
        assert_eq!(*dims.get_dimension(1).range().start(), -5.0);
        assert_eq!(*dims.get_dimension(1).range().end(), 5.0);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 0);

        // Check third dimension
        assert_eq!(*dims.get_dimension(2).range().start(), 100.0);
        assert_eq!(*dims.get_dimension(2).range().end(), 200.0);
        assert_eq!(dims.get_dimension(2).number_of_doublings(), 0);
    }

    #[test]
    #[should_panic(expected = "Dimension max must be greater than or equal to min")]
    fn given_bounds_with_invalid_range_when_new_then_dimension_panics() {
        let bounds = vec![0.0..=5.0, 10.0..=0.0]; // Second range is invalid
        Dimensions::new(&bounds);
    }

    #[test]
    fn given_bounds_with_equal_start_end_when_new_then_dimension_created() {
        let bounds = vec![5.0..=5.0]; // Valid range with equal start and end
        let dims = Dimensions::new(&bounds);

        assert_eq!(dims.num_dimensions(), 1);
        assert_eq!(*dims.get_dimension(0).range(), 5.0..=5.0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 0);
    }

    #[test]
    fn given_bounds_with_negative_values_when_new_then_dimensions_created_correctly() {
        let bounds = vec![-10.0..=-1.0, -100.0..=100.0];
        let dims = Dimensions::new(&bounds);

        assert_eq!(dims.num_dimensions(), 2);
        assert_eq!(*dims.get_dimension(0).range(), -10.0..=-1.0);
        assert_eq!(*dims.get_dimension(1).range(), -100.0..=100.0);
        assert_eq!(dims.version(), 0);
    }
}
