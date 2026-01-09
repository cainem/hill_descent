//! Regions module - spatial partitions for organism management.
//!
//! Regions group organisms by their spatial location (region key) and handle
//! carrying capacity calculation, organism culling, and reproduction selection.

pub mod organism_entry;
pub mod populate;
pub mod process;
pub mod region;
pub mod region_key;
pub mod update_carrying_capacities;

pub use organism_entry::OrganismEntry;
pub use region::Region;
pub use region_key::RegionKey;

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::parameters::GlobalConstants;

/// Container managing all Region instances.
///
/// Regions are ephemeral - rebuilt each training run from organism region keys.
/// Uses Rayon for parallel processing (not in a thread pool).
#[derive(Debug, Clone)]
pub struct Regions {
    /// Map from region key to region
    regions: IndexMap<RegionKey, Region, FxBuildHasher>,
    /// Target "ideal" number of regions
    target_regions: usize,
    /// Total population size for capacity calculations
    population_size: usize,
}

impl Regions {
    /// Creates a new Regions container.
    ///
    /// # Arguments
    ///
    /// * `global_constants` - Configuration containing population size and target regions
    ///
    /// # Panics
    ///
    /// Panics if population_size or target_regions is 0.
    pub fn new(global_constants: &GlobalConstants) -> Self {
        if global_constants.population_size() == 0 {
            panic!("population_size must be greater than 0 for Regions initialization.");
        }
        if global_constants.target_regions() == 0 {
            panic!("target_regions must be greater than 0 for Regions initialization.");
        }
        Self {
            regions: IndexMap::with_hasher(FxBuildHasher),
            target_regions: global_constants.target_regions(),
            population_size: global_constants.population_size(),
        }
    }

    /// Returns the number of regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Returns true if there are no regions.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Returns a reference to the region associated with the given key.
    pub fn get_region(&self, key: &RegionKey) -> Option<&Region> {
        self.regions.get(key)
    }

    /// Returns a mutable reference to the region associated with the given key.
    pub fn get_region_mut(&mut self, key: &RegionKey) -> Option<&mut Region> {
        self.regions.get_mut(key)
    }

    /// Returns the target number of regions.
    pub fn target_regions(&self) -> usize {
        self.target_regions
    }

    /// Returns the population size.
    pub fn population_size(&self) -> usize {
        self.population_size
    }

    /// Clears all regions.
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Returns an iterator over all regions.
    pub fn iter(&self) -> impl Iterator<Item = (&RegionKey, &Region)> {
        self.regions.iter()
    }

    /// Returns a mutable iterator over all regions.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&RegionKey, &mut Region)> {
        self.regions.iter_mut()
    }

    /// Inserts or gets a region for the given key.
    pub fn get_or_insert(&mut self, key: RegionKey) -> &mut Region {
        self.regions.entry(key).or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_constants(pop: usize, regions: usize) -> GlobalConstants {
        GlobalConstants::new(pop, regions)
    }

    #[test]
    fn given_valid_constants_when_new_then_regions_created() {
        let regions = Regions::new(&make_constants(100, 10));
        assert!(regions.is_empty());
        assert_eq!(regions.target_regions(), 10);
        assert_eq!(regions.population_size(), 100);
    }

    #[test]
    #[should_panic(expected = "Population size cannot be zero")]
    fn given_zero_population_when_new_then_panics() {
        Regions::new(&make_constants(0, 10));
    }

    #[test]
    #[should_panic(expected = "Max regions cannot be zero")]
    fn given_zero_target_regions_when_new_then_panics() {
        Regions::new(&make_constants(100, 0));
    }

    #[test]
    fn given_empty_regions_when_len_then_returns_zero() {
        let regions = Regions::new(&make_constants(100, 10));
        assert_eq!(regions.len(), 0);
    }
}
