use crate::world::dimensions::{Dimensions, dimension::Dimension};
use std::ops::RangeInclusive;

impl Dimensions {
    pub fn new(parameter_bounds: &[RangeInclusive<f64>]) -> Self {
        let created_dimensions: Vec<Dimension> = parameter_bounds
            .iter()
            .map(|bounds| Dimension::new(bounds.clone(), 0)) // Initial divisions set to 0
            .collect();

        Self {
            dimensions: created_dimensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::RangeInclusive;

    #[test]
    fn given_empty_parameter_bounds_when_new_then_no_dimensions_are_created() {
        let bounds: Vec<RangeInclusive<f64>> = vec![];
        let dimensions_obj = Dimensions::new(&bounds);

        assert!(dimensions_obj.dimensions.is_empty());
    }

    #[test]
    fn given_single_bound_when_new_then_one_dimension_created_with_zero_doublings() {
        let bounds = vec![0.0..=10.0];
        let dimensions_obj = Dimensions::new(&bounds);

        assert_eq!(dimensions_obj.dimensions.len(), 1);
        assert_eq!(*dimensions_obj.dimensions[0].range(), 0.0..=10.0);
        assert_eq!(dimensions_obj.dimensions[0].number_of_doublings(), 0);
        assert_eq!(dimensions_obj.dimensions[0].num_intervals(), 1.0);
    }

    #[test]
    fn given_multiple_bounds_when_new_then_all_dimensions_created_with_zero_doublings() {
        let bounds = vec![0.0..=10.0, -5.0..=5.0, 100.0..=200.0];
        let dimensions_obj = Dimensions::new(&bounds);

        assert_eq!(dimensions_obj.dimensions.len(), 3);

        // Check first dimension
        assert_eq!(*dimensions_obj.dimensions[0].range().start(), 0.0_f64);
        assert_eq!(*dimensions_obj.dimensions[0].range().end(), 10.0_f64);
        assert_eq!(dimensions_obj.dimensions[0].number_of_doublings(), 0);
        assert_eq!(dimensions_obj.dimensions[0].num_intervals(), 1.0);

        // Check second dimension
        assert_eq!(*dimensions_obj.dimensions[1].range().start(), -5.0_f64);
        assert_eq!(*dimensions_obj.dimensions[1].range().end(), 5.0_f64);
        assert_eq!(dimensions_obj.dimensions[1].number_of_doublings(), 0);
        assert_eq!(dimensions_obj.dimensions[1].num_intervals(), 1.0);

        // Check third dimension
        assert_eq!(*dimensions_obj.dimensions[2].range().start(), 100.0_f64);
        assert_eq!(*dimensions_obj.dimensions[2].range().end(), 200.0_f64);
        assert_eq!(dimensions_obj.dimensions[2].number_of_doublings(), 0);
        assert_eq!(dimensions_obj.dimensions[2].num_intervals(), 1.0);
    }

    #[test]
    #[should_panic(
        expected = "Dimension max must be greater than or equal to min. Start: 10, End: 0"
    )]
    fn given_bounds_with_invalid_range_when_new_then_dimension_new_panics() {
        let bounds = vec![0.0..=5.0, 10.0..=0.0]; // Second range is invalid
        Dimensions::new(&bounds); // This will call Dimension::new, which should panic
    }

    #[test]
    fn given_bounds_with_equal_start_end_when_new_then_dimension_created() {
        let bounds = vec![5.0..=5.0]; // Valid range with equal start and end
        let dimensions_obj = Dimensions::new(&bounds);

        assert_eq!(dimensions_obj.dimensions.len(), 1);
        assert_eq!(*dimensions_obj.dimensions[0].range(), 5.0..=5.0);
        assert_eq!(dimensions_obj.dimensions[0].number_of_doublings(), 0);
        assert_eq!(dimensions_obj.dimensions[0].num_intervals(), 1.0);
    }

    #[test]
    fn given_bounds_with_negative_values_when_new_then_dimensions_created_correctly() {
        let bounds = vec![-10.0..=-1.0, -100.0..=100.0];
        let dimensions_obj = Dimensions::new(&bounds);

        assert_eq!(dimensions_obj.dimensions.len(), 2);
        assert_eq!(*dimensions_obj.dimensions[0].range(), -10.0..=-1.0);
        assert_eq!(*dimensions_obj.dimensions[1].range(), -100.0..=100.0);
        assert_eq!(dimensions_obj.dimensions[0].number_of_doublings(), 0);
        assert_eq!(dimensions_obj.dimensions[1].number_of_doublings(), 0);
    }
}
