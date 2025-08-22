use crate::world::regions::region_handle_dispenser::RegionHandleDispenser;

impl RegionHandleDispenser {
    /// Returns all existing full_region_keys in the dispenser that are adjacent
    /// to the provided full_region_key (differ by exactly one coordinate by ±1).
    ///
    /// Note: returns owned keys for simplicity and to avoid borrowing complexity.
    #[allow(dead_code)]
    pub fn get_adjacent_full_keys(&self, _full_region_key: &[usize]) -> Vec<Vec<usize>> {
        // Strategy: generate 2*d neighbor keys and probe the map. Collect any that exist.
        // Deterministic order is preserved by iterating dimensions in index order and probing
        // via the BTreeMap.
        todo!()
    }
}
