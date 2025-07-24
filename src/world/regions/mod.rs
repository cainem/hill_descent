use std::collections::BTreeMap;

use region::Region;

pub mod add_phenotypes;
pub mod handle_out_of_bounds;
pub mod handle_successful_update;
pub mod prune_empty_regions;
pub mod region;

pub mod calculate_dimension_stats;
pub mod count_unique_values_with_tolerance;
pub mod find_most_diverse_index;
pub mod get_most_common_key;
pub mod get_most_diverse_dimension;
mod refill;
pub mod repopulate;
pub mod update;
pub mod update_all_region_min_scores;
pub mod update_carrying_capacities;

use crate::parameters::global_constants::GlobalConstants;

#[derive(Debug, Clone)]
// Container managing all Region instances and enforcing global constraints such as maximum regions and population size.
pub struct Regions {
    regions: BTreeMap<Vec<usize>, Region>,
    // the target "ideal" number of regions
    // the algorithm doesn't strictly enforce it as a maximum number of regions
    // but it won't be more that target_regions * 2
    target_regions: usize,
    population_size: usize,
}

impl Regions {
    pub fn new(global_constants: &GlobalConstants) -> Self {
        if global_constants.population_size() == 0 {
            // Consistent with target_regions check, though population_size=0 might be a valid scenario for some tests.
            // However, for carrying capacity calculation, P > 0 is implied by the PDD formula.
            panic!(
                "population_size must be greater than 0 for Regions initialization if carrying capacities are to be calculated."
            );
        }
        if global_constants.target_regions() == 0 {
            // This panic is consistent with Dimensions::new behaviour
            panic!("target_regions must be greater than 0 for Regions initialization.");
        }
        Self {
            regions: BTreeMap::new(),
            target_regions: global_constants.target_regions(),
            population_size: global_constants.population_size(), // Initialize population_size
        }
    }

    pub fn regions(&self) -> &BTreeMap<Vec<usize>, Region> {
        &self.regions
    }

    /// Returns a mutable reference to the regions map.
    pub fn regions_mut(&mut self) -> &mut BTreeMap<Vec<usize>, Region> {
        &mut self.regions
    }
}
