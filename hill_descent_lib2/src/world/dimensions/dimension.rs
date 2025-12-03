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

    /// Expands the bounds of this dimension.
    ///
    /// The expansion factor and logic should match the original implementation.
    pub fn expand_bounds(&mut self) {
        todo!("Implement expand_bounds")
    }

    /// Returns which interval a value falls into.
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
        todo!("Implement get_interval")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
