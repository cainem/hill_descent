use crate::world::dimensions::dimension::Dimension;

pub mod calculate_dimensions_key;
pub mod dimension;
pub mod new;

pub use calculate_dimensions_key::{CalculateDimensionsKeyResult, calculate_dimensions_key};

#[derive(Debug, Clone)]
pub struct Dimensions {
    dimensions: Vec<Dimension>,
    last_division_index: usize,
}

impl Dimensions {
    pub fn get_dimensions(&self) -> &Vec<Dimension> {
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

    /// Divides the next dimension to increase region granularity.
    /// Returns true if a division was made, false otherwise.
    pub fn divide_next_dimension(&mut self) -> bool {
        if self.dimensions.is_empty() {
            return false;
        }

        // Cycle through dimensions, dividing one at a time.
        self.last_division_index = (self.last_division_index + 1) % self.dimensions.len();
        let dim_to_divide = &mut self.dimensions[self.last_division_index];
        let current_divisions = dim_to_divide.number_of_divisions();
        dim_to_divide.set_number_of_divisions(current_divisions + 1);

        true
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
