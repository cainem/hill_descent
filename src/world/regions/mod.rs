use std::collections::BTreeMap;

use region::Region;

pub mod region;

#[derive(Debug, Clone)]
pub struct Regions {
    _regions: BTreeMap<Vec<usize>, Region>,
}

impl Default for Regions {
    fn default() -> Self {
        Self::new()
    }
}

impl Regions {
    pub fn new() -> Self {
        Regions {
            _regions: BTreeMap::new(),
        }
    }

    pub fn get_region(&self, key: &Vec<usize>) -> Option<&Region> {
        self._regions.get(key)
    }

    pub fn regions(&self) -> &BTreeMap<Vec<usize>, Region> {
        &self._regions
    }
}
