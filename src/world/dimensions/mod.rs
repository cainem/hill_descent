use crate::world::dimensions::dimension::Dimension;

pub mod calculate_dimensions_key;
pub mod dimension;
pub mod double_regions;
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
