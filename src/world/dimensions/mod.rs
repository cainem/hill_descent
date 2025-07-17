pub mod calculate_dimensions_key;
pub mod dimension;
pub mod new;

pub use calculate_dimensions_key::{CalculateDimensionsKeyResult, calculate_dimensions_key};
pub use dimension::Dimension;

#[derive(Debug, Clone)]
// Holds the spatial dimensions (axes) of the world along with bookkeeping data.
pub struct Dimensions {
    dimensions: Vec<Dimension>,
    // TODO - This should no longer be needed
    last_division_index: usize,
}

impl Dimensions {
    pub fn get_dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }

    pub fn get_last_division_index(&self) -> usize {
        self.last_division_index
    }

    /// Returns the number of spatial dimensions.
    pub fn num_dimensions(&self) -> usize {
        self.dimensions.len()
    }

    /// Returns a reference to a specific dimension by its index.
    pub fn get_dimension(&self, index: usize) -> &Dimension {
        &self.dimensions[index]
    }

    /// Calculates the total number of possible regions.
    pub fn get_total_possible_regions(&self) -> usize {
        if self.dimensions.is_empty() {
            // If there are no dimensions, there is one region (the whole space).
            // However, the update logic now panics on 0 dimensions, so this is a safeguard.
            return 1;
        }
        // The total number of regions is the product of the number of intervals in each dimension.
        self.dimensions.iter().map(|d| d.num_intervals()).product()
    }

    /// Expands the bounds of a specified dimension.
    pub fn expand_bounds(&mut self, dim_idx: usize) {
        if let Some(dimension) = self.dimensions.get_mut(dim_idx) {
            dimension.expand_bounds();
        }
    }

    /// Divides the dimension at `dim_idx`, increasing the total potential regions by a factor
    /// of two. Panics if `dim_idx` is out of bounds or if there are no defined dimensions.
    pub fn divide_next_dimension(&mut self, dim_idx: usize) {
        assert!(
            !self.dimensions.is_empty(),
            "divide_next_dimension called on empty Dimensions set"
        );
        assert!(
            dim_idx < self.dimensions.len(),
            "dim_idx {} out of bounds: {} dimensions",
            dim_idx,
            self.dimensions.len()
        );

        self.last_division_index = dim_idx;
        let dim = &mut self.dimensions[dim_idx];
        let current_divisions = dim.number_of_divisions();
        dim.set_number_of_divisions(current_divisions + 1);
    }
}

#[cfg(test)]
impl Dimensions {
    /// Test-only constructor to create a `Dimensions` object with a specific set of `Dimension`s.
    pub fn new_for_test(dimensions: Vec<Dimension>) -> Self {
        let last_division_index = if dimensions.is_empty() {
            0
        } else {
            // A sensible default for tests that don't care about this value.
            // The logic in `double_regions` depends on this, but `update_dimensions_key` does not.
            dimensions.len() - 1
        };
        Self {
            dimensions,
            last_division_index,
        }
    }
}

#[cfg(test)]
mod tests_divide_next_dimension {
    use super::*;
    use crate::world::dimensions::dimension::Dimension;

    #[test]
    fn given_valid_index_when_divide_next_dimension_then_divisions_increment() {
        let mut dims = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=1.0, 0),
            Dimension::new(0.0..=1.0, 0),
        ]);
        assert_eq!(dims.get_dimension(1).number_of_divisions(), 0);
        dims.divide_next_dimension(1);
        assert_eq!(dims.get_last_division_index(), 1);
        assert_eq!(dims.get_dimension(1).number_of_divisions(), 1);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn given_out_of_bounds_index_when_divide_next_dimension_then_panics() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=1.0, 0)]);
        dims.divide_next_dimension(5);
    }

    #[test]
    #[should_panic(expected = "empty Dimensions set")]
    fn given_empty_dimensions_when_divide_next_dimension_then_panics() {
        let mut dims = Dimensions::new_for_test(vec![]);
        dims.divide_next_dimension(0);
    }
}
