pub mod adjust_limits;
pub mod calculate_dimensions_key;
pub mod dimension;
pub mod divide_dimension;
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

    /// Returns a mutable reference to a specific dimension by its index.
    pub fn get_dimension_mut(&mut self, index: usize) -> &mut Dimension {
        &mut self.dimensions[index]
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
}

#[cfg(test)]
impl Dimensions {
    /// Test-only constructor to create a `Dimensions` object with a specific set of `Dimension`s.
    pub fn new_for_test(dimensions: Vec<Dimension>) -> Self {
        Self { dimensions }
    }
}
