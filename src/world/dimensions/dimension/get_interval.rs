use crate::world::dimensions::dimension::Dimension;

impl Dimension {
    /// Determines which 0-indexed interval a given value falls into.
    ///
    /// The dimension is divided into 2^number_of_doublings intervals.
    /// Each doubling splits all existing intervals in half.
    ///
    /// # Parameters
    /// * `value`: The value to check.
    ///
    /// # Returns
    /// * `Some(usize)`: The interval index if the value is within the dimension's range.
    /// * `None`: If the value is outside the dimension's range.
    pub fn get_interval(&self, value: f64) -> Option<usize> {
        let range = self.range();
        let start = *range.start();
        let end = *range.end();
        let doublings = self.number_of_doublings();

        // Check if value is outside the range
        if value < start || value > end {
            return None;
        }

        // Handle special case: single point range or no doublings.
        if start == end || doublings == 0 {
            return Some(0);
        }

        // The number of intervals is 2^doublings.
        let num_intervals = self.num_intervals() as f64;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interval_basic_and_boundaries() {
        // Range 0..=10 with 1 doubling (2^1 = 2 intervals of size 5)
        // Intervals: [0, 5), [5, 10]
        let dimension1 = Dimension {
            range: 0.0..=10.0,
            number_of_doublings: 1,
        };
        assert_eq!(dimension1.get_interval(0.0), Some(0));
        assert_eq!(dimension1.get_interval(4.999), Some(0));
        assert_eq!(dimension1.get_interval(5.0), Some(1));
        assert_eq!(dimension1.get_interval(10.0), Some(1));

        // Range 0..=10 with 3 doublings (2^3 = 8 intervals of size 1.25)
        // Intervals: [0, 1.25), [1.25, 2.5), [2.5, 3.75), [3.75, 5), [5, 6.25), [6.25, 7.5), [7.5, 8.75), [8.75, 10]
        let dimension2 = Dimension {
            range: 0.0..=10.0,
            number_of_doublings: 3,
        };
        assert_eq!(dimension2.get_interval(0.0), Some(0));
        assert_eq!(dimension2.get_interval(1.0), Some(0));
        assert_eq!(dimension2.get_interval(1.25), Some(1));
        assert_eq!(dimension2.get_interval(2.5), Some(2));
        assert_eq!(dimension2.get_interval(6.0), Some(4));
        assert_eq!(dimension2.get_interval(10.0), Some(7));
    }

    #[test]
    fn test_get_interval_out_of_bounds() {
        let dimension = Dimension {
            range: 0.0..=10.0,
            number_of_doublings: 5,
        };

        assert_eq!(dimension.get_interval(-0.1), None); // Below range
        assert_eq!(dimension.get_interval(10.1), None); // Above range
    }

    #[test]
    fn test_get_interval_zero_doublings() {
        // Range 0..=10 with 0 doublings (2^0 = 1 interval covering the entire range)
        let dimension = Dimension {
            range: 0.0..=10.0,
            number_of_doublings: 0,
        };

        assert_eq!(dimension.get_interval(0.0), Some(0)); // Start of range
        assert_eq!(dimension.get_interval(5.0), Some(0)); // Middle of range
        assert_eq!(dimension.get_interval(10.0), Some(0)); // End of range
    }

    #[test]
    fn test_get_interval_single_point_range() {
        // Single point range 5..=5 with various doublings
        let dimension1 = Dimension {
            range: 5.0..=5.0,
            number_of_doublings: 0,
        };
        let dimension2 = Dimension {
            range: 5.0..=5.0,
            number_of_doublings: 5,
        };

        // In a single point range, any value that's in range (i.e., exactly equal to that point)
        // must be in interval 0, regardless of the number of doublings
        assert_eq!(dimension1.get_interval(5.0), Some(0));
        assert_eq!(dimension2.get_interval(5.0), Some(0));

        // Values outside the single point are out of range
        assert_eq!(dimension1.get_interval(4.9), None);
        assert_eq!(dimension1.get_interval(5.1), None);
    }

    #[test]
    fn test_get_interval_at_boundaries() {
        // Range 0..=10 with 2 doublings (2^2 = 4 intervals of size 2.5)
        // Intervals: [0, 2.5), [2.5, 5), [5, 7.5), [7.5, 10]
        let dimension = Dimension {
            range: 0.0..=10.0,
            number_of_doublings: 2,
        };

        // Test exactly at the boundaries
        assert_eq!(dimension.get_interval(0.0), Some(0)); // Start boundary
        assert_eq!(dimension.get_interval(2.5), Some(1)); // First quarter boundary
        assert_eq!(dimension.get_interval(5.0), Some(2)); // Middle boundary
        assert_eq!(dimension.get_interval(7.5), Some(3)); // Third quarter boundary
        assert_eq!(dimension.get_interval(10.0), Some(3)); // End boundary

        // Test close to but not at boundaries
        assert_eq!(dimension.get_interval(2.4), Some(0)); // Just before first quarter
        assert_eq!(dimension.get_interval(2.6), Some(1)); // Just after first quarter
        assert_eq!(dimension.get_interval(9.999), Some(3)); // Just before end boundary
    }

    #[test]
    fn test_get_interval_negative_range() {
        // Range -10..=-5 with 5 doublings (2^5 = 32 intervals)
        let dimension = Dimension {
            range: -10.0..=-5.0,
            number_of_doublings: 5,
        };

        assert_eq!(dimension.get_interval(-10.0), Some(0)); // Start of range
        // With 32 intervals, interval_size = 5.0/32 = 0.15625
        // (-7.5 - (-10.0)) / interval_size = 2.5 / 0.15625 = 16.0. floor() = 16.
        assert_eq!(dimension.get_interval(-7.5), Some(16)); // Middle of range, falls into interval 16
        assert_eq!(dimension.get_interval(-5.0), Some(31)); // End of range (last interval)

        // Out of bounds
        assert_eq!(dimension.get_interval(-10.1), None); // Below range
        assert_eq!(dimension.get_interval(-4.9), None); // Above range
    }

    #[test]
    fn test_get_interval_mixed_range() {
        // Range -5..=5 with 10 doublings (2^10 = 1024 intervals)
        let dimension = Dimension {
            range: -5.0..=5.0,
            number_of_doublings: 10,
        };

        assert_eq!(dimension.get_interval(-5.0), Some(0)); // Start of range
        assert_eq!(dimension.get_interval(0.0), Some(512)); // Middle of range (1024 intervals, middle is at 512)
        assert_eq!(dimension.get_interval(5.0), Some(1023)); // End of range (last interval)

        // Out of bounds
        assert_eq!(dimension.get_interval(-5.1), None); // Below range
        assert_eq!(dimension.get_interval(5.1), None); // Above range
    }

    #[test]
    fn test_get_interval_floating_point_precision() {
        // Range 0..=1 with 10 doublings (2^10 = 1024 intervals)
        // Each interval is 1.0/1024 ≈ 0.000977 wide
        let dimension = Dimension {
            range: 0.0..=1.0,
            number_of_doublings: 10,
        };

        // Test with values that might have floating point precision issues
        // With 1024 intervals, interval_size = 1.0/1024 ≈ 0.000977
        assert_eq!(dimension.get_interval(0.1), Some(102)); // (0.1 / (1.0/1024)) = 102.4 -> floor(102.4) = 102
        assert_eq!(dimension.get_interval(0.2), Some(204)); // (0.2 / (1.0/1024)) = 204.8 -> floor(204.8) = 204

        // For value = 0.3, (0.3 / (1.0/1024)) = 307.2. floor(307.2) = 307.
        assert_eq!(dimension.get_interval(0.3), Some(307));

        // For value = 0.1 + 0.1 + 0.1 (which might be ~0.30000000000000004)
        // Should still map to the same interval due to small precision differences
        let value = 0.1 + 0.1 + 0.1;
        assert_eq!(dimension.get_interval(value), Some(307));
    }
}
