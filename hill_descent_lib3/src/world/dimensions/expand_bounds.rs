//! Bounds expansion for dimensions.

use super::Dimensions;

impl Dimensions {
    /// Expands the bounds of a specified dimension and increments version.
    ///
    /// # Arguments
    ///
    /// * `dim_idx` - Index of the dimension to expand
    ///
    /// # Side Effects
    ///
    /// - Expands the bounds of the specified dimension by 50% on each side
    /// - Increments the version number
    ///
    /// # Panics
    ///
    /// Panics if `dim_idx` is out of bounds.
    pub fn expand_bounds(&mut self, dim_idx: usize) {
        self.dimensions[dim_idx].expand_bounds();
        self.version += 1;
    }

    /// Expands bounds for multiple dimensions and increments version once.
    ///
    /// If `dim_indices` is empty, no changes are made and version is not incremented.
    ///
    /// # Arguments
    ///
    /// * `dim_indices` - Indices of dimensions to expand
    ///
    /// # Side Effects
    ///
    /// - Expands the bounds of all specified dimensions
    /// - Increments the version number once (if any dimensions were expanded)
    ///
    /// # Panics
    ///
    /// Panics if any index in `dim_indices` is out of bounds.
    pub fn expand_bounds_multiple(&mut self, dim_indices: &[usize]) {
        if dim_indices.is_empty() {
            return;
        }

        for &idx in dim_indices {
            self.dimensions[idx].expand_bounds();
        }
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::dimensions::Dimension;

    #[test]
    fn given_dimensions_when_expand_bounds_then_version_increments() {
        let mut dims =
            Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0), Dimension::new(-5.0..=5.0)]);

        assert_eq!(dims.version(), 0);

        dims.expand_bounds(0);
        assert_eq!(dims.version(), 1);

        dims.expand_bounds(1);
        assert_eq!(dims.version(), 2);
    }

    #[test]
    fn given_dimensions_when_expand_bounds_then_dimension_range_expands() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(10.0..=20.0)]);

        dims.expand_bounds(0);

        // Width is 10, expansion is 5 on each side
        assert_eq!(*dims.get_dimension(0).range(), 5.0..=25.0);
    }

    #[test]
    fn given_dimensions_when_expand_bounds_multiple_then_version_increments_once() {
        let mut dims = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=10.0),
            Dimension::new(-5.0..=5.0),
            Dimension::new(100.0..=200.0),
        ]);

        assert_eq!(dims.version(), 0);

        dims.expand_bounds_multiple(&[0, 2]);
        assert_eq!(dims.version(), 1);
    }

    #[test]
    fn given_dimensions_when_expand_bounds_multiple_then_all_specified_dimensions_expand() {
        let mut dims = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=10.0),    // Width 10, expand 5
            Dimension::new(-5.0..=5.0),    // Width 10, expand 5
            Dimension::new(100.0..=200.0), // Width 100, expand 50
        ]);

        dims.expand_bounds_multiple(&[0, 2]);

        // First dimension expanded
        assert_eq!(*dims.get_dimension(0).range(), -5.0..=15.0);
        // Second dimension not expanded
        assert_eq!(*dims.get_dimension(1).range(), -5.0..=5.0);
        // Third dimension expanded
        assert_eq!(*dims.get_dimension(2).range(), 50.0..=250.0);
    }

    #[test]
    fn given_empty_indices_when_expand_bounds_multiple_then_version_unchanged() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0)]);

        dims.expand_bounds_multiple(&[]);

        assert_eq!(dims.version(), 0);
        assert_eq!(*dims.get_dimension(0).range(), 0.0..=10.0);
    }

    #[test]
    #[should_panic]
    fn given_out_of_bounds_index_when_expand_bounds_then_panics() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0)]);
        dims.expand_bounds(5);
    }

    #[test]
    #[should_panic]
    fn given_out_of_bounds_index_when_expand_bounds_multiple_then_panics() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0)]);
        dims.expand_bounds_multiple(&[0, 5]);
    }

    #[test]
    fn given_multiple_expansions_when_called_then_version_increments_each_time() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0)]);

        dims.expand_bounds(0);
        assert_eq!(dims.version(), 1);

        dims.expand_bounds_multiple(&[0]);
        assert_eq!(dims.version(), 2);

        dims.expand_bounds(0);
        assert_eq!(dims.version(), 3);
    }
}
