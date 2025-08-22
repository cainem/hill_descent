use crate::world::regions::region_handle_dispenser::RegionHandleDispenser;
use crate::world::regions::region_handle_dispenser::region_handle::RegionHandle;

impl RegionHandleDispenser {
    /// Merge all provided zone IDs into a single canonical zone.
    /// For determinism, the canonical ID is the minimum of zones_to_merge.
    #[allow(dead_code)]
    pub fn merge_zones(&mut self, mut zones_to_merge: Vec<usize>) {
        // No zones or a single zone -> nothing to do
        if zones_to_merge.len() < 2 {
            return;
        }

        // Deduplicate and find canonical (minimum) zone id
        zones_to_merge.sort_unstable();
        zones_to_merge.dedup();
        if zones_to_merge.len() < 2 {
            return;
        }
        let canonical = zones_to_merge[0];

        // Relabel any RegionHandle whose zone is in zones_to_merge to the canonical id
        for handle in self.key_mapper.values_mut() {
            let zone = handle.adjacent_zone_id();
            // zones_to_merge is sorted; use binary_search for O(log n) lookup
            if zone != canonical && zones_to_merge.binary_search(&zone).is_ok() {
                let id = handle.id();
                *handle = RegionHandle::new(id, canonical);
            }
        }
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
    fn given_empty_input_when_merge_zones_then_no_changes() {
        let mut d = RegionHandleDispenser::default();
        // Preload a couple of entries
        d.key_mapper = BTreeMap::from([(vec![1, 2], h(10, 5)), (vec![3, 4], h(11, 6))]);
        d.merge_zones(vec![]);

        assert_eq!(d.key_mapper.get(&vec![1, 2]).unwrap().adjacent_zone_id(), 5);
        assert_eq!(d.key_mapper.get(&vec![3, 4]).unwrap().adjacent_zone_id(), 6);
    }

    #[test]
    fn given_single_zone_when_merge_zones_then_no_changes() {
        let mut d = RegionHandleDispenser::default();
        d.key_mapper = BTreeMap::from([(vec![1], h(1, 7)), (vec![2], h(2, 8))]);
        d.merge_zones(vec![7]);

        assert_eq!(d.key_mapper.get(&vec![1]).unwrap().adjacent_zone_id(), 7);
        assert_eq!(d.key_mapper.get(&vec![2]).unwrap().adjacent_zone_id(), 8);
    }

    #[test]
    fn given_multiple_zones_when_merge_zones_then_all_mapped_to_min_canonical() {
        let mut d = RegionHandleDispenser::default();
        d.key_mapper = BTreeMap::from([
            (vec![0], h(1, 10)),
            (vec![1], h(2, 12)),
            (vec![2], h(3, 11)),
            (vec![3], h(4, 99)), // not in merge set
        ]);

        d.merge_zones(vec![12, 11]); // canonical should be 11

        assert_eq!(d.key_mapper.get(&vec![0]).unwrap().adjacent_zone_id(), 10);
        assert_eq!(d.key_mapper.get(&vec![1]).unwrap().adjacent_zone_id(), 11);
        assert_eq!(d.key_mapper.get(&vec![2]).unwrap().adjacent_zone_id(), 11);
        assert_eq!(d.key_mapper.get(&vec![3]).unwrap().adjacent_zone_id(), 99);

        // IDs must be preserved
        assert_eq!(d.key_mapper.get(&vec![1]).unwrap().id(), 2);
        assert_eq!(d.key_mapper.get(&vec![2]).unwrap().id(), 3);
    }

    #[test]
    fn given_duplicates_and_missing_zones_when_merge_zones_then_only_present_zones_change() {
        let mut d = RegionHandleDispenser::default();
        d.key_mapper = BTreeMap::from([
            (vec![10], h(10, 1)),
            (vec![11], h(11, 2)),
            (vec![12], h(12, 3)),
        ]);

        // 2 is duplicated, 999 not present anywhere; canonical will be 2 after sort/dedup(2,3)
        d.merge_zones(vec![2, 2, 3, 999]);

        assert_eq!(d.key_mapper.get(&vec![10]).unwrap().adjacent_zone_id(), 1);
        assert_eq!(d.key_mapper.get(&vec![11]).unwrap().adjacent_zone_id(), 2);
        assert_eq!(d.key_mapper.get(&vec![12]).unwrap().adjacent_zone_id(), 2);
    }
}
