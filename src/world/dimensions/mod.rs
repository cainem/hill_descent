pub mod calculate_dimensions_key;
pub mod dimension;
pub mod new;

pub use calculate_dimensions_key::{CalculateDimensionsKeyResult, calculate_dimensions_key};
pub use dimension::Dimension;

#[derive(Debug, Clone)]
// Holds the spatial dimensions (axes) of the world along with bookkeeping data.
pub struct Dimensions {
    dimensions: Vec<Dimension>,
}

impl Dimensions {
    pub fn get_dimensions(&self) -> &[Dimension] {
        &self.dimensions
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
        let product: f64 = self.dimensions.iter().map(|d| d.num_intervals()).product();
        product as usize
    }

    /// Expands the bounds of a specified dimension.
    pub fn expand_bounds(&mut self, dim_idx: usize) {
        if let Some(dimension) = self.dimensions.get_mut(dim_idx) {
            dimension.expand_bounds();
        }
    }

    /// Divides the dimension at `dim_idx` by doubling the number of intervals.
    /// Increments the number_of_doublings by 1, which doubles the total intervals (2^doublings).
    /// Panics if `dim_idx` is out of bounds or if there are no defined dimensions.
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

        let dim = &mut self.dimensions[dim_idx];
        let current_doublings = dim.number_of_doublings();
        dim.set_number_of_doublings(current_doublings + 1);
    }
}

#[cfg(test)]
impl Dimensions {
    /// Test-only constructor to create a `Dimensions` object with a specific set of `Dimension`s.
    pub fn new_for_test(dimensions: Vec<Dimension>) -> Self {
        Self { dimensions }
    }
}

#[cfg(test)]
mod tests_divide_next_dimension {
    use super::*;
    use crate::world::dimensions::dimension::Dimension;

    #[test]
    fn given_zero_doublings_when_divide_next_dimension_then_becomes_one() {
        let mut dims = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=1.0, 0),
            Dimension::new(0.0..=1.0, 0),
        ]);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 0);
        dims.divide_next_dimension(1);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 1);
    }

    #[test]
    fn given_non_zero_doublings_when_divide_next_dimension_then_increments() {
        let mut dims = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=1.0, 1),
            Dimension::new(0.0..=1.0, 2),
        ]);

        // Test incrementing from 1 to 2
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 1);
        dims.divide_next_dimension(0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 2);

        // Test incrementing from 2 to 3
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 2);
        dims.divide_next_dimension(1);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 3);
    }

    #[test]
    fn given_sequence_when_divide_next_dimension_then_doublings_increment_by_one() {
        let mut dims = Dimensions::new_for_test(vec![Dimension::new(0.0..=1.0, 0)]);

        // Test the sequence: 0 -> 1 -> 2 -> 3 -> 4 (doublings increment by 1)
        // Actual intervals: 1 -> 2 -> 4 -> 8 -> 16 (2^doublings)
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 0);
        assert_eq!(dims.get_dimension(0).num_intervals(), 1.0);

        dims.divide_next_dimension(0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 1);
        assert_eq!(dims.get_dimension(0).num_intervals(), 2.0);

        dims.divide_next_dimension(0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 2);
        assert_eq!(dims.get_dimension(0).num_intervals(), 4.0);

        dims.divide_next_dimension(0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 3);
        assert_eq!(dims.get_dimension(0).num_intervals(), 8.0);

        dims.divide_next_dimension(0);
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 4);
        assert_eq!(dims.get_dimension(0).num_intervals(), 16.0);
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
