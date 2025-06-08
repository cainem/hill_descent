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
            .map(|limit_range| Dimension::new(limit_range, 1))
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
    fn given_valid_limits_when_new_dimensions_then_dimensions_are_created_correctly() {
        let limits = vec![0.0..=10.0, 5.0..=15.0];
        let dimensions_obj = Dimensions::new(limits);

        assert_eq!(dimensions_obj._dimensions.len(), 2);
        assert_eq!(*dimensions_obj._dimensions[0].range(), 0.0..=10.0);
        assert_eq!(dimensions_obj._dimensions[0].number_of_divisions(), 1);
        assert_eq!(*dimensions_obj._dimensions[1].range(), 5.0..=15.0);
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
        let limits = vec![0.0..=10.0, 10.0..=0.0]; // Second one is invalid
        Dimensions::new(limits);
    }

    #[test]
    fn given_single_valid_limit_when_new_dimensions_then_one_dimension_is_created() {
        let limits = vec![0.0..=1.0];
        let dimensions_obj = Dimensions::new(limits);

        assert_eq!(dimensions_obj._dimensions.len(), 1);
        assert_eq!(*dimensions_obj._dimensions[0].range(), 0.0..=1.0);
        assert_eq!(dimensions_obj._dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj._last_division_index, 0);
    }
}
