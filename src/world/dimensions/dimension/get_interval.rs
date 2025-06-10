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
        
        // Handle special case: single point range
        if start == end {
            return Some(0);
        }
        
        // Handle special case: no divisions (just one interval)
        if divisions == 0 {
            return Some(0);
        }
        
        // Calculate the size of each interval
        let interval_size = (end - start) / divisions as f64;
        
        // Handle the case when value is exactly at the end of the range
        if value == end {
            return Some(divisions);
        }
        
        // Calculate which interval the value falls into
        let interval = ((value - start) / interval_size).floor() as usize;
        
        // Safety check in case of floating-point errors
        if interval > divisions {
            Some(divisions)
        } else {
            Some(interval)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::RangeInclusive;
    
    #[test]
    fn test_get_interval_basic() {
        // Range 0..=10 with 5 divisions (6 intervals)
        // Intervals: [0,2), [2,4), [4,6), [6,8), [8,10), [10,10]
        let dimension = Dimension { range: 0.0..=10.0, number_of_divisions: 5 };
        
        assert_eq!(dimension.get_interval(0.0), Some(0));     // Start of range
        assert_eq!(dimension.get_interval(1.9), Some(0));     // First interval
        assert_eq!(dimension.get_interval(2.0), Some(1));     // Start of second interval
        assert_eq!(dimension.get_interval(5.5), Some(2));     // Middle interval
        assert_eq!(dimension.get_interval(9.9), Some(4));     // Last interval (except end point)
        assert_eq!(dimension.get_interval(10.0), Some(5));    // End of range (special case)
    }
    
    #[test]
    fn test_get_interval_out_of_bounds() {
        let dimension = Dimension { range: 0.0..=10.0, number_of_divisions: 5 };
        
        assert_eq!(dimension.get_interval(-0.1), None);    // Below range
        assert_eq!(dimension.get_interval(10.1), None);    // Above range
    }
    
    #[test]
    fn test_get_interval_zero_divisions() {
        // Range 0..=10 with 0 divisions (1 interval covering the entire range)
        let dimension = Dimension { range: 0.0..=10.0, number_of_divisions: 0 };
        
        assert_eq!(dimension.get_interval(0.0), Some(0));     // Start of range
        assert_eq!(dimension.get_interval(5.0), Some(0));     // Middle of range
        assert_eq!(dimension.get_interval(10.0), Some(0));    // End of range
    }
    
    #[test]
    fn test_get_interval_single_point_range() {
        // Single point range 5..=5 with various divisions
        let dimension1 = Dimension { range: 5.0..=5.0, number_of_divisions: 0 };
        let dimension2 = Dimension { range: 5.0..=5.0, number_of_divisions: 5 };
        
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
        let dimension = Dimension { range: 0.0..=10.0, number_of_divisions: 2 };
        
        // Test exactly at the boundaries
        assert_eq!(dimension.get_interval(0.0), Some(0));     // Start boundary
        assert_eq!(dimension.get_interval(5.0), Some(1));     // Middle boundary
        assert_eq!(dimension.get_interval(10.0), Some(2));    // End boundary
        
        // Test close to but not at boundaries
        assert_eq!(dimension.get_interval(4.999), Some(0));   // Just before middle boundary
        assert_eq!(dimension.get_interval(5.001), Some(1));   // Just after middle boundary
        assert_eq!(dimension.get_interval(9.999), Some(1));   // Just before end boundary
    }
    
    #[test]
    fn test_get_interval_negative_range() {
        // Range -10..=-5 with 5 divisions (6 intervals)
        let dimension = Dimension { range: -10.0..=-5.0, number_of_divisions: 5 };
        
        assert_eq!(dimension.get_interval(-10.0), Some(0));  // Start of range
        assert_eq!(dimension.get_interval(-7.5), Some(2));   // Middle of range
        assert_eq!(dimension.get_interval(-5.0), Some(5));   // End of range
        
        // Out of bounds
        assert_eq!(dimension.get_interval(-10.1), None);    // Below range
        assert_eq!(dimension.get_interval(-4.9), None);     // Above range
    }
    
    #[test]
    fn test_get_interval_mixed_range() {
        // Range -5..=5 with 10 divisions (11 intervals)
        let dimension = Dimension { range: -5.0..=5.0, number_of_divisions: 10 };
        
        assert_eq!(dimension.get_interval(-5.0), Some(0));   // Start of range
        assert_eq!(dimension.get_interval(0.0), Some(5));    // Middle of range
        assert_eq!(dimension.get_interval(5.0), Some(10));   // End of range
        
        // Out of bounds
        assert_eq!(dimension.get_interval(-5.1), None);     // Below range
        assert_eq!(dimension.get_interval(5.1), None);      // Above range
    }
    
    #[test]
    fn test_get_interval_floating_point_precision() {
        // Range 0..=1 with 10 divisions
        // Each interval is 0.1 wide
        let dimension = Dimension { range: 0.0..=1.0, number_of_divisions: 10 };
        
        // Test with values that might have floating point precision issues
        assert_eq!(dimension.get_interval(0.1), Some(1));
        assert_eq!(dimension.get_interval(0.2), Some(2));
        assert_eq!(dimension.get_interval(0.3), Some(3));
        
        // Edge case with potential floating point errors
        // 0.1 * 3 might not be exactly 0.3 in floating point
        let value = 0.1 + 0.1 + 0.1;
        assert_eq!(dimension.get_interval(value), Some(3));
    }
}
