//! Dimensions module - spatial bounds with versioning for incremental updates.
//!
//! Dimensions track the spatial bounds of the optimization problem with
//! version numbers to enable efficient incremental region key updates.

pub mod adjust_limits;
pub mod calculate_dimensions_key;
pub mod dimension;
pub mod divide_dimension;
pub mod expand_bounds;
pub mod new;

pub use dimension::Dimension;

/// Spatial dimensions of the optimization problem.
///
/// Each dimension represents one axis of the search space with its bounds
/// and subdivision level. The version number increments whenever bounds
/// change, allowing organisms to detect when they need to recalculate
/// their region keys.
#[derive(Debug, Clone, PartialEq)]
pub struct Dimensions {
    /// The individual dimensions (axes) of the search space
    dimensions: Vec<Dimension>,
    /// Version number, incremented on each bound change
    version: u64,
}

impl Dimensions {
    /// Returns the current version number.
    ///
    /// Version starts at 0 and increments each time `expand_bounds` is called.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns a slice of all dimensions.
    pub fn get_dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }

    /// Returns the number of spatial dimensions.
    pub fn num_dimensions(&self) -> usize {
        self.dimensions.len()
    }

    /// Returns a reference to a specific dimension by its index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn get_dimension(&self, index: usize) -> &Dimension {
        &self.dimensions[index]
    }

    /// Returns a mutable reference to a specific dimension by its index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn get_dimension_mut(&mut self, index: usize) -> &mut Dimension {
        &mut self.dimensions[index]
    }

    /// Calculates the total number of possible regions.
    ///
    /// This is the product of the number of intervals in each dimension.
    pub fn get_total_possible_regions(&self) -> usize {
        if self.dimensions.is_empty() {
            return 1;
        }
        let product: f64 = self.dimensions.iter().map(|d| d.num_intervals()).product();
        product as usize
    }
}

impl Default for Dimensions {
    /// Creates an empty Dimensions with version 0.
    fn default() -> Self {
        Self {
            dimensions: Vec::new(),
            version: 0,
        }
    }
}

#[cfg(test)]
impl Dimensions {
    /// Test-only constructor to create a `Dimensions` object with a specific set of `Dimension`s.
    pub fn new_for_test(dimensions: Vec<Dimension>) -> Self {
        Self {
            dimensions,
            version: 0,
        }
    }

    /// Test-only constructor with explicit version.
    pub fn new_for_test_with_version(dimensions: Vec<Dimension>, version: u64) -> Self {
        Self {
            dimensions,
            version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_new_for_test_when_version_called_then_returns_zero() {
        let dims = Dimensions::new_for_test(vec![]);
        assert_eq!(dims.version(), 0);
    }

    #[test]
    fn given_new_for_test_with_version_when_version_called_then_returns_specified_version() {
        let dims = Dimensions::new_for_test_with_version(vec![], 42);
        assert_eq!(dims.version(), 42);
    }

    #[test]
    fn given_empty_dimensions_when_get_total_possible_regions_then_returns_one() {
        let dims = Dimensions::new_for_test(vec![]);
        assert_eq!(dims.get_total_possible_regions(), 1);
    }

    #[test]
    fn given_dimensions_when_num_dimensions_then_returns_count() {
        let dims = Dimensions::new_for_test(vec![
            Dimension::new(-10.0..=10.0),
            Dimension::new(-5.0..=5.0),
        ]);
        assert_eq!(dims.num_dimensions(), 2);
    }
}
