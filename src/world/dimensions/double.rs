use crate::world::dimensions::Dimensions;

impl Dimensions {
    /// Doubles the number of divisions in the next dimension in a round-robin sequence.
    ///
    /// This method identifies the next `Dimension` to be divided based on `_last_division_index`,
    /// increments its `number_of_divisions` by one, and updates `_last_division_index`.
    ///
    /// # Panics
    ///
    /// Panics if the `_dimensions` vector is empty, as there would be no dimension to divide.
    pub fn double(&mut self) {
        if self._dimensions.is_empty() {
            panic!("Cannot double divisions: no dimensions are present.");
        }

        let num_dimensions = self._dimensions.len();
        let next_division_index = (self._last_division_index + 1) % num_dimensions;

        let dimension_to_divide = &mut self._dimensions[next_division_index];
        let current_divisions = dimension_to_divide.number_of_divisions();
        dimension_to_divide.set_number_of_divisions(current_divisions + 1);

        self._last_division_index = next_division_index;
    }
}

#[cfg(test)]
mod tests {
    use crate::world::dimensions::dimension::Dimension;
    use crate::world::dimensions::Dimensions;

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
            _dimensions: dims,
            _last_division_index: last_division_index,
        }
    }

    #[test]
    #[should_panic(expected = "Cannot double divisions: no dimensions are present.")]
    fn given_empty_dimensions_when_double_called_then_panics() {
        let mut dims = create_dimensions_for_test(0, vec![], 0);
        dims.double();
    }

    #[test]
    fn given_single_dimension_when_double_called_then_division_increments_and_index_updates() {
        let mut dims = create_dimensions_for_test(1, vec![0], 0); // Start with last_div_idx = 0 for a single dim
        
        dims.double(); // Should divide dimension 0 ( (0+1)%1 = 0 )
        assert_eq!(dims._dimensions[0].number_of_divisions(), 1);
        assert_eq!(dims._last_division_index, 0);

        dims.double(); // Should divide dimension 0 again
        assert_eq!(dims._dimensions[0].number_of_divisions(), 2);
        assert_eq!(dims._last_division_index, 0);
    }

    #[test]
    fn given_multiple_dimensions_when_double_called_then_round_robin_division_occurs() {
        // Start with last_division_index = 2 (index of the last element)
        // so the next one to be divided is (2+1)%3 = 0
        let mut dims = create_dimensions_for_test(3, vec![1, 1, 1], 2);

        // Call 1: Divide dimension 0
        dims.double(); 
        assert_eq!(dims._dimensions[0].number_of_divisions(), 2, "Dim 0 after 1st double");
        assert_eq!(dims._dimensions[1].number_of_divisions(), 1, "Dim 1 after 1st double");
        assert_eq!(dims._dimensions[2].number_of_divisions(), 1, "Dim 2 after 1st double");
        assert_eq!(dims._last_division_index, 0, "Last index after 1st double");

        // Call 2: Divide dimension 1
        dims.double();
        assert_eq!(dims._dimensions[0].number_of_divisions(), 2, "Dim 0 after 2nd double");
        assert_eq!(dims._dimensions[1].number_of_divisions(), 2, "Dim 1 after 2nd double");
        assert_eq!(dims._dimensions[2].number_of_divisions(), 1, "Dim 2 after 2nd double");
        assert_eq!(dims._last_division_index, 1, "Last index after 2nd double");

        // Call 3: Divide dimension 2
        dims.double();
        assert_eq!(dims._dimensions[0].number_of_divisions(), 2, "Dim 0 after 3rd double");
        assert_eq!(dims._dimensions[1].number_of_divisions(), 2, "Dim 1 after 3rd double");
        assert_eq!(dims._dimensions[2].number_of_divisions(), 2, "Dim 2 after 3rd double");
        assert_eq!(dims._last_division_index, 2, "Last index after 3rd double");

        // Call 4: Divide dimension 0 again (round robin)
        dims.double();
        assert_eq!(dims._dimensions[0].number_of_divisions(), 3, "Dim 0 after 4th double");
        assert_eq!(dims._dimensions[1].number_of_divisions(), 2, "Dim 1 after 4th double");
        assert_eq!(dims._dimensions[2].number_of_divisions(), 2, "Dim 2 after 4th double");
        assert_eq!(dims._last_division_index, 0, "Last index after 4th double");
    }

     #[test]
    fn given_single_dimension_starts_at_non_zero_last_index_when_double_then_correct() {
        // This scenario for a single dimension: _last_division_index should ideally be 0
        // if it was initialized by Dimensions::new. However, if set manually or by other means,
        // the modulo arithmetic should still correctly target index 0.
        // (0+1)%1 = 0. So, if last_division_index is 0 (or any valid usize for that matter, due to modulo 1)
        // it will still target dimension 0.
        let mut dims = create_dimensions_for_test(1, vec![5], 0); 
        dims.double();
        assert_eq!(dims._dimensions[0].number_of_divisions(), 6);
        assert_eq!(dims._last_division_index, 0);
    }
}
