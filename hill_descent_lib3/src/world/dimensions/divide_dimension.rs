//! Dimension subdivision for region refinement.
//!
//! This module provides the ability to divide dimensions by doubling the number
//! of intervals, which increases the spatial resolution of the search space.

use super::Dimensions;

impl Dimensions {
    /// Divides the dimension at `dim_idx` by doubling the number of intervals, if precision allows.
    ///
    /// Increments the `number_of_doublings` by 1, which doubles the total intervals (2^doublings).
    ///
    /// # Arguments
    /// * `dim_idx` - The index of the dimension to divide.
    ///
    /// # Returns
    /// * `true` if the division was successful.
    /// * `false` if the division would result in intervals smaller than the floating-point precision allows.
    ///
    /// # Panics
    /// * Panics if `dim_idx` is out of bounds or if there are no defined dimensions.
    pub fn divide_dimension(&mut self, dim_idx: usize) -> bool {
        assert!(
            !self.dimensions.is_empty(),
            "divide_dimension called on empty Dimensions set"
        );
        assert!(
            dim_idx < self.dimensions.len(),
            "dim_idx {} out of bounds: {} dimensions",
            dim_idx,
            self.dimensions.len()
        );

        let dim = &mut self.dimensions[dim_idx];
        let current_doublings = dim.number_of_doublings();
        let new_doublings = current_doublings + 1;

        // Calculate the width of one of the new intervals
        let num_new_intervals = 2.0_f64.powi(new_doublings as i32);
        let range = dim.range();
        let range_width = range.end() - range.start();
        let new_interval_width = range_width / num_new_intervals;

        // Check for catastrophic loss of precision. If adding the new interval width
        // to the start of the range doesn't change the value, we've hit the f64 limit.
        if *range.start() + new_interval_width == *range.start() {
            return false;
        }

        dim.set_number_of_doublings(new_doublings);
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::world::dimensions::Dimensions;
    use crate::world::dimensions::dimension::Dimension;

    #[test]
    fn given_zero_doublings_when_divide_dimension_then_becomes_one() {
        let mut dims =
            Dimensions::new_for_test(vec![Dimension::new(0.0..=1.0), Dimension::new(0.0..=1.0)]);
        dims.get_dimension_mut(1).set_number_of_doublings(0);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 0);
        let result = dims.divide_dimension(1);
        assert!(result);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 1);
    }

    #[test]
    fn given_non_zero_doublings_when_divide_dimension_then_increments() {
        let mut dims =
            Dimensions::new_for_test(vec![Dimension::new(0.0..=1.0), Dimension::new(0.0..=1.0)]);
        dims.get_dimension_mut(0).set_number_of_doublings(1);
        dims.get_dimension_mut(1).set_number_of_doublings(2);

        // Test incrementing from 1 to 2
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 1);
        assert!(dims.divide_dimension(0));
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 2);

        // Test incrementing from 2 to 3
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 2);
        assert!(dims.divide_dimension(1));
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 3);
    }

    #[test]
    fn given_sequence_when_divide_dimension_then_doublings_increment_by_one() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=1.0)]);

        // Test the sequence: 0 -> 1 -> 2 -> 3 -> 4 (doublings increment by 1)
        // Actual intervals: 1 -> 2 -> 4 -> 8 -> 16 (2^doublings)
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 0);
        assert_eq!(dims.get_dimension(0).num_intervals(), 1.0);

        assert!(dims.divide_dimension(0));
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 1);
        assert_eq!(dims.get_dimension(0).num_intervals(), 2.0);

        assert!(dims.divide_dimension(0));
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 2);
        assert_eq!(dims.get_dimension(0).num_intervals(), 4.0);

        assert!(dims.divide_dimension(0));
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 3);
        assert_eq!(dims.get_dimension(0).num_intervals(), 8.0);

        assert!(dims.divide_dimension(0));
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 4);
        assert_eq!(dims.get_dimension(0).num_intervals(), 16.0);
    }

    #[test]
    fn given_precision_at_limit_when_divide_dimension_then_returns_false() {
        // f64 has about 53 bits of significand precision.
        // If we make 53 doublings, we have 2^53 intervals. The interval width will be
        // (range / 2^53). Adding this to the start of the range might result in the same number
        // if the range start is 1.0, as 1.0 + 1.0/2^53 might be indistinguishable from 1.0.
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(1.0..=2.0)]);
        dims.get_dimension_mut(0).set_number_of_doublings(51);

        // The 52nd doubling should be fine.
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 51);
        assert!(dims.divide_dimension(0));
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 52);

        // The 53rd should fail, as the interval becomes too small to be represented.
        let result = dims.divide_dimension(0);
        assert!(!result);
        // The number of doublings should not have changed.
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 52);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn given_out_of_bounds_index_when_divide_dimension_then_panics() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=1.0)]);
        dims.divide_dimension(5);
    }

    #[test]
    #[should_panic(expected = "empty Dimensions set")]
    fn given_empty_dimensions_when_divide_dimension_then_panics() {
        let mut dims = Dimensions::new_for_test(vec![]);
        dims.divide_dimension(0);
    }
}
