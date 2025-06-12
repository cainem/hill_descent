use crate::world::organisms::organism::update_region_key::OrganismUpdateRegionKeyResult;
use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};

pub enum RegionResult {
    DoubleRegions, // we need to divide the next dimension in line hence doubling the regions
    ExpandDimension(usize), // we need to double the range of the dimensions returned
    Complete,
}

impl Regions {
    pub fn update(&mut self, organisms: &mut Organisms, dimensions: &mut Dimensions) {
        // get total number of distinct locations for organisms
        let distinct_locations_count = organisms.distinct_locations_count();

        // empty all of the regions in the btreemap (if the regions don't need to double or the dimensions expand we can reuse them)
        loop {
            let result = self.update_step(organisms, dimensions, distinct_locations_count);

            // Handle the result of the update step
            match result {
                RegionResult::DoubleRegions => {
                    // empty to btreemap
                    // Logic for doubling regions if needed
                }
                RegionResult::ExpandDimension(_dimensions_to_expand) => {
                    // empty the btreemap
                    // Logic for expanding dimensions based on the provided indices
                }
                RegionResult::Complete => {
                    // Logic for completing the update process
                    // break out of loop when complete
                }
            }
        }
    }

    fn update_step(
        &mut self,
        organisms: &mut Organisms,
        dimensions: &mut Dimensions,
        distinct_locations_count: usize,
    ) -> RegionResult {
        // Attempt to update region keys for all organisms based on the current dimensions.
        match organisms.update_all_region_keys(dimensions) {
            OrganismUpdateRegionKeyResult::Success => {
                // All organisms' region keys updated successfully.
                // Populate the regions with organisms based on their new keys.
                self.add_phenotypes(organisms);

                let num_populated_regions = self.regions().len();

                // Decide the next step based on region population and distinct locations:
                // - If regions are sufficiently populated or all distinct locations have a region, the update is complete.
                // - Otherwise, regions may need to be doubled (e.g., by splitting a dimension further).
                if num_populated_regions > self.max_regions / 2 {
                    RegionResult::Complete
                } else if distinct_locations_count > 0
                    && num_populated_regions == distinct_locations_count
                {
                    // (Ensure distinct_locations_count > 0 to avoid premature completion with no organisms)
                    RegionResult::Complete
                } else {
                    // Otherwise, regions need to be doubled (e.g., by splitting a dimension).
                    RegionResult::DoubleRegions
                }
            }
            OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index) => {
                // If any organism fails, we need to expand the dimension at the given index
                RegionResult::ExpandDimension(dimension_index)
            }
        }
    }
}
