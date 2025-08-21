use std::collections::BTreeMap;

use crate::world::regions::region::region_key_dispenser::region_key::RegionKey;

mod get_adjacent_full_keys;
mod merge_zones;
mod region_key;
mod reset;
mod insert_region_key;

pub struct RegionKeyDispenser {
    key_mapper: BTreeMap<Vec<usize>, RegionKey>,
    next_key: usize,
    next_adjacent_zone_key: usize,
}
