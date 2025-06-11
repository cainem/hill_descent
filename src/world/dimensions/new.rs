use std::ops::RangeInclusive;

use crate::parameters::global_constants::GlobalConstants;
use crate::world::dimensions::{Dimensions, dimension::Dimension};

impl Dimensions {
    pub fn new(
        parameter_bounds: &[RangeInclusive<f64>],
        global_constants: &GlobalConstants,
    ) -> Self {
        assert!(
            global_constants.max_regions() > 0,
            "max_regions must be greater than 0."
        );

        let mut created_dimensions: Vec<Dimension> = parameter_bounds
            .iter()
            .map(|bounds| Dimension::new(bounds.clone(), 0)) // Initial divisions set to 0
            .collect();

        let num_problem_dimensions = created_dimensions.len();

        if num_problem_dimensions == 0 {
            return Self {
                dimensions: created_dimensions,
                last_division_index: 0, // Consistent with old tests for empty limits
            };
        }

        // Start with a primed index, so the first dimension considered for division is index 0.
        // This will be the final _last_division_index if no divisions occur.
        let mut effective_last_division_index = num_problem_dimensions - 1;
        let mut current_potential_regions: usize = 1; // Starts at 2^0 = 1 region

        // Loop to add divisions until max_regions is met or exceeded
        loop {
            let next_index_to_try = (effective_last_division_index + 1) % num_problem_dimensions;

            // Calculate potential regions if this dimension is divided.
            // Each division in any dimension doubles the total number of potential regions.
            match current_potential_regions.checked_mul(2) {
                Some(potential_after_division)
                    if potential_after_division <= global_constants.max_regions() =>
                {
                    // It's possible to divide this dimension further
                    let current_divisions =
                        created_dimensions[next_index_to_try].number_of_divisions();
                    created_dimensions[next_index_to_try]
                        .set_number_of_divisions(current_divisions + 1);
                    current_potential_regions = potential_after_division;
                    effective_last_division_index = next_index_to_try; // This dimension was the last one successfully divided
                }
                _ => {
                    // Cannot divide further (either multiplication overflowed or exceeded max_regions)
                    break;
                }
            }
            // If current_potential_regions == global_constants.max_regions(), the next iteration
            // will attempt to multiply by 2. If this doesn't overflow, it will exceed max_regions,
            // causing the loop to break. This ensures we don't exceed max_regions.
        }

        Self {
            dimensions: created_dimensions,
            last_division_index: effective_last_division_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;

    #[test]
    fn given_empty_parameter_bounds_when_new_then_no_dimensions_are_created() {
        let bounds: Vec<RangeInclusive<f64>> = vec![];
        let gc = GlobalConstants::new(100, 100); // population_size, max_regions
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert!(dimensions_obj.dimensions.is_empty());
        assert_eq!(dimensions_obj.last_division_index, 0);
    }

    #[test]
    #[should_panic(
        expected = "Dimension max must be greater than or equal to min. Start: 10, End: 0"
    )]
    fn given_bounds_with_invalid_range_when_new_then_dimension_new_panics() {
        let bounds = vec![0.0..=10.0, 10.0..=0.0]; // Second range is invalid
        let gc = GlobalConstants::new(100, 100);
        Dimensions::new(&bounds, &gc); // This will call Dimension::new, which should panic
    }

    #[test]
    fn given_single_bound_and_max_regions_one_when_new_then_no_divisions_occur() {
        let bounds = vec![0.0..=10.0];
        let gc = GlobalConstants::new(100, 1); // max_regions = 1
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert_eq!(dimensions_obj.dimensions.len(), 1);
        assert_eq!(*dimensions_obj.dimensions[0].range(), 0.0..=10.0);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 0);
        assert_eq!(dimensions_obj.last_division_index, 0); // num_dims - 1 = 1 - 1 = 0
    }

    #[test]
    fn given_single_bound_and_max_regions_allows_some_divisions_then_divisions_occur() {
        let bounds = vec![0.0..=10.0];
        // max_regions = 8 allows 2^3 regions, so 3 divisions for a single dimension.
        // 1 region (0 div) -> 2 regions (1 div) -> 4 regions (2 div) -> 8 regions (3 div)
        // Next would be 16 regions, which exceeds 8.
        let gc = GlobalConstants::new(100, 8);
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert_eq!(dimensions_obj.dimensions.len(), 1);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 3);
        assert_eq!(dimensions_obj.last_division_index, 0);
    }

    #[test]
    fn given_single_bound_and_max_regions_allows_fewer_divisions_than_max_possible() {
        let bounds = vec![0.0..=10.0];
        // max_regions = 7. Allows 2 divisions (4 regions). Next is 8 regions > 7.
        let gc = GlobalConstants::new(100, 7);
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert_eq!(dimensions_obj.dimensions.len(), 1);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 2);
        assert_eq!(dimensions_obj.last_division_index, 0);
    }

    #[test]
    fn given_two_bounds_and_max_regions_one_then_no_divisions_occur() {
        let bounds = vec![0.0..=10.0, 0.0..=5.0];
        let gc = GlobalConstants::new(100, 1);
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert_eq!(dimensions_obj.dimensions.len(), 2);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 0);
        assert_eq!(dimensions_obj.dimensions[1].number_of_divisions(), 0);
        assert_eq!(dimensions_obj.last_division_index, 1); // num_dims - 1 = 2 - 1 = 1
    }

    #[test]
    fn given_two_bounds_and_max_regions_two_then_one_division_on_first_dim() {
        let bounds = vec![0.0..=10.0, 0.0..=5.0];
        let gc = GlobalConstants::new(100, 2); // Allows 1 total division (2^1 regions)
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert_eq!(dimensions_obj.dimensions.len(), 2);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[1].number_of_divisions(), 0);
        assert_eq!(dimensions_obj.last_division_index, 0); // Dimension 0 was divided
    }

    #[test]
    fn given_two_bounds_and_max_regions_three_then_one_division_on_first_dim() {
        let bounds = vec![0.0..=10.0, 0.0..=5.0];
        let gc = GlobalConstants::new(100, 3); // Still allows only 1 total division (2^1=2 regions, next is 2^2=4 regions)
        let dimensions_obj = Dimensions::new(&bounds, &gc);

        assert_eq!(dimensions_obj.dimensions.len(), 2);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[1].number_of_divisions(), 0);
        assert_eq!(dimensions_obj.last_division_index, 0);
    }

    #[test]
    fn given_two_bounds_and_max_regions_four_then_one_division_on_each_dim() {
        let bounds = vec![0.0..=10.0, 0.0..=5.0];
        let gc = GlobalConstants::new(100, 4); // Allows 2 total divisions (2^2=4 regions)
        let dimensions_obj = Dimensions::new(&bounds, &gc);
        // Iteration 1: Divides dim 0. regions = 2. last_idx = 0. divs = [1,0]
        // Iteration 2: Divides dim 1. regions = 4. last_idx = 1. divs = [1,1]
        // Iteration 3: Tries dim 0. potential regions = 8. 8 > 4. Break.
        assert_eq!(dimensions_obj.dimensions.len(), 2);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[1].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.last_division_index, 1); // Dimension 1 was divided last
    }

    #[test]
    fn given_three_bounds_and_max_regions_allows_round_robin_divisions() {
        let bounds = vec![0.0..=1.0, 2.0..=3.0, 4.0..=5.0];
        // max_regions = 8 allows 3 total divisions (2^3 regions)
        let gc = GlobalConstants::new(100, 8);
        let dimensions_obj = Dimensions::new(&bounds, &gc);
        // Initial: divs=[0,0,0], last_idx_prime=2, regions=1
        // Iteration 1 (dim 0): divs=[1,0,0], last_idx=0, regions=2
        // Iteration 2 (dim 1): divs=[1,1,0], last_idx=1, regions=4
        // Iteration 3 (dim 2): divs=[1,1,1], last_idx=2, regions=8
        // Iteration 4 (dim 0): Tries dim 0. potential regions = 16. 16 > 8. Break.
        assert_eq!(dimensions_obj.dimensions.len(), 3);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[1].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[2].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.last_division_index, 2);
    }

    #[test]
    fn given_three_bounds_and_max_regions_stops_before_full_round_robin() {
        let bounds = vec![0.0..=1.0, 2.0..=3.0, 4.0..=5.0];
        // max_regions = 7 allows 2 total divisions (2^2=4 regions, next is 2^3=8)
        let gc = GlobalConstants::new(100, 7);
        let dimensions_obj = Dimensions::new(&bounds, &gc);
        // Initial: divs=[0,0,0], last_idx_prime=2, regions=1
        // Iteration 1 (dim 0): divs=[1,0,0], last_idx=0, regions=2
        // Iteration 2 (dim 1): divs=[1,1,0], last_idx=1, regions=4
        // Iteration 3 (dim 2): Tries dim 2. potential regions = 8. 8 > 7. Break.
        assert_eq!(dimensions_obj.dimensions.len(), 3);
        assert_eq!(dimensions_obj.dimensions[0].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[1].number_of_divisions(), 1);
        assert_eq!(dimensions_obj.dimensions[2].number_of_divisions(), 0);
        assert_eq!(dimensions_obj.last_division_index, 1);
    }
}
