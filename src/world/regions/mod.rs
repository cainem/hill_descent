use std::collections::BTreeMap;

use region::Region;

pub mod add_phenotypes;
pub mod region;
pub mod reset;
pub mod update;

use crate::parameters::global_constants::GlobalConstants;

#[derive(Debug, Clone)]
pub struct Regions {
    regions: BTreeMap<Vec<usize>, Region>,
    max_regions: usize,
}

impl Regions {
    pub fn new(global_constants: &GlobalConstants) -> Self {
        if global_constants.max_regions() == 0 {
            // This panic is consistent with Dimensions::new behaviour
            panic!("max_regions must be greater than 0 for Regions initialization.");
        }
        Self {
            regions: BTreeMap::new(),
            max_regions: global_constants.max_regions(),
        }
    }

    pub fn get_region(&self, key: &Vec<usize>) -> Option<&Region> {
        self.regions.get(key)
    }

    pub fn regions(&self) -> &BTreeMap<Vec<usize>, Region> {
        &self.regions
    }
}
