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
    /// - Expands the bounds of the specified dimension
    /// - Increments the version number
    ///
    /// # Returns
    ///
    /// The indices of dimensions that changed (just `dim_idx`).
    pub fn expand_bounds(&mut self, dim_idx: usize) -> Vec<usize> {
        todo!("Implement expand_bounds with version increment")
    }

    /// Expands bounds for multiple dimensions and increments version once.
    ///
    /// # Arguments
    ///
    /// * `dim_indices` - Indices of dimensions to expand
    ///
    /// # Side Effects
    ///
    /// - Expands the bounds of all specified dimensions
    /// - Increments the version number once
    ///
    /// # Returns
    ///
    /// The indices of dimensions that changed.
    pub fn expand_bounds_multiple(&mut self, dim_indices: &[usize]) -> Vec<usize> {
        todo!("Implement expand_bounds_multiple with version increment")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_dimensions_when_expand_bounds_then_version_increments() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_dimensions_when_expand_bounds_multiple_then_version_increments_once() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_dimensions_when_expand_bounds_then_returns_changed_indices() {
        todo!()
    }
}
