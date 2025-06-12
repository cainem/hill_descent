use crate::world::{dimensions::Dimensions, organisms::Organisms};

impl super::Regions {
    /// Handles the successful update of all organism region keys.
    ///
    /// This function repopulates the regions with the organisms, prunes any
    /// regions that are now empty, and then determines if the simulation
    /// should continue dividing dimensions or stop.
    ///
    /// # Returns
    ///
    /// Returns `true` if the simulation has reached a stable state and should
    /// stop, `false` otherwise.
    pub(super) fn handle_successful_update(
        &mut self,
        organisms: &mut Organisms,
        dimensions: &mut Dimensions,
    ) -> bool {
        self.add_phenotypes(organisms);
        self.prune_empty_regions();

        // Stop if we've hit the max number of regions, or if all organisms are in one region.
        if self.regions.len() >= self.max_regions || organisms.distinct_locations_count() <= 1 {
            return true; // Stable state reached.
        }

        // Stop if the number of *potential* regions (if we were to divide further)
        // has met or exceeded the maximum. This prevents attempting divisions
        // that would lead to too many conceptual regions.
        if dimensions.get_total_possible_regions() >= self.max_regions {
            return true; // Stable state: cannot refine further due to max potential regions.
        }

        // Try to divide the dimension with the highest organism count.
        if dimensions.divide_next_dimension() {
            // The dimension change invalidates all existing region keys.
            // Clear all regions so they can be rebuilt in the next iteration.
            self.regions.clear();
            false // Continue loop
        } else {
            // No more divisions possible.
            true // Stable state: cannot refine further.
        }
    }
}
