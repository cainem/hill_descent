use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};

pub enum RegionResult {
    DoubleRegions, // we need to divide the next dimension in line hence doubling the regions
    ExpandDimension(usize), // we need to double the range of the dimensions returned
    Complete,
}

impl Regions {
    pub fn update(&mut self, organisms: &mut Organisms, dimensions: &mut Dimensions) {
        // Call the update_step method to perform the update logic

        loop {
            let result = self.update_step(organisms, dimensions);

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
    ) -> RegionResult {
        // clear the btreemap

        // for each organism work out the region keys by seeing which dimension range they fall in
        // this can be worked out by calling get_intervals on the dimension in question

        // assign the organisms to their appropriate regions tracking the number of distinct locations within each region
        // (multiple organisms can have the same location)
        // track dimensions that are not big enough to hold any organism
        // if there are any dimensions that are not big enough to hold any organism then we need to expand the dimensions
        // return ExpandDimensions with the dimensions that need to be expanded

        // if the number of populated regions is greater that max_regions / 2 then we are finished return Complete
        // if the number of regions equals the number of distinct organism locations then we are finished return Complete
        // else return DoubleRegions

        todo!();
    }
}
