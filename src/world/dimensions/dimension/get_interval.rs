use crate::world::dimensions::dimension::Dimension;

impl Dimension {
    /// Determines which 0-indexed interval a given value falls into.
    ///
    /// If the dimension's range is divided into `number_of_divisions` divisions,
    /// there are `number_of_divisions + 1` intervals (indexed from 0 to `number_of_divisions`).
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
        let divisions = self.number_of_divisions();

        // Check if value is outside the range
        if value < start || value > end {
            return None;
        }

        // Handle special case: single point range or no divisions.
        if start == end || divisions == 0 {
            return Some(0);
        }

        // The number of intervals is `divisions + 1`.
        let num_intervals = (divisions + 1) as f64;
        let interval_size = (end - start) / num_intervals;

        // Handle case where range is tiny and interval_size is zero.
        if interval_size == 0.0 {
            // If size is 0, all values are effectively at the start,
            // except for the exact end value.
            return if value == end {
                Some(divisions)
            } else {
                Some(0)
            };
        }

        // Calculate which interval the value falls into.
        let pre_clamp_interval_float = (value - start) / interval_size;
        let mut interval = pre_clamp_interval_float.floor() as usize;

        // Clamp the interval to the max index, which is `divisions`.
        // This handles the `value == end` case correctly, as it might calculate
        // to `divisions + 1` due to floating point representation, and ensures
        // it falls into the last interval.
        if interval > divisions {
            interval = divisions;
        }

        Some(interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interval_basic_and_boundaries() {
        // Range 0..=10 with 1 division (2 intervals of size 5)
        // Intervals: [0, 5), [5, 10]
        let dimension1 = Dimension {
            range: 0.0..=10.0,
            number_of_divisions: 1,
        };
        assert_eq!(dimension1.get_interval(0.0), Some(0));
        assert_eq!(dimension1.get_interval(4.999), Some(0));
        assert_eq!(dimension1.get_interval(5.0), Some(1));
        assert_eq!(dimension1.get_interval(10.0), Some(1));

        // Range 0..=10 with 3 divisions (4 intervals of size 2.5)
        // Intervals: [0, 2.5), [2.5, 5), [5, 7.5), [7.5, 10]
        let dimension2 = Dimension {
            range: 0.0..=10.0,
            number_of_divisions: 3,
        };
        assert_eq!(dimension2.get_interval(0.0), Some(0));
        assert_eq!(dimension2.get_interval(2.4), Some(0));
        assert_eq!(dimension2.get_interval(2.5), Some(1));
        assert_eq!(dimension2.get_interval(6.0), Some(2));
        assert_eq!(dimension2.get_interval(7.5), Some(3));
        assert_eq!(dimension2.get_interval(10.0), Some(3));
    }

    #[test]
    fn test_get_interval_out_of_bounds() {
        let dimension = Dimension {
            range: 0.0..=10.0,
            number_of_divisions: 5,
        };

        assert_eq!(dimension.get_interval(-0.1), None); // Below range
        assert_eq!(dimension.get_interval(10.1), None); // Above range
    }

    #[test]
    fn test_get_interval_zero_divisions() {
        // Range 0..=10 with 0 divisions (1 interval covering the entire range)
        let dimension = Dimension {
            range: 0.0..=10.0,
            number_of_divisions: 0,
        };

        assert_eq!(dimension.get_interval(0.0), Some(0)); // Start of range
        assert_eq!(dimension.get_interval(5.0), Some(0)); // Middle of range
        assert_eq!(dimension.get_interval(10.0), Some(0)); // End of range
    }

    #[test]
    fn test_get_interval_single_point_range() {
        // Single point range 5..=5 with various divisions
        let dimension1 = Dimension {
            range: 5.0..=5.0,
            number_of_divisions: 0,
        };
        let dimension2 = Dimension {
            range: 5.0..=5.0,
            number_of_divisions: 5,
        };

        // In a single point range, any value that's in range (i.e., exactly equal to that point)
        // must be in interval 0, regardless of the number of divisions
        assert_eq!(dimension1.get_interval(5.0), Some(0));
        assert_eq!(dimension2.get_interval(5.0), Some(0));

        // Values outside the single point are out of range
        assert_eq!(dimension1.get_interval(4.9), None);
        assert_eq!(dimension1.get_interval(5.1), None);
    }

    #[test]
    fn test_get_interval_at_boundaries() {
        // Range 0..=10 with 2 divisions (3 intervals)
        // Intervals: [0,5), [5,10), [10,10]
        let dimension = Dimension {
            range: 0.0..=10.0,
            number_of_divisions: 2,
        };

        // Test exactly at the boundaries
        assert_eq!(dimension.get_interval(0.0), Some(0)); // Start boundary
        assert_eq!(dimension.get_interval(5.0), Some(1)); // Middle boundary
        assert_eq!(dimension.get_interval(10.0), Some(2)); // End boundary

        // Test close to but not at boundaries
        assert_eq!(dimension.get_interval(4.999), Some(1)); // Just before middle boundary
        assert_eq!(dimension.get_interval(5.001), Some(1)); // Just after middle boundary
        assert_eq!(dimension.get_interval(9.999), Some(2)); // Just before end boundary
    }

    #[test]
    fn test_get_interval_negative_range() {
        // Range -10..=-5 with 5 divisions (6 intervals)
        let dimension = Dimension {
            range: -10.0..=-5.0,
            number_of_divisions: 5,
        };

        assert_eq!(dimension.get_interval(-10.0), Some(0)); // Start of range
        // With interval_size = 5.0/6.0 = 0.833...
        // (-7.5 - (-10.0)) / interval_size = 2.5 / 0.833... = 3.0. floor() = 3.
        assert_eq!(dimension.get_interval(-7.5), Some(3)); // Middle of range, falls into interval 3
        assert_eq!(dimension.get_interval(-5.0), Some(5)); // End of range

        // Out of bounds
        assert_eq!(dimension.get_interval(-10.1), None); // Below range
        assert_eq!(dimension.get_interval(-4.9), None); // Above range
    }

    #[test]
    fn test_get_interval_mixed_range() {
        // Range -5..=5 with 10 divisions (11 intervals)
        let dimension = Dimension {
            range: -5.0..=5.0,
            number_of_divisions: 10,
        };

        assert_eq!(dimension.get_interval(-5.0), Some(0)); // Start of range
        assert_eq!(dimension.get_interval(0.0), Some(5)); // Middle of range
        assert_eq!(dimension.get_interval(5.0), Some(10)); // End of range

        // Out of bounds
        assert_eq!(dimension.get_interval(-5.1), None); // Below range
        assert_eq!(dimension.get_interval(5.1), None); // Above range
    }

    #[test]
    fn test_get_interval_floating_point_precision() {
        // Range 0..=1 with 10 divisions
        // Each interval is 0.1 wide
        let dimension = Dimension {
            range: 0.0..=1.0,
            number_of_divisions: 10,
        };

        // Test with values that might have floating point precision issues
        assert_eq!(dimension.get_interval(0.1), Some(1)); // (0.1 / (1/11)) = 1.1 -> floor(1.1) = 1
        assert_eq!(dimension.get_interval(0.2), Some(2)); // (0.2 / (1/11)) = 2.2 -> floor(2.2) = 2

        // For value = 0.3, (0.3 / (1.0/11.0)) = 3.3. floor(3.3) = 3.
        assert_eq!(dimension.get_interval(0.3), Some(3));

        // For value = 0.1 + 0.1 + 0.1 (which might be ~0.30000000000000004)
        // (0.30000000000000004 / (1.0/11.0)) approx 3.3000000000000007 -> floor = 3
        let value = 0.1 + 0.1 + 0.1;
        assert_eq!(dimension.get_interval(value), Some(3));
    }
}
