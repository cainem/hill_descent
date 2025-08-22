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
    pub fn get_or_insert(&mut self, full_region_key: &[usize]) -> RegionHandle {
        // 1) Lookup existing
        if let Some(handle) = self.key_mapper.get(full_region_key) {
            return *handle;
        }

        // 2) Find neighbouring existing keys (±1 in one dimension) and collect their zone ids
        let neighbors = self.get_adjacent_full_keys(full_region_key);
        let mut neighbor_zones: Vec<usize> = neighbors
            .iter()
            .filter_map(|k| self.key_mapper.get(k))
            .map(|h| h.adjacent_zone_id())
            .collect();
        neighbor_zones.sort_unstable();
        neighbor_zones.dedup();

        // 3) Determine zone assignment
        let zone_id = match neighbor_zones.as_slice() {
            [] => {
                // no neighbors -> allocate new zone id
                let id = self.next_adjacent_zone_id;
                self.next_adjacent_zone_id += 1;
                id
            }
            [single] => *single,
            _ => {
                // multiple zones -> merge into min (deterministic)
                self.merge_zones(neighbor_zones.clone());
                neighbor_zones[0]
            }
        };

        // 4) Allocate new handle id and insert
        let handle = RegionHandle::new(self.next_id, zone_id);
        self.next_id += 1;
        self.key_mapper.insert(full_region_key.to_vec(), handle);
        handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn h(id: usize, z: usize) -> RegionHandle {
        RegionHandle::new(id, z)
    }

    #[test]
    fn given_existing_key_when_get_or_insert_then_returns_existing_handle_and_no_change() {
        let mut d = RegionHandleDispenser::default();
        d.key_mapper = BTreeMap::from([(vec![1, 2], h(42, 7))]);
        d.next_id = 5;
        d.next_adjacent_zone_id = 3;

        let got = d.get_or_insert(&[1, 2]);
        assert_eq!(got.id(), 42);
        assert_eq!(got.adjacent_zone_id(), 7);
        // Ensure counters unchanged and map not mutated
        assert_eq!(d.next_id, 5);
        assert_eq!(d.next_adjacent_zone_id, 3);
        assert_eq!(d.key_mapper.len(), 1);
    }

    #[test]
    fn given_no_neighbors_when_get_or_insert_then_allocates_new_zone_and_id() {
        let mut d = RegionHandleDispenser::default();
        // Unrelated entries far away
        d.key_mapper = BTreeMap::from([(vec![100, 100], h(1, 99))]);
        // Defaults: next_id = 0, next_adjacent_zone_id = 0

        let got = d.get_or_insert(&[4, 5]);
        assert_eq!(got.id(), 0);
        assert_eq!(got.adjacent_zone_id(), 0);
        assert_eq!(d.next_id, 1);
        assert_eq!(d.next_adjacent_zone_id, 1);
        assert_eq!(d.key_mapper.get(&vec![4, 5]).copied(), Some(got));
    }

    #[test]
    fn given_single_neighbor_zone_when_get_or_insert_then_reuses_that_zone() {
        let mut d = RegionHandleDispenser::default();
        // Base query [4,5]; provide one adjacent neighbor [4,6] with zone 7
        d.key_mapper = BTreeMap::from([(vec![4, 6], h(10, 7))]);

        let got = d.get_or_insert(&[4, 5]);
        assert_eq!(got.id(), 0);
        assert_eq!(got.adjacent_zone_id(), 7);
        // Zone counter should remain 0 because we reused an existing zone id
        assert_eq!(d.next_adjacent_zone_id, 0);
        assert_eq!(d.next_id, 1);
    }

    #[test]
    fn given_multiple_neighbor_zones_when_get_or_insert_then_merges_and_uses_min_zone() {
        let mut d = RegionHandleDispenser::default();
        // Query [4,5]; neighbors: [3,5] in zone 10, [5,5] in zone 2
        d.key_mapper = BTreeMap::from([(vec![3, 5], h(1, 10)), (vec![5, 5], h(2, 2))]);

        let got = d.get_or_insert(&[4, 5]);
        // Canonical zone should be min(2,10) = 2
        assert_eq!(got.adjacent_zone_id(), 2);
        assert_eq!(got.id(), 0);
        // Existing entries should be relabeled to canonical id
        assert_eq!(d.key_mapper.get(&vec![3, 5]).unwrap().adjacent_zone_id(), 2);
        assert_eq!(d.key_mapper.get(&vec![5, 5]).unwrap().adjacent_zone_id(), 2);
        // Zone counter unchanged because we didn't allocate a new zone
        assert_eq!(d.next_adjacent_zone_id, 0);
        assert_eq!(d.next_id, 1);
    }
}
