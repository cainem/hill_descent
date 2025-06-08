use std::ops::RangeInclusive;

use crate::world::dimensions::dimension::Dimension;

pub mod dimension;

#[derive(Debug, Clone)]
pub struct Dimensions {
    _dimensions: Vec<Dimension>,
    _last_division_index: usize,
}

impl Dimensions {
    pub fn new(limits: Vec<RangeInclusive<f64>>) -> Self {
        let created_dimensions: Vec<Dimension> = limits
            .into_iter()
            .map(|limit_range| {
                let (min_orig, max_orig) = (*limit_range.start(), *limit_range.end());
                let width_orig = max_orig - min_orig;
                let width_doubled = width_orig * 2.0;
                let midpoint = (min_orig + max_orig) / 2.0;

                let new_min = midpoint - width_doubled / 2.0;
                let new_max = midpoint + width_doubled / 2.0;
                Dimension::new(new_min..=new_max, 1)
            })
            .collect();

        Self {
            _dimensions: created_dimensions,
            _last_division_index: 0,
        }
    }

    pub fn get_dimensions(&self) -> &Vec<Dimension> {
        &self._dimensions
    }

    pub fn get_last_division_index(&self) -> usize {
        self._last_division_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_limits_when_new_dimensions_then_dimensions_are_created_with_doubled_ranges() {
        let limits = vec![0.0..=10.0, 5.0..=15.0];
        let dimensions_obj = Dimensions::new(limits);

        assert_eq!(dimensions_obj._dimensions.len(), 2);
        // Original: 0.0..=10.0, Midpoint: 5.0, Width: 10.0. Doubled Width: 20.0. New Range: 5.0 +/- 10.0 => -5.0..=15.0
        assert_eq!(*dimensions_obj._dimensions[0].range(), -5.0..=15.0);
        assert_eq!(dimensions_obj._dimensions[0].number_of_divisions(), 1);
        // Original: 5.0..=15.0, Midpoint: 10.0, Width: 10.0. Doubled Width: 20.0. New Range: 10.0 +/- 10.0 => 0.0..=20.0
        assert_eq!(*dimensions_obj._dimensions[1].range(), 0.0..=20.0);
        assert_eq!(dimensions_obj._dimensions[1].number_of_divisions(), 1);
        assert_eq!(dimensions_obj._last_division_index, 0);
    }

    #[test]
    fn given_empty_limits_when_new_dimensions_then_no_dimensions_are_created() {
        let limits: Vec<RangeInclusive<f64>> = vec![];
        let dimensions_obj = Dimensions::new(limits);

        assert!(dimensions_obj._dimensions.is_empty());
        assert_eq!(dimensions_obj._last_division_index, 0);
    }

    #[test]
    #[should_panic(expected = "Dimension max must be greater than or equal to min")]
    fn given_limits_with_invalid_range_when_new_dimensions_then_dimension_new_panics() {
        // Original: 10.0..=0.0, Midpoint: 5.0, Width: -10.0. Doubled Width: -20.0. New Range: 5.0 +/- (-10.0) => 15.0..=-5.0
        // This will be passed to Dimension::new and should panic.
        let limits = vec![0.0..=10.0, 10.0..=0.0];
        Dimensions::new(limits);
    }

    #[test]
    fn given_single_valid_limit_when_new_dimensions_then_one_dimension_is_created_with_doubled_range()
     {
        let limits = vec![0.0..=1.0];
        let dimensions_obj = Dimensions::new(limits);

        assert_eq!(dimensions_obj._dimensions.len(), 1);
        // Original: 0.0..=1.0, Midpoint: 0.5, Width: 1.0. Doubled Width: 2.0. New Range: 0.5 +/- 1.0 => -0.5..=1.5
        assert_eq!(*dimensions_obj._dimensions[0].range(), -0.5..=1.5);
        assert_eq!(dimensions_obj._dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj._last_division_index, 0);
    }

    #[test]
    fn given_zero_width_limit_when_new_dimensions_then_range_is_unchanged() {
        let limits = vec![5.0..=5.0];
        let dimensions_obj = Dimensions::new(limits);

        assert_eq!(dimensions_obj._dimensions.len(), 1);
        // Original: 5.0..=5.0, Midpoint: 5.0, Width: 0.0. Doubled Width: 0.0. New Range: 5.0 +/- 0.0 => 5.0..=5.0
        assert_eq!(*dimensions_obj._dimensions[0].range(), 5.0..=5.0);
        assert_eq!(dimensions_obj._dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj._last_division_index, 0);
    }
}
