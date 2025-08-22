use crate::world::regions::region_handle_dispenser::RegionHandleDispenser;

impl RegionHandleDispenser {
    /// Clears all state and resets counters (next_id and next_adjacent_zone_id) back to 0.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.key_mapper.clear();
        self.next_id = 0;
        self.next_adjacent_zone_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::regions::region_handle_dispenser::region_handle::RegionHandle;
    use std::collections::BTreeMap;

    fn h(id: usize, z: usize) -> RegionHandle { RegionHandle::new(id, z) }

    #[test]
    fn given_populated_state_when_reset_then_map_cleared_and_counters_zero() {
        let mut d = RegionHandleDispenser::default();
        d.key_mapper = BTreeMap::from([
            (vec![1, 2, 3], h(10, 5)),
            (vec![4, 5, 6], h(11, 6)),
        ]);
        d.next_id = 123;
        d.next_adjacent_zone_id = 456;

        d.reset();

        assert!(d.key_mapper.is_empty());
        assert_eq!(d.next_id, 0);
        assert_eq!(d.next_adjacent_zone_id, 0);
    }

    #[test]
    fn given_state_reset_when_insert_new_key_then_ids_restart_from_zero() {
        let mut d = RegionHandleDispenser::default();
        // Preload and then reset
        d.key_mapper = BTreeMap::from([(vec![9, 9], h(99, 9))]);
        d.next_id = 42;
        d.next_adjacent_zone_id = 7;
        d.reset();

        let handle = d.get_or_insert(&[1, 1]);
        assert_eq!(handle.id(), 0);
        assert_eq!(handle.adjacent_zone_id(), 0);
        assert_eq!(d.next_id, 1);
        assert_eq!(d.next_adjacent_zone_id, 1);
        assert_eq!(d.key_mapper.get(&vec![1, 1]).copied(), Some(handle));
    }
}
