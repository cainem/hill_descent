use std::collections::BTreeMap;

use region::Region;

pub mod region;
pub mod update;

#[derive(Debug, Clone, Default)]
pub struct Regions {
    _regions: BTreeMap<Vec<usize>, Region>,
}

impl Regions {
    pub fn get_region(&self, key: &Vec<usize>) -> Option<&Region> {
        self._regions.get(key)
    }

    pub fn regions(&self) -> &BTreeMap<Vec<usize>, Region> {
        &self._regions
    }
}
