use std::ops::{Range, RangeInclusive};

#[derive(Debug, Clone)]
pub struct Dimension {
    range: RangeInclusive<f64>,
    number_of_divisions: usize,
}

#[derive(Debug, PartialEq)]
pub enum IntervalType {
    Standard(Range<f64>),
    EndOfRange(RangeInclusive<f64>),
}

impl Dimension {
    pub fn new(min: f64, max: f64, number_of_divisions: usize) -> Self {
        assert!(
            number_of_divisions > 0,
            "Dimension divisions must be greater than 0"
        );
        assert!(
            max >= min,
            "Dimension max must be greater than or equal to min"
        );
        Self {
            range: min..=max,
            number_of_divisions,
        }
    }

    pub fn get_intervals(&self) -> impl Iterator<Item = IntervalType> + '_ {
        assert!(
            self.number_of_divisions > 0,
            "Cannot get intervals with zero divisions. Divisions must be > 0."
        );

        let min_val = *self.range.start();
        let max_val = *self.range.end();

        // Handle the case where min_val == max_val to avoid division by zero if number_of_divisions > 0
        // or to correctly create intervals of zero length.
        let step = if min_val == max_val {
            0.0
        } else {
            (max_val - min_val) / self.number_of_divisions as f64
        };

        (0..self.number_of_divisions).map(move |i| {
            let start = min_val + i as f64 * step;

            if i == self.number_of_divisions - 1 {
                // Ensure the last interval precisely ends at max_val and is inclusive.
                IntervalType::EndOfRange(start..=max_val)
            } else {
                let calculated_end = min_val + (i as f64 + 1.0) * step;
                IntervalType::Standard(start..calculated_end)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*; // This brings Dimension and IntervalType into scope

    const EPSILON: f64 = 1e-9; // Tolerance for floating-point comparisons

    #[test]
    #[should_panic(expected = "Dimension divisions must be greater than 0")]
    fn given_zero_divisions_when_new_dimension_then_panics() {
        Dimension::new(0.0, 5.0, 0);
    }

    #[test]
    #[should_panic(expected = "Dimension max must be greater than or equal to min")]
    fn given_max_less_than_min_when_new_dimension_then_panics() {
        Dimension::new(5.0, 0.0, 1);
    }

    #[test]
    fn given_dimension_when_get_intervals_with_multiple_divisions_then_returns_correct_intervals() {
        let dimension = Dimension::new(0.0, 3.0, 3);
        let intervals: Vec<IntervalType> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 3);
        assert_eq!(intervals[0], IntervalType::Standard(0.0..1.0));
        assert_eq!(intervals[1], IntervalType::Standard(1.0..2.0));
        assert_eq!(intervals[2], IntervalType::EndOfRange(2.0..=3.0));
    }

    #[test]
    fn given_dimension_when_get_intervals_with_one_division_then_returns_single_interval() {
        let dimension = Dimension::new(0.0, 5.0, 1);
        let intervals: Vec<IntervalType> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0], IntervalType::EndOfRange(0.0..=5.0));
    }

    #[test]
    fn given_dimension_with_negative_min_when_get_intervals_then_returns_correct_intervals() {
        let dimension = Dimension::new(-2.0, 2.0, 4);
        let intervals: Vec<IntervalType> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 4);
        assert_eq!(intervals[0], IntervalType::Standard(-2.0..-1.0));
        assert_eq!(intervals[1], IntervalType::Standard(-1.0..0.0));
        assert_eq!(intervals[2], IntervalType::Standard(0.0..1.0));
        assert_eq!(intervals[3], IntervalType::EndOfRange(1.0..=2.0));
    }

    #[test]
    fn given_dimension_with_non_integer_step_when_get_intervals_then_returns_correct_intervals() {
        let dimension = Dimension::new(0.0, 1.0, 2);
        let intervals: Vec<IntervalType> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 2);
        assert_eq!(intervals[0], IntervalType::Standard(0.0..0.5));
        assert_eq!(intervals[1], IntervalType::EndOfRange(0.5..=1.0));
    }

    #[test]
    fn given_dimension_where_min_equals_max_when_get_intervals_then_returns_intervals_of_zero_length()
     {
        let dimension = Dimension::new(5.0, 5.0, 3);
        let intervals: Vec<IntervalType> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 3);
        // When min == max, step is 0. All intervals start at min.
        // The standard intervals will be min..min.
        // The last interval will be min..=max (which is min..=min).
        assert_eq!(intervals[0], IntervalType::Standard(5.0..5.0));
        assert_eq!(intervals[1], IntervalType::Standard(5.0..5.0));
        assert_eq!(intervals[2], IntervalType::EndOfRange(5.0..=5.0));
    }

    #[test]
    fn given_dimension_with_fp_step_when_get_intervals_then_boundaries_are_consistent_and_last_interval_ends_at_max()
     {
        let dimension = Dimension::new(0.0, 0.3, 3);
        let intervals: Vec<IntervalType> = dimension.get_intervals().collect();
        assert_eq!(intervals.len(), 3);

        let expected_boundaries = [0.0, 0.1, 0.2, 0.3];

        match intervals[0] {
            IntervalType::Standard(ref r) => {
                assert!(
                    (r.start - expected_boundaries[0]).abs() < EPSILON,
                    "Interval 0 start mismatch. Expected {}, got {}",
                    expected_boundaries[0],
                    r.start
                );
                assert!(
                    (r.end - expected_boundaries[1]).abs() < EPSILON,
                    "Interval 0 end mismatch. Expected {}, got {}",
                    expected_boundaries[1],
                    r.end
                );
            }
            _ => panic!("Expected Standard interval for interval 0"),
        }

        match intervals[1] {
            IntervalType::Standard(ref r) => {
                assert!(
                    (r.start - expected_boundaries[1]).abs() < EPSILON,
                    "Interval 1 start mismatch. Expected {}, got {}",
                    expected_boundaries[1],
                    r.start
                );
                assert!(
                    (r.end - expected_boundaries[2]).abs() < EPSILON,
                    "Interval 1 end mismatch. Expected {}, got {}",
                    expected_boundaries[2],
                    r.end
                );
            }
            _ => panic!("Expected Standard interval for interval 1"),
        }

        match intervals[2] {
            IntervalType::EndOfRange(ref r) => {
                assert!(
                    (*r.start() - expected_boundaries[2]).abs() < EPSILON,
                    "Interval 2 start mismatch. Expected {}, got {}",
                    expected_boundaries[2],
                    *r.start()
                );
                // The last interval's end is explicitly set to dimension's max_val.
                assert_eq!(
                    *r.end(),
                    *dimension.range.end(),
                    "Interval 2 end should be exactly dimension max_val"
                );
                assert!(
                    (*r.end() - expected_boundaries[3]).abs() < EPSILON,
                    "Interval 2 end mismatch with expected boundary. Expected {}, got {}",
                    expected_boundaries[3],
                    *r.end()
                );
            }
            _ => panic!("Expected EndOfRange interval for interval 2"),
        }
    }
}
