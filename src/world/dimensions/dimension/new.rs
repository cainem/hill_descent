use std::ops::RangeInclusive;

use crate::world::dimensions::dimension::Dimension;

impl Dimension {
    pub fn new(range_bounds: RangeInclusive<f64>, number_of_divisions: usize) -> Self {
        assert!(
            number_of_divisions > 0,
            "Dimension divisions must be greater than 0"
        );
        assert!(
            *range_bounds.end() >= *range_bounds.start(),
            "Dimension max must be greater than or equal to min"
        );
        Self {
            range: range_bounds,
            number_of_divisions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // This brings Dimension and IntervalType into scope

    #[test]
    #[should_panic(expected = "Dimension divisions must be greater than 0")]
    fn given_zero_divisions_when_new_dimension_then_panics() {
        Dimension::new(0.0..=5.0, 0);
    }

    #[test]
    #[should_panic(expected = "Dimension max must be greater than or equal to min")]
    fn given_max_less_than_min_when_new_dimension_then_panics() {
        Dimension::new(5.0..=0.0, 1);
    }
}
