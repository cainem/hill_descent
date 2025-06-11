use crate::world::dimensions::Dimensions;

impl Dimensions {
    /// Updates the number of divisions in the next dimension in a round-robin sequence.
    ///
    /// The method identifies the next `Dimension` to be divided based on `last_division_index`.
    /// It then updates its `number_of_divisions` such that the number of regions
    /// (i.e., `number_of_divisions + 1`) aims to be the next power of two.
    /// Specifically:
    /// - If the current number of regions is already a power of two, it's doubled.
    /// - Otherwise, it's increased to the next highest power of two.
    ///
    /// For example, if `number_of_divisions` is 0 (1 region), it becomes 1 (2 regions).
    /// If it's 1 (2 regions), it becomes 3 (4 regions).
    /// If it's 2 (3 regions), it becomes 3 (4 regions).
    /// The `last_division_index` is updated to point to the dimension that was just modified.
    ///
    /// # Panics
    ///
    /// Panics if the `dimensions` vector is empty, as there would be no dimension to divide.
    pub fn double_regions(&mut self) {
        if self.dimensions.is_empty() {
            panic!("Cannot double divisions: no dimensions are present.");
        }

        let num_dimensions = self.dimensions.len();
        let next_division_index = (self.last_division_index + 1) % num_dimensions;

        let dimension_to_divide = &mut self.dimensions[next_division_index];
        let current_divisions = dimension_to_divide.number_of_divisions();

        let current_regions_on_axis = current_divisions.saturating_add(1);

        let new_regions_on_axis = if current_regions_on_axis == 0 {
            // This case implies current_divisions was usize::MAX. next_power_of_two(0) is 1.
            // Results in 0 divisions, effectively cycling from MAX to 0.
            1
        } else if current_regions_on_axis.is_power_of_two() {
            // If already a power of two, double it.
            // Saturating_mul to prevent overflow, though unlikely for typical division counts.
            current_regions_on_axis.saturating_mul(2)
        } else {
            // Otherwise, find the next power of two.
            // next_power_of_two returns 0 if overflow, which means new_number_of_divisions becomes usize::MAX.
            current_regions_on_axis.next_power_of_two()
        };

        // If new_regions_on_axis is 0 (due to overflow from next_power_of_two or mul),
        // then new_number_of_divisions will wrap to usize::MAX.
        // This is an extreme edge case. For practical purposes, it should not happen.
        // If it did, it means we can't divide further along this axis in a meaningful way.
        let new_number_of_divisions = new_regions_on_axis.saturating_sub(1);

        dimension_to_divide.set_number_of_divisions(new_number_of_divisions);

        self.last_division_index = next_division_index;
    }
}

#[cfg(test)]
mod tests {
    use crate::world::dimensions::Dimensions;
    use crate::world::dimensions::dimension::Dimension;

    // Helper to create Dimensions for testing
    fn create_dimensions_for_test(
        num_dims: usize,
        initial_divisions: Vec<usize>,
        last_division_index: usize,
    ) -> Dimensions {
        assert_eq!(
            num_dims,
            initial_divisions.len(),
            "Number of dimensions must match length of initial_divisions vector"
        );
        let mut dims = Vec::new();
        for i in 0..num_dims {
            dims.push(Dimension::new(0.0..=1.0, initial_divisions[i]));
        }
        Dimensions {
            dimensions: dims,
            last_division_index,
        }
    }

    #[test]
    #[should_panic(expected = "Cannot double divisions: no dimensions are present.")]
    fn given_empty_dimensions_when_double_called_then_panics() {
        let mut dims = create_dimensions_for_test(0, vec![], 0);
        dims.double_regions();
    }

    #[test]
    fn given_single_dimension_when_double_called_then_division_updates_correctly() {
        let mut dims = create_dimensions_for_test(1, vec![0], 0); // Start with 0 divisions

        // Call 1: 0 divisions (1 region) -> 1 division (2 regions)
        dims.double_regions();
        assert_eq!(dims.dimensions[0].number_of_divisions(), 1, "0 -> 1");
        assert_eq!(dims.last_division_index, 0);

        // Call 2: 1 division (2 regions) -> 3 divisions (4 regions)
        dims.double_regions();
        assert_eq!(dims.dimensions[0].number_of_divisions(), 3, "1 -> 3");
        assert_eq!(dims.last_division_index, 0);

        // Call 3: 3 divisions (4 regions) -> 7 divisions (8 regions)
        dims.double_regions();
        assert_eq!(dims.dimensions[0].number_of_divisions(), 7, "3 -> 7");
        assert_eq!(dims.last_division_index, 0);

        // Test starting from a non-sequence number, e.g. 2 divisions (3 regions)
        let mut dims_from_2 = create_dimensions_for_test(1, vec![2], 0);
        // 2 divisions (3 regions) -> 3 divisions (4 regions)
        dims_from_2.double_regions();
        assert_eq!(dims_from_2.dimensions[0].number_of_divisions(), 3, "2 -> 3");
        assert_eq!(dims_from_2.last_division_index, 0);
    }

    #[test]
    fn given_multiple_dimensions_when_double_called_then_round_robin_division_updates_correctly() {
        // Start with last_division_index = 2 (index of the last element)
        // so the next one to be divided is (2+1)%3 = 0
        // Initial divisions: Dim0=0 (1 region), Dim1=1 (2 regions), Dim2=2 (3 regions)
        let mut dims = create_dimensions_for_test(3, vec![0, 1, 2], 2);

        // Call 1: Divide dimension 0. Currently 0 divisions (1 region).
        // 0 -> 1 division (2 regions)
        dims.double_regions();
        assert_eq!(dims.dimensions[0].number_of_divisions(), 1, "Dim 0: 0->1");
        assert_eq!(
            dims.dimensions[1].number_of_divisions(),
            1,
            "Dim 1 unchanged"
        );
        assert_eq!(
            dims.dimensions[2].number_of_divisions(),
            2,
            "Dim 2 unchanged"
        );
        assert_eq!(dims.last_division_index, 0, "Last index after 1st double");

        // Call 2: Divide dimension 1. Currently 1 division (2 regions).
        // 1 -> 3 divisions (4 regions)
        dims.double_regions();
        assert_eq!(
            dims.dimensions[0].number_of_divisions(),
            1,
            "Dim 0 unchanged"
        );
        assert_eq!(dims.dimensions[1].number_of_divisions(), 3, "Dim 1: 1->3");
        assert_eq!(
            dims.dimensions[2].number_of_divisions(),
            2,
            "Dim 2 unchanged"
        );
        assert_eq!(dims.last_division_index, 1, "Last index after 2nd double");

        // Call 3: Divide dimension 2. Currently 2 divisions (3 regions).
        // 2 -> 3 divisions (4 regions)
        dims.double_regions();
        assert_eq!(
            dims.dimensions[0].number_of_divisions(),
            1,
            "Dim 0 unchanged"
        );
        assert_eq!(
            dims.dimensions[1].number_of_divisions(),
            3,
            "Dim 1 unchanged"
        );
        assert_eq!(dims.dimensions[2].number_of_divisions(), 3, "Dim 2: 2->3");
        assert_eq!(dims.last_division_index, 2, "Last index after 3rd double");

        // Call 4: Divide dimension 0 again. Currently 1 division (2 regions).
        // 1 -> 3 divisions (4 regions)
        dims.double_regions();
        assert_eq!(dims.dimensions[0].number_of_divisions(), 3, "Dim 0: 1->3");
        assert_eq!(
            dims.dimensions[1].number_of_divisions(),
            3,
            "Dim 1 unchanged"
        );
        assert_eq!(
            dims.dimensions[2].number_of_divisions(),
            3,
            "Dim 2 unchanged"
        );
        assert_eq!(dims.last_division_index, 0, "Last index after 4th double");
    }

    #[test]
    fn given_single_dimension_starts_at_arbitrary_value_when_double_then_correct() {
        // Test with initial divisions = 5 (6 regions)
        // 5 divisions (6 regions) -> 7 divisions (8 regions)
        let mut dims = create_dimensions_for_test(1, vec![5], 0);
        dims.double_regions();
        assert_eq!(dims.dimensions[0].number_of_divisions(), 7, "5 -> 7");
        assert_eq!(dims.last_division_index, 0);

        // Test with initial divisions = 6 (7 regions)
        // 6 divisions (7 regions) -> 7 divisions (8 regions)
        let mut dims2 = create_dimensions_for_test(1, vec![6], 0);
        dims2.double_regions();
        assert_eq!(dims2.dimensions[0].number_of_divisions(), 7, "6 -> 7");
        assert_eq!(dims2.last_division_index, 0);
    }
}
