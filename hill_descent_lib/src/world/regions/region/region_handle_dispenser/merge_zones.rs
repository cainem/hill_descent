use crate::world::regions::region::region_handle_dispenser::RegionHandleDispenser;

impl RegionHandleDispenser {
    /// Merge all provided zone IDs into a single canonical zone.
    /// For determinism, the canonical ID is the minimum of zones_to_merge.
    #[allow(dead_code)]
    pub fn merge_zones(&mut self, _zones_to_merge: Vec<usize>) {
        // Iterate key_mapper and relabel any RegionHandle with an adjacent_zone_id in zones_to_merge
        // to the canonical (min) zone id. No attempt is made to reuse freed IDs later.
        todo!();
    }
}
