#[derive(Debug, Clone, Copy)]
pub struct RegionKey {
    key: usize,               // key that uniquely maps to a full key
    adjacent_zone_key: usize, // key that uniquely identifies a connected region in n-dimensional space
}
