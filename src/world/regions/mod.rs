use std::collections::BTreeMap;

use region::Region;

pub mod add_phenotypes;
pub mod handle_out_of_bounds;
pub mod handle_successful_update;
pub mod prune_empty_regions;
pub mod region;
pub mod reset;
pub mod update;
pub mod update_all_region_min_scores;
pub mod update_carrying_capacities;

use crate::parameters::global_constants::GlobalConstants;

#[derive(Debug, Clone)]
pub struct Regions {
    regions: BTreeMap<Vec<usize>, Region>,
    max_regions: usize,
    population_size: usize, // Added population_size
}

impl Regions {
    pub fn new(global_constants: &GlobalConstants) -> Self {
        if global_constants.population_size() == 0 {
            // Consistent with max_regions check, though population_size=0 might be a valid scenario for some tests.
            // However, for carrying capacity calculation, P > 0 is implied by the PDD formula.
            panic!(
                "population_size must be greater than 0 for Regions initialization if carrying capacities are to be calculated."
            );
        }
        if global_constants.max_regions() == 0 {
            // This panic is consistent with Dimensions::new behaviour
            panic!("max_regions must be greater than 0 for Regions initialization.");
        }
        Self {
            regions: BTreeMap::new(),
            max_regions: global_constants.max_regions(),
            population_size: global_constants.population_size(), // Initialize population_size
        }
    }

    pub fn get_region(&self, key: &Vec<usize>) -> Option<&Region> {
        self.regions.get(key)
    }

    pub fn regions(&self) -> &BTreeMap<Vec<usize>, Region> {
        &self.regions
    }

    /// Returns a mutable reference to the regions map.
    pub fn regions_mut(&mut self) -> &mut BTreeMap<Vec<usize>, Region> {
        &mut self.regions
    }
}
