use std::collections::BTreeMap;

use region::Region;

pub mod region;

#[derive(Debug, Clone)]
pub struct Regions {
    _regions: BTreeMap<Vec<usize>, Region>,
}
