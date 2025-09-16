use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use region::Region;

pub mod add_organisms;
pub mod adjust_regions;
pub mod handle_out_of_bounds;
pub mod prune_empty_regions;
pub mod region;
pub mod sort_regions;
pub mod truncate_regions;
pub mod update;
pub mod update_carrying_capacities;

pub mod calculate_dimension_stats;
pub mod count_unique_values_with_tolerance;
pub mod find_most_diverse_index;
pub mod get_most_common_key;
pub mod get_most_diverse_dimension;
mod refill;
pub mod repopulate;

use crate::parameters::global_constants::GlobalConstants;

#[derive(Debug, Clone)]
// Container managing all Region instances and enforcing global constraints such as maximum regions and population size.
pub struct Regions {
    regions: IndexMap<Vec<usize>, Region, FxBuildHasher>,
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
            regions: IndexMap::with_hasher(FxBuildHasher),
            target_regions: global_constants.target_regions(),
            population_size: global_constants.population_size(), // Initialize population_size
        }
    }

    // Encapsulated read operations

    /// Returns the number of regions.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Returns true if there are no regions.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Returns a reference to the region associated with the given key.
    #[allow(dead_code)]
    pub fn get_region(&self, key: &[usize]) -> Option<&Region> {
        self.regions.get(key)
    }

    /// Returns a mutable reference to the region associated with the given key.
    #[allow(dead_code)]
    pub fn get_region_mut(&mut self, key: &[usize]) -> Option<&mut Region> {
        self.regions.get_mut(key)
    }

    /// Returns an iterator over (key, region) pairs.
    pub fn iter_regions(&self) -> impl Iterator<Item = (&Vec<usize>, &Region)> {
        self.regions.iter()
    }

    /// Returns a mutable iterator over (key, region) pairs.
    #[allow(dead_code)]
    pub fn iter_regions_mut(&mut self) -> impl Iterator<Item = (&Vec<usize>, &mut Region)> {
        self.regions.iter_mut()
    }

    /// Returns an iterator over region values only.
    #[allow(dead_code)]
    pub fn iter_region_values(&self) -> impl Iterator<Item = &Region> {
        self.regions.values()
    }

    /// Returns a mutable iterator over region values only.
    pub fn iter_region_values_mut(&mut self) -> impl Iterator<Item = &mut Region> {
        self.regions.values_mut()
    }

    /// Returns an iterator over region keys only.
    #[allow(dead_code)]
    pub fn iter_region_keys(&self) -> impl Iterator<Item = &Vec<usize>> {
        self.regions.keys()
    }

    // Encapsulated write operations

    /// Inserts a region with the given key. Returns the previous region if one existed.
    #[allow(dead_code)]
    pub fn insert_region(&mut self, key: Vec<usize>, region: Region) -> Option<Region> {
        self.regions.insert(key, region)
    }

    /// Removes a region with the given key. Returns the removed region if it existed.
    #[allow(dead_code)]
    pub fn remove_region(&mut self, key: &[usize]) -> Option<Region> {
        self.regions.shift_remove(key)
    }

    /// Retains only the regions for which the predicate returns true.
    pub fn retain_regions<F>(&mut self, predicate: F)
    where
        F: FnMut(&Vec<usize>, &mut Region) -> bool,
    {
        self.regions.retain(predicate)
    }

    /// Returns true if a region with the given key exists.
    #[allow(dead_code)]
    pub fn contains_region_key(&self, key: &[usize]) -> bool {
        self.regions.contains_key(key)
    }

    // Specialized operations for common patterns

    /// Collects all region keys into a vector. Useful for iteration when mutations are needed.
    #[allow(dead_code)]
    pub fn collect_region_keys(&self) -> Vec<Vec<usize>> {
        self.regions.keys().cloned().collect()
    }

    /// Clears all regions from the collection.
    #[allow(dead_code)]
    pub fn clear_regions(&mut self) {
        self.regions.clear()
    }
}

#[cfg(test)]
mod encapsulation_tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;

    fn create_test_regions() -> Regions {
        let global_constants = GlobalConstants::new(100, 10);
        Regions::new(&global_constants)
    }

    fn create_test_region() -> Region {
        Region::new()
    }

    // len() tests

    #[test]
    fn given_empty_regions_when_len_then_returns_zero() {
        let regions = create_test_regions();
        assert_eq!(regions.len(), 0);
    }

    #[test]
    fn given_regions_with_single_region_when_len_then_returns_one() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        assert_eq!(regions.len(), 1);
    }

    #[test]
    fn given_regions_with_multiple_regions_when_len_then_returns_correct_count() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());
        regions.insert_region(vec![2, 3], create_test_region());
        assert_eq!(regions.len(), 3);
    }

    // is_empty() tests

    #[test]
    fn given_empty_regions_when_is_empty_then_returns_true() {
        let regions = create_test_regions();
        assert!(regions.is_empty());
    }

    #[test]
    fn given_regions_with_single_region_when_is_empty_then_returns_false() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        assert!(!regions.is_empty());
    }

    // get_region() tests

    #[test]
    fn given_empty_regions_when_get_region_then_returns_none() {
        let regions = create_test_regions();
        assert!(regions.get_region(&[0, 1]).is_none());
    }

    #[test]
    fn given_regions_with_region_when_get_region_with_existing_key_then_returns_some() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        regions.insert_region(key.clone(), create_test_region());

        let result = regions.get_region(&key);
        assert!(result.is_some());
    }

    #[test]
    fn given_regions_with_region_when_get_region_with_non_existing_key_then_returns_none() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());

        let result = regions.get_region(&[1, 2]);
        assert!(result.is_none());
    }

    // get_region_mut() tests

    #[test]
    fn given_empty_regions_when_get_region_mut_then_returns_none() {
        let mut regions = create_test_regions();
        assert!(regions.get_region_mut(&[0, 1]).is_none());
    }

    #[test]
    fn given_regions_with_region_when_get_region_mut_with_existing_key_then_returns_some() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        regions.insert_region(key.clone(), create_test_region());

        let result = regions.get_region_mut(&key);
        assert!(result.is_some());
    }

    #[test]
    fn given_regions_with_region_when_get_region_mut_with_non_existing_key_then_returns_none() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());

        let result = regions.get_region_mut(&[1, 2]);
        assert!(result.is_none());
    }

    #[test]
    fn given_regions_with_region_when_get_region_mut_then_allows_mutation() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let mut region = create_test_region();
        region.set_min_score(Some(42.0));
        regions.insert_region(key.clone(), region);

        if let Some(region_mut) = regions.get_region_mut(&key) {
            region_mut.set_min_score(Some(99.0));
        }

        assert_eq!(regions.get_region(&key).unwrap().min_score(), Some(99.0));
    }

    // iter_regions() tests

    #[test]
    fn given_empty_regions_when_iter_regions_then_yields_no_items() {
        let regions = create_test_regions();
        let count = regions.iter_regions().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn given_regions_with_single_region_when_iter_regions_then_yields_one_item() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());

        let count = regions.iter_regions().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn given_regions_with_multiple_regions_when_iter_regions_then_yields_all_items() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());
        regions.insert_region(vec![2, 3], create_test_region());

        let count = regions.iter_regions().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn given_regions_with_region_when_iter_regions_then_yields_correct_key_value_pairs() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let mut region = create_test_region();
        region.set_min_score(Some(42.0));
        regions.insert_region(key.clone(), region);

        let items: Vec<_> = regions.iter_regions().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].0, &key);
        assert_eq!(items[0].1.min_score(), Some(42.0));
    }

    // iter_regions_mut() tests

    #[test]
    fn given_empty_regions_when_iter_regions_mut_then_yields_no_items() {
        let mut regions = create_test_regions();
        let count = regions.iter_regions_mut().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn given_regions_with_region_when_iter_regions_mut_then_allows_mutation() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let mut region = create_test_region();
        region.set_min_score(Some(42.0));
        regions.insert_region(key.clone(), region);

        for (_, region) in regions.iter_regions_mut() {
            region.set_min_score(Some(99.0));
        }

        assert_eq!(regions.get_region(&key).unwrap().min_score(), Some(99.0));
    }

    // iter_region_values() tests

    #[test]
    fn given_empty_regions_when_iter_region_values_then_yields_no_items() {
        let regions = create_test_regions();
        let count = regions.iter_region_values().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn given_regions_with_multiple_regions_when_iter_region_values_then_yields_all_values() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());

        let count = regions.iter_region_values().count();
        assert_eq!(count, 2);
    }

    // iter_region_values_mut() tests

    #[test]
    fn given_empty_regions_when_iter_region_values_mut_then_yields_no_items() {
        let mut regions = create_test_regions();
        let count = regions.iter_region_values_mut().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn given_regions_with_region_when_iter_region_values_mut_then_allows_mutation() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let mut region = create_test_region();
        region.set_min_score(Some(42.0));
        regions.insert_region(key.clone(), region);

        for region in regions.iter_region_values_mut() {
            region.set_min_score(Some(99.0));
        }

        assert_eq!(regions.get_region(&key).unwrap().min_score(), Some(99.0));
    }

    // iter_region_keys() tests

    #[test]
    fn given_empty_regions_when_iter_region_keys_then_yields_no_items() {
        let regions = create_test_regions();
        let count = regions.iter_region_keys().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn given_regions_with_multiple_regions_when_iter_region_keys_then_yields_all_keys() {
        let mut regions = create_test_regions();
        let key1 = vec![0, 1];
        let key2 = vec![1, 2];
        regions.insert_region(key1.clone(), create_test_region());
        regions.insert_region(key2.clone(), create_test_region());

        let keys: Vec<_> = regions.iter_region_keys().collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&key1));
        assert!(keys.contains(&&key2));
    }

    // insert_region() tests

    #[test]
    fn given_empty_regions_when_insert_region_then_returns_none_and_adds_region() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let region = create_test_region();

        let result = regions.insert_region(key.clone(), region);

        assert!(result.is_none());
        assert_eq!(regions.len(), 1);
        assert!(regions.contains_region_key(&key));
    }

    #[test]
    fn given_regions_with_existing_key_when_insert_region_then_returns_previous_region() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let mut old_region = create_test_region();
        old_region.set_min_score(Some(42.0));
        let mut new_region = create_test_region();
        new_region.set_min_score(Some(99.0));

        regions.insert_region(key.clone(), old_region);
        let result = regions.insert_region(key.clone(), new_region);

        assert!(result.is_some());
        assert_eq!(result.unwrap().min_score(), Some(42.0));
        assert_eq!(regions.get_region(&key).unwrap().min_score(), Some(99.0));
    }

    // remove_region() tests

    #[test]
    fn given_empty_regions_when_remove_region_then_returns_none() {
        let mut regions = create_test_regions();
        let result = regions.remove_region(&[0, 1]);
        assert!(result.is_none());
    }

    #[test]
    fn given_regions_with_region_when_remove_region_with_existing_key_then_returns_region_and_removes_it()
     {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        let mut region = create_test_region();
        region.set_min_score(Some(42.0));
        regions.insert_region(key.clone(), region);

        let result = regions.remove_region(&key);

        assert!(result.is_some());
        assert_eq!(result.unwrap().min_score(), Some(42.0));
        assert_eq!(regions.len(), 0);
        assert!(!regions.contains_region_key(&key));
    }

    #[test]
    fn given_regions_with_region_when_remove_region_with_non_existing_key_then_returns_none() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());

        let result = regions.remove_region(&[1, 2]);
        assert!(result.is_none());
        assert_eq!(regions.len(), 1);
    }

    // retain_regions() tests

    #[test]
    fn given_empty_regions_when_retain_regions_then_remains_empty() {
        let mut regions = create_test_regions();
        regions.retain_regions(|_, _| true);
        assert!(regions.is_empty());
    }

    #[test]
    fn given_regions_when_retain_regions_with_always_true_predicate_then_keeps_all_regions() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());

        regions.retain_regions(|_, _| true);
        assert_eq!(regions.len(), 2);
    }

    #[test]
    fn given_regions_when_retain_regions_with_always_false_predicate_then_removes_all_regions() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());

        regions.retain_regions(|_, _| false);
        assert!(regions.is_empty());
    }

    #[test]
    fn given_regions_when_retain_regions_with_selective_predicate_then_keeps_matching_regions() {
        let mut regions = create_test_regions();
        let mut region1 = create_test_region();
        region1.set_min_score(Some(10.0));
        let mut region2 = create_test_region();
        region2.set_min_score(Some(50.0));

        regions.insert_region(vec![0, 1], region1);
        regions.insert_region(vec![1, 2], region2);

        // Keep only regions with min_score > 25.0
        regions.retain_regions(|_, region| region.min_score().is_some_and(|score| score > 25.0));

        assert_eq!(regions.len(), 1);
        assert!(regions.contains_region_key(&[1, 2]));
        assert!(!regions.contains_region_key(&[0, 1]));
    }

    // contains_region_key() tests

    #[test]
    fn given_empty_regions_when_contains_region_key_then_returns_false() {
        let regions = create_test_regions();
        assert!(!regions.contains_region_key(&[0, 1]));
    }

    #[test]
    fn given_regions_with_region_when_contains_region_key_with_existing_key_then_returns_true() {
        let mut regions = create_test_regions();
        let key = vec![0, 1];
        regions.insert_region(key.clone(), create_test_region());

        assert!(regions.contains_region_key(&key));
    }

    #[test]
    fn given_regions_with_region_when_contains_region_key_with_non_existing_key_then_returns_false()
    {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());

        assert!(!regions.contains_region_key(&[1, 2]));
    }

    // collect_region_keys() tests

    #[test]
    fn given_empty_regions_when_collect_region_keys_then_returns_empty_vector() {
        let regions = create_test_regions();
        let keys = regions.collect_region_keys();
        assert!(keys.is_empty());
    }

    #[test]
    fn given_regions_with_multiple_regions_when_collect_region_keys_then_returns_all_keys() {
        let mut regions = create_test_regions();
        let key1 = vec![0, 1];
        let key2 = vec![1, 2];
        let key3 = vec![2, 3];

        regions.insert_region(key1.clone(), create_test_region());
        regions.insert_region(key2.clone(), create_test_region());
        regions.insert_region(key3.clone(), create_test_region());

        let keys = regions.collect_region_keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&key1));
        assert!(keys.contains(&key2));
        assert!(keys.contains(&key3));
    }

    // clear_regions() tests

    #[test]
    fn given_empty_regions_when_clear_regions_then_remains_empty() {
        let mut regions = create_test_regions();
        regions.clear_regions();
        assert!(regions.is_empty());
    }

    #[test]
    fn given_regions_with_multiple_regions_when_clear_regions_then_becomes_empty() {
        let mut regions = create_test_regions();
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());
        regions.insert_region(vec![2, 3], create_test_region());

        regions.clear_regions();
        assert!(regions.is_empty());
        assert_eq!(regions.len(), 0);
    }

    // Integration tests for complex scenarios

    #[test]
    fn given_regions_when_multiple_operations_then_maintains_consistency() {
        let mut regions = create_test_regions();

        // Insert some regions
        regions.insert_region(vec![0, 1], create_test_region());
        regions.insert_region(vec![1, 2], create_test_region());
        regions.insert_region(vec![2, 3], create_test_region());
        assert_eq!(regions.len(), 3);

        // Remove one
        let removed = regions.remove_region(&[1, 2]);
        assert!(removed.is_some());
        assert_eq!(regions.len(), 2);

        // Check remaining keys
        let keys = regions.collect_region_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&vec![0, 1]));
        assert!(keys.contains(&vec![2, 3]));
        assert!(!keys.contains(&vec![1, 2]));

        // Clear all
        regions.clear_regions();
        assert!(regions.is_empty());
    }
}
