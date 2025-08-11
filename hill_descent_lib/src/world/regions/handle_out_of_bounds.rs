use crate::world::dimensions::Dimensions;

impl super::Regions {
    /// Handles the scenario where an organism is found to be outside the current
    /// world bounds. It expands the necessary dimension to include the organism.
    ///
    /// # Arguments
    ///
    /// * `dimensions` - A mutable reference to the world's dimensions.
    /// * `dimension_index` - The index of the dimension that needs to be expanded.
    ///
    /// This function also clears all existing regions, as the change in dimensions
    /// invalidates all current region keys.
    pub(super) fn handle_out_of_bounds(
        &mut self,
        dimensions: &mut Dimensions,
        dimension_index: usize,
    ) {
        dimensions.expand_bounds(dimension_index);
        // The dimension change invalidates all existing region keys.
        // Clear all regions so they can be rebuilt in the next iteration.
        self.regions.clear();
    }
}
