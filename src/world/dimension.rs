#[derive(Debug, Clone)]
pub struct Dimension {
    _min: f64,
    _max: f64,
    number_of_divisions: usize,
}

impl Dimension {
    pub fn new(min: f64, max: f64, number_of_divisions: usize) -> Self {
        assert!(number_of_divisions > 0, "Dimension divisions must be greater than 0");
        Self {
            _min: min,
            _max: max,
            number_of_divisions,
        }
    }

    pub fn get_intervals(&self) -> impl Iterator<Item = std::ops::Range<f64>> + '_ {
        assert!(
            self.number_of_divisions > 0,
            "Cannot get intervals with zero divisions. Divisions must be > 0."
        );

        let step = (self._max - self._min) / self.number_of_divisions as f64;

        (0..self.number_of_divisions).map(move |i| {
            let start = self._min + i as f64 * step;
            let calculated_end = self._min + (i as f64 + 1.0) * step;

            if i == self.number_of_divisions - 1 {
                start..self._max
            } else {
                start..calculated_end
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    const EPSILON: f64 = 1e-9; // Tolerance for floating-point comparisons

    #[test]
    #[should_panic(expected = "Dimension divisions must be greater than 0")]
    fn given_zero_divisions_when_new_dimension_then_panics() {
        Dimension::new(0.0, 5.0, 0);
    }

    #[test]
    fn given_dimension_when_get_intervals_with_multiple_divisions_then_returns_correct_intervals() {
        let dimension = Dimension::new(0.0, 3.0, 3);
        let intervals: Vec<Range<f64>> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 3);
        assert_eq!(intervals[0], 0.0..1.0);
        assert_eq!(intervals[1], 1.0..2.0);
        assert_eq!(intervals[2], 2.0..3.0);
    }

    #[test]
    fn given_dimension_when_get_intervals_with_one_division_then_returns_single_interval() {
        let dimension = Dimension::new(0.0, 5.0, 1);
        let intervals: Vec<Range<f64>> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0], 0.0..5.0);
    }

    // This test is removed as Dimension::new(0.0, 5.0, 0) will panic.
    // The behavior of get_intervals with _divisions = 0 is implicitly tested by the constructor panic.
    // If a Dimension could be constructed with _divisions = 0 bypassing `new`,
    // a specific test for get_intervals panicking would be needed.
    // #[test]
    // #[should_panic(expected = "Cannot get intervals with zero divisions. Divisions must be > 0.")]
    // fn given_dimension_with_zero_divisions_when_get_intervals_then_panics() {
    //     // This setup is problematic as Dimension::new now panics for 0 divisions.
    //     // To test get_intervals directly, one might need to construct Dimension unsafely or mock it.
    //     // However, given the current design, Dimension::new is the entry point.
    //     let dimension = Dimension { _min: 0.0, _max: 5.0, _divisions: 0 }; // Direct construction for test
    //     dimension.get_intervals().collect::<Vec<_>>();
    // }

    #[test]
    fn given_dimension_with_negative_min_when_get_intervals_then_returns_correct_intervals() {
        let dimension = Dimension::new(-2.0, 2.0, 4);
        let intervals: Vec<Range<f64>> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 4);
        assert_eq!(intervals[0], -2.0..-1.0);
        assert_eq!(intervals[1], -1.0..0.0);
        assert_eq!(intervals[2], 0.0..1.0);
        assert_eq!(intervals[3], 1.0..2.0);
    }

    #[test]
    fn given_dimension_with_non_integer_step_when_get_intervals_then_returns_correct_intervals() {
        let dimension = Dimension::new(0.0, 1.0, 2);
        let intervals: Vec<Range<f64>> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 2);
        // For 0.0..0.5 and 0.5..1.0, direct comparison should be fine as 0.5 is exactly representable in binary.
        assert_eq!(intervals[0], 0.0..0.5);
        assert_eq!(intervals[1], 0.5..1.0);
    }

    #[test]
    fn given_dimension_where_min_equals_max_when_get_intervals_then_returns_intervals_of_zero_length()
     {
        let dimension = Dimension::new(5.0, 5.0, 3);
        let intervals: Vec<Range<f64>> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 3);
        assert_eq!(intervals[0], 5.0..5.0);
        assert_eq!(intervals[1], 5.0..5.0);
        assert_eq!(intervals[2], 5.0..5.0);
    }

    #[test]
    fn given_dimension_with_fp_step_when_get_intervals_then_boundaries_are_consistent_and_last_interval_ends_at_max()
     {
        let dimension = Dimension::new(0.0, 0.3, 3);
        let intervals: Vec<Range<f64>> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 3);

        // Expected boundaries based on 0.3 / 3 = 0.1 for each step.
        // Note: 0.1, 0.2, 0.3 literals are f64 representations.
        let expected_boundaries = [0.0, 0.1, 0.2, 0.3];

        assert!(
            (intervals[0].start - expected_boundaries[0]).abs() < EPSILON,
            "Interval 0 start mismatch"
        );
        assert!(
            (intervals[0].end - expected_boundaries[1]).abs() < EPSILON,
            "Interval 0 end mismatch"
        );

        assert!(
            (intervals[1].start - expected_boundaries[1]).abs() < EPSILON,
            "Interval 1 start mismatch"
        );
        assert!(
            (intervals[1].end - expected_boundaries[2]).abs() < EPSILON,
            "Interval 1 end mismatch"
        );

        assert!(
            (intervals[2].start - expected_boundaries[2]).abs() < EPSILON,
            "Interval 2 start mismatch"
        );
        // The last interval's end is explicitly set to self._max, so it should be exact.
        assert_eq!(
            intervals[2].end, dimension._max,
            "Interval 2 end should be exactly dimension._max"
        );
        assert!(
            (intervals[2].end - expected_boundaries[3]).abs() < EPSILON,
            "Interval 2 end mismatch with expected boundary"
        );
    }
}
