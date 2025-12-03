//! Dimensions constructor.

use std::ops::RangeInclusive;

use super::{Dimension, Dimensions};

impl Dimensions {
    /// Creates new Dimensions from parameter bounds and target region count.
    ///
    /// # Arguments
    ///
    /// * `param_range` - Slice of ranges for each parameter dimension
    /// * `target_regions` - Target number of regions (used to calculate initial doublings)
    ///
    /// # Returns
    ///
    /// A new Dimensions with version 0.
    ///
    /// # Panics
    ///
    /// Panics if `param_range` is empty or `target_regions` is 0.
    pub fn new(param_range: &[RangeInclusive<f64>], target_regions: usize) -> Self {
        todo!("Implement Dimensions::new")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Implementation pending"]
    fn given_valid_params_when_new_then_dimensions_created() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_new_dimensions_when_version_called_then_returns_zero() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    #[should_panic]
    fn given_empty_params_when_new_then_panics() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    #[should_panic]
    fn given_zero_target_regions_when_new_then_panics() {
        todo!()
    }
}
