use crate::world::regions::region_handle_dispenser::RegionHandleDispenser;
use crate::world::regions::region_handle_dispenser::region_handle::RegionHandle;

impl RegionHandleDispenser {
    /// Returns the existing RegionHandle for the given full_region_key if present,
    /// otherwise computes adjacency, merges zones if needed, creates a new handle,
    /// inserts it, and returns it.
    ///
    /// - full_region_key: slice of per-dimension interval indices
    /// - Determinism: relies on BTreeMap iteration order where relevant
    #[allow(dead_code)]
    pub fn get_or_insert(&mut self, _full_region_key: &[usize]) -> RegionHandle {
        // 1) Lookup existing
        // if let Some(handle) = self.key_mapper.get(full_region_key) { return *handle; }
        //
        // 2) Find neighboring existing keys (±1 in one dimension) and collect their zone ids
        // let neighbors = self.get_adjacent_full_keys(full_region_key);
        // let mut neighbor_zones: Vec<usize> = neighbors
        //     .iter()
        //     .filter_map(|k| self.key_mapper.get(k))
        //     .map(|h| h.adjacent_zone_id())
        //     .collect();
        // neighbor_zones.sort_unstable();
        // neighbor_zones.dedup();
        //
        // 3) Determine zone assignment
        // let zone_id = match neighbor_zones.as_slice() {
        //     [] => { // no neighbors
        //         let id = self.next_adjacent_zone_id;
        //         self.next_adjacent_zone_id += 1;
        //         id
        //     }
        //     [single] => *single,
        //     _ => { // multiple zones -> merge into min
        //         self.merge_zones(neighbor_zones.clone());
        //         neighbor_zones[0]
        //     }
        // };
        //
        // 4) Allocate new handle id and insert
        // let handle = RegionHandle::new(self.next_id, zone_id);
        // self.next_id += 1;
        // self.key_mapper.insert(full_region_key.to_vec(), handle);
        // handle
        todo!()
    }
}
