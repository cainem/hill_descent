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
        _organisms: &mut Organisms,
        _dimensions: &mut Dimensions,
        _distinct_locations_count: usize,
    ) -> RegionResult {
        // for each organism work out the region keys by seeing which dimension range they fall in
        // this can be worked out by calling get_intervals on the dimension in question

        // assign the organisms to their appropriate regions
        // if a dimension is not big enough to hold any organisms return immediately with ExpandDimension

        // if the number of populated regions is greater that max_regions / 2 then we are finished return Complete
        // if the number of regions equals the number of distinct organism locations then we are finished return Complete
        // else return DoubleRegions

        todo!();
    }
}
