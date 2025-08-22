use crate::world::regions::region_handle_dispenser::RegionHandleDispenser;

impl RegionHandleDispenser {
    /// Returns all existing full_region_keys in the dispenser that are adjacent
    /// to the provided full_region_key (differ by exactly one coordinate by ±1).
    ///
    /// Note: returns owned keys for simplicity and to avoid borrowing complexity.
    #[allow(dead_code)]
    pub fn get_adjacent_full_keys(&self, full_region_key: &[usize]) -> Vec<Vec<usize>> {
        // Strategy: generate 2*d neighbor keys and probe the map. Collect any that exist.
        // Deterministic order is preserved by iterating dimensions in index order and probing
        // via the BTreeMap. Order within a dimension is [-1, +1].
        let mut result: Vec<Vec<usize>> = Vec::new();

        for (i, &val) in full_region_key.iter().enumerate() {
            // Try val - 1 if it doesn't underflow
            if val > 0 {
                let mut neighbor = full_region_key.to_vec();
                neighbor[i] = val - 1;
                if self.key_mapper.contains_key(&neighbor) {
                    result.push(neighbor);
                }
            }

            // Try val + 1 if it doesn't overflow
            if let Some(vp1) = val.checked_add(1) {
                let mut neighbor = full_region_key.to_vec();
                neighbor[i] = vp1;
                if self.key_mapper.contains_key(&neighbor) {
                    result.push(neighbor);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::regions::region_handle_dispenser::region_handle::RegionHandle;
    use std::collections::BTreeMap;

    fn make_handle(id: usize, zone: usize) -> RegionHandle {
        RegionHandle::new(id, zone)
    }

    #[test]
    fn given_empty_map_when_query_any_key_then_returns_empty_neighbors() {
        let dispenser = RegionHandleDispenser::default();
        let res = dispenser.get_adjacent_full_keys(&[1, 2, 3]);
        assert!(res.is_empty());
    }

    #[test]
    fn given_populated_map_when_query_key_then_returns_existing_adjacents_in_dimension_order() {
        let mut dispenser = RegionHandleDispenser::default();
        // Populate the internal map with all 6 adjacents for base [4,5,6]
        let mut map: BTreeMap<Vec<usize>, RegionHandle> = BTreeMap::new();
        map.insert(vec![3, 5, 6], make_handle(1, 10)); // dim0 -1
        map.insert(vec![5, 5, 6], make_handle(2, 10)); // dim0 +1
        map.insert(vec![4, 4, 6], make_handle(3, 10)); // dim1 -1
        map.insert(vec![4, 6, 6], make_handle(4, 10)); // dim1 +1
        map.insert(vec![4, 5, 5], make_handle(5, 10)); // dim2 -1
        map.insert(vec![4, 5, 7], make_handle(6, 10)); // dim2 +1
        // Some non-adjacent noise
        map.insert(vec![4, 5, 8], make_handle(7, 11));
        dispenser.key_mapper = map;

        let res = dispenser.get_adjacent_full_keys(&[4, 5, 6]);

        // Expect exactly the six, in deterministic order: for each dim, - then +
        let expected = vec![
            vec![3, 5, 6],
            vec![5, 5, 6],
            vec![4, 4, 6],
            vec![4, 6, 6],
            vec![4, 5, 5],
            vec![4, 5, 7],
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn given_key_with_zero_coordinate_when_query_then_no_minus_neighbor_for_that_dim() {
        let mut dispenser = RegionHandleDispenser::default();
        let mut map: BTreeMap<Vec<usize>, RegionHandle> = BTreeMap::new();
        // Base [0,5]
        map.insert(vec![1, 5], make_handle(1, 1)); // dim0 +1 only valid
        map.insert(vec![0, 4], make_handle(2, 1)); // dim1 -1
        map.insert(vec![0, 6], make_handle(3, 1)); // dim1 +1
        dispenser.key_mapper = map;

        let res = dispenser.get_adjacent_full_keys(&[0, 5]);
        let expected = vec![vec![1, 5], vec![0, 4], vec![0, 6]];
        assert_eq!(res, expected);
    }

    #[test]
    fn given_key_with_usize_max_when_query_then_no_plus_neighbor_for_that_dim() {
        let mut dispenser = RegionHandleDispenser::default();
        let mut map: BTreeMap<Vec<usize>, RegionHandle> = BTreeMap::new();
        let umax = usize::MAX;
        map.insert(vec![umax - 1, 10], make_handle(1, 1)); // dim0 -1 exists
        map.insert(vec![umax, 9], make_handle(2, 1)); // dim1 -1
        map.insert(vec![umax, 11], make_handle(3, 1)); // dim1 +1
        dispenser.key_mapper = map;

        let res = dispenser.get_adjacent_full_keys(&[umax, 10]);
        // Expect: [umax-1,10] (dim0 -1), then dim0 +1 skipped, then dim1 -1 and +1
        let expected = vec![vec![umax - 1, 10], vec![umax, 9], vec![umax, 11]];
        assert_eq!(res, expected);
    }
}
