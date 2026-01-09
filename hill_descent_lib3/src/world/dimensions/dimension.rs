//! Single dimension (axis) of the search space.

use std::ops::RangeInclusive;

/// A single axis in the simulation space with its bounds and doubling count.
#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    /// The range of valid values for this dimension
    range: RangeInclusive<f64>,
    /// Number of times this dimension has been subdivided (for region calculation)
    number_of_doublings: usize,
}

impl Dimension {
    /// Creates a new dimension with the given range.
    ///
    /// The dimension starts with 0 doublings (single interval).
    ///
    /// # Panics
    ///
    /// Panics if the range end is less than the range start.
    pub fn new(range: RangeInclusive<f64>) -> Self {
        assert!(
            *range.end() >= *range.start(),
            "Dimension max must be greater than or equal to min. Start: {}, End: {}",
            range.start(),
            range.end()
        );
        Self {
            range,
            number_of_doublings: 0,
        }
    }

    /// Returns the range of the dimension.
    pub fn range(&self) -> &RangeInclusive<f64> {
        &self.range
    }

    /// Returns the number of times the dimension has been doubled (split in half).
    pub fn number_of_doublings(&self) -> usize {
        self.number_of_doublings
    }

    /// Sets the number of times the dimension has been doubled (split in half).
    pub fn set_number_of_doublings(&mut self, new_doublings: usize) {
        self.number_of_doublings = new_doublings;
    }

    /// Returns the number of intervals the dimension is divided into.
    ///
    /// Number of intervals is 2^d, where d is the number of doublings.
    pub fn num_intervals(&self) -> f64 {
        2.0_f64.powi(self.number_of_doublings as i32)
    }

    /// Sets the range of the dimension.
    ///
    /// # Panics
    ///
    /// Panics if the new range end is less than the range start.
    pub fn set_range(&mut self, new_range: RangeInclusive<f64>) {
        assert!(
            *new_range.end() >= *new_range.start(),
            "Dimension max must be greater than or equal to min. Start: {}, End: {}",
            new_range.start(),
            new_range.end()
        );
        self.range = new_range;
    }

    /// Expands the dimension's range by 50% on each side.
    ///
    /// If the range has zero width (start == end), expands by a fixed amount of 0.5.
    pub fn expand_bounds(&mut self) {
        let start = *self.range.start();
        let end = *self.range.end();
        let width = end - start;

        if width == 0.0 {
            // If the range has no width, expand by a fixed amount.
            self.range = (start - 0.5)..=(end + 0.5);
        } else {
            let expansion = width / 2.0;
            self.range = (start - expansion)..=(end + expansion);
        }
    }

    /// Returns which 0-indexed interval a value falls into.
    ///
    /// The dimension is divided into 2^number_of_doublings intervals.
    /// Each doubling splits all existing intervals in half.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to check
    ///
    /// # Returns
    ///
    /// * `Some(interval)` - The 0-based interval index
    /// * `None` - If the value is outside the dimension's range
    pub fn get_interval(&self, value: f64) -> Option<usize> {
        let start = *self.range.start();
        let end = *self.range.end();

        // Check if value is outside the range
        if value < start || value > end {
            return None;
        }

        // Handle special case: single point range or no doublings.
        if start == end || self.number_of_doublings == 0 {
            return Some(0);
        }

        // The number of intervals is 2^doublings.
        let num_intervals = self.num_intervals();
        let interval_size = (end - start) / num_intervals;

        // Handle case where range is tiny and interval_size is zero.
        if interval_size == 0.0 {
            // If size is 0, all values are effectively at the start,
            // except for the exact end value.
            return if value == end {
                Some((num_intervals as usize).saturating_sub(1))
            } else {
                Some(0)
            };
        }

        // Calculate which interval the value falls into.
        let pre_clamp_interval_float = (value - start) / interval_size;
        let mut interval = pre_clamp_interval_float.floor() as usize;

        // Clamp the interval to the max index, which is `num_intervals - 1`.
        // This handles the `value == end` case correctly, as it might calculate
        // to `num_intervals` due to floating point representation, and ensures
        // it falls into the last interval.
        let max_interval = (num_intervals as usize).saturating_sub(1);
        if interval > max_interval {
            interval = max_interval;
        }

        Some(interval)
    }

    /// Returns the bounds (start, end) for a given interval index.
    ///
    /// This is the inverse of `get_interval` - given an interval index,
    /// returns the range of values that would fall into that interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - The 0-based interval index
    ///
    /// # Returns
    ///
    /// * `Some((start, end))` - The bounds of the interval
    /// * `None` - If the interval index is out of range
    pub fn interval_bounds(&self, interval: usize) -> Option<(f64, f64)> {
        let num_intervals = self.num_intervals() as usize;

        // Check if interval is out of range
        if interval >= num_intervals {
            return None;
        }

        let start = *self.range.start();
        let end = *self.range.end();

        // Handle special case: single interval (no doublings or single point)
        if num_intervals == 1 || start == end {
            return Some((start, end));
        }

        let interval_size = (end - start) / num_intervals as f64;
        let interval_start = start + interval as f64 * interval_size;
        let interval_end = if interval == num_intervals - 1 {
            end // Last interval ends exactly at range end
        } else {
            interval_start + interval_size
        };

        Some((interval_start, interval_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== new() tests ====================

    #[test]
    fn given_valid_range_when_new_then_dimension_created() {
        let dim = Dimension::new(-10.0..=10.0);
        assert_eq!(*dim.range().start(), -10.0);
        assert_eq!(*dim.range().end(), 10.0);
        assert_eq!(dim.number_of_doublings(), 0);
    }

    #[test]
    #[should_panic(expected = "Dimension max must be greater than or equal to min")]
    fn given_invalid_range_when_new_then_panics() {
        Dimension::new(10.0..=-10.0);
    }

    #[test]
    fn given_equal_range_when_new_then_dimension_created() {
        let dim = Dimension::new(5.0..=5.0);
        assert_eq!(*dim.range().start(), 5.0);
        assert_eq!(*dim.range().end(), 5.0);
    }

    // ==================== num_intervals() tests ====================

    #[test]
    fn given_zero_doublings_when_num_intervals_then_returns_one() {
        let dim = Dimension::new(0.0..=1.0);
        assert_eq!(dim.num_intervals(), 1.0);
    }

    #[test]
    fn given_one_doubling_when_num_intervals_then_returns_two() {
        let mut dim = Dimension::new(0.0..=1.0);
        dim.set_number_of_doublings(1);
        assert_eq!(dim.num_intervals(), 2.0);
    }

    #[test]
    fn given_three_doublings_when_num_intervals_then_returns_eight() {
        let mut dim = Dimension::new(0.0..=1.0);
        dim.set_number_of_doublings(3);
        assert_eq!(dim.num_intervals(), 8.0);
    }

    // ==================== expand_bounds() tests ====================

    #[test]
    fn given_dimension_with_zero_width_when_expand_bounds_then_range_expands_by_fixed_amount() {
        let mut dim = Dimension::new(0.0..=0.0);
        dim.expand_bounds();
        assert_eq!(*dim.range(), -0.5..=0.5);
    }

    #[test]
    fn given_dimension_with_non_zero_width_when_expand_bounds_then_range_expands_by_50_percent() {
        let mut dim = Dimension::new(10.0..=20.0);
        dim.expand_bounds();
        assert_eq!(*dim.range(), 5.0..=25.0);
    }

    #[test]
    fn given_dimension_with_negative_range_when_expand_bounds_then_range_expands_correctly() {
        let mut dim = Dimension::new(-10.0..=-5.0);
        dim.expand_bounds();
        // Width is 5.0, expansion is 2.5
        assert_eq!(*dim.range(), -12.5..=-2.5);
    }

    // ==================== get_interval() tests ====================

    #[test]
    fn given_value_within_range_with_one_doubling_when_get_interval_then_returns_correct_interval()
    {
        // Range 0..=10 with 1 doubling (2^1 = 2 intervals of size 5)
        // Intervals: [0, 5), [5, 10]
        let mut dim = Dimension::new(0.0..=10.0);
        dim.set_number_of_doublings(1);

        assert_eq!(dim.get_interval(0.0), Some(0));
        assert_eq!(dim.get_interval(4.999), Some(0));
        assert_eq!(dim.get_interval(5.0), Some(1));
        assert_eq!(dim.get_interval(10.0), Some(1));
    }

    #[test]
    fn given_value_within_range_with_three_doublings_when_get_interval_then_returns_correct_interval()
     {
        // Range 0..=10 with 3 doublings (2^3 = 8 intervals of size 1.25)
        let mut dim = Dimension::new(0.0..=10.0);
        dim.set_number_of_doublings(3);

        assert_eq!(dim.get_interval(0.0), Some(0));
        assert_eq!(dim.get_interval(1.0), Some(0));
        assert_eq!(dim.get_interval(1.25), Some(1));
        assert_eq!(dim.get_interval(2.5), Some(2));
        assert_eq!(dim.get_interval(6.0), Some(4));
        assert_eq!(dim.get_interval(10.0), Some(7));
    }

    #[test]
    fn given_value_out_of_bounds_when_get_interval_then_returns_none() {
        let mut dim = Dimension::new(0.0..=10.0);
        dim.set_number_of_doublings(5);

        assert_eq!(dim.get_interval(-0.1), None); // Below range
        assert_eq!(dim.get_interval(10.1), None); // Above range
    }

    #[test]
    fn given_zero_doublings_when_get_interval_then_all_values_in_interval_zero() {
        let dim = Dimension::new(0.0..=10.0);

        assert_eq!(dim.get_interval(0.0), Some(0));
        assert_eq!(dim.get_interval(5.0), Some(0));
        assert_eq!(dim.get_interval(10.0), Some(0));
    }

    #[test]
    fn given_single_point_range_when_get_interval_then_returns_zero_for_exact_value() {
        let dim = Dimension::new(5.0..=5.0);

        assert_eq!(dim.get_interval(5.0), Some(0));
        assert_eq!(dim.get_interval(4.9), None);
        assert_eq!(dim.get_interval(5.1), None);
    }

    #[test]
    fn given_single_point_range_with_doublings_when_get_interval_then_returns_zero() {
        let mut dim = Dimension::new(5.0..=5.0);
        dim.set_number_of_doublings(5);

        // In a single point range, any value that's in range must be in interval 0
        assert_eq!(dim.get_interval(5.0), Some(0));
    }

    #[test]
    fn given_two_doublings_when_get_interval_at_boundaries_then_returns_correct_intervals() {
        // Range 0..=10 with 2 doublings (2^2 = 4 intervals of size 2.5)
        // Intervals: [0, 2.5), [2.5, 5), [5, 7.5), [7.5, 10]
        let mut dim = Dimension::new(0.0..=10.0);
        dim.set_number_of_doublings(2);

        assert_eq!(dim.get_interval(0.0), Some(0));
        assert_eq!(dim.get_interval(2.5), Some(1));
        assert_eq!(dim.get_interval(5.0), Some(2));
        assert_eq!(dim.get_interval(7.5), Some(3));
        assert_eq!(dim.get_interval(10.0), Some(3)); // End boundary

        // Just before/after boundaries
        assert_eq!(dim.get_interval(2.4), Some(0));
        assert_eq!(dim.get_interval(2.6), Some(1));
        assert_eq!(dim.get_interval(9.999), Some(3));
    }

    #[test]
    fn given_negative_range_when_get_interval_then_returns_correct_intervals() {
        // Range -10..=-5 with 5 doublings (2^5 = 32 intervals)
        let mut dim = Dimension::new(-10.0..=-5.0);
        dim.set_number_of_doublings(5);

        assert_eq!(dim.get_interval(-10.0), Some(0));
        assert_eq!(dim.get_interval(-7.5), Some(16)); // Middle of range
        assert_eq!(dim.get_interval(-5.0), Some(31));

        // Out of bounds
        assert_eq!(dim.get_interval(-10.1), None);
        assert_eq!(dim.get_interval(-4.9), None);
    }

    #[test]
    fn given_mixed_range_when_get_interval_then_handles_zero_crossing() {
        // Range -5..=5 with 10 doublings (2^10 = 1024 intervals)
        let mut dim = Dimension::new(-5.0..=5.0);
        dim.set_number_of_doublings(10);

        assert_eq!(dim.get_interval(-5.0), Some(0));
        assert_eq!(dim.get_interval(0.0), Some(512)); // Middle
        assert_eq!(dim.get_interval(5.0), Some(1023));
    }

    #[test]
    fn given_many_doublings_when_get_interval_then_handles_floating_point_precision() {
        // Range 0..=1 with 10 doublings (2^10 = 1024 intervals)
        let mut dim = Dimension::new(0.0..=1.0);
        dim.set_number_of_doublings(10);

        assert_eq!(dim.get_interval(0.1), Some(102));
        assert_eq!(dim.get_interval(0.2), Some(204));
        assert_eq!(dim.get_interval(0.3), Some(307));

        // Floating point accumulation: 0.1 + 0.1 + 0.1 might be ~0.30000000000000004
        let value = 0.1 + 0.1 + 0.1;
        assert_eq!(dim.get_interval(value), Some(307));
    }
}
