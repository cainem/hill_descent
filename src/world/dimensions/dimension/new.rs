use std::ops::RangeInclusive;

use crate::world::dimensions::dimension::Dimension;

impl Dimension {
    pub fn new(range_bounds: RangeInclusive<f64>, number_of_doublings: usize) -> Self {
        assert!(
            *range_bounds.end() >= *range_bounds.start(),
            "Dimension max must be greater than or equal to min. Start: {}, End: {}",
            range_bounds.start(),
            range_bounds.end()
        );
        Self {
            range: range_bounds,
            number_of_doublings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // This brings Dimension and IntervalType into scope

    #[test]
    fn given_zero_divisions_when_new_dimension_then_succeeds() {
        let dim = Dimension::new(0.0..=5.0, 0);
        assert_eq!(dim.number_of_doublings(), 0);
        assert_eq!(*dim.range(), 0.0..=5.0);
    }

    #[test]
    #[should_panic(
        expected = "Dimension max must be greater than or equal to min. Start: 5, End: 0"
    )]
    fn given_max_less_than_min_when_new_dimension_then_panics() {
        Dimension::new(5.0..=0.0, 1);
    }

    #[test]
    fn given_valid_input_when_new_dimension_then_succeeds() {
        let dim = Dimension::new(1.0..=5.0, 2);
        assert_eq!(dim.number_of_doublings(), 2);
        assert_eq!(*dim.range(), 1.0..=5.0);
    }
}
