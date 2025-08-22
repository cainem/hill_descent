use std::collections::BTreeMap;

use self::region_handle::RegionHandle;

mod get_adjacent_full_keys;
mod get_or_insert;
mod merge_zones;
mod region_handle;
mod reset;

/// Manages compact handles for regions in an n-dimensional grid.
/// Maps potentially large full_region_keys (Vec<usize>) to small, stable RegionHandles
/// that are cheaper to store and compare. Also tracks spatial adjacency zones for
/// future carrying capacity allocation strategies.
#[derive(Default)]
#[allow(dead_code)]
pub struct RegionHandleDispenser {
    /// Maps a full_region_key (Vec of per-dimension indices) to a compact RegionHandle
    key_mapper: BTreeMap<Vec<usize>, RegionHandle>,
    /// Monotonic counter for assigning unique region IDs
    next_id: usize,
    /// Monotonic counter for assigning unique adjacency zone IDs
    next_adjacent_zone_id: usize,
}
