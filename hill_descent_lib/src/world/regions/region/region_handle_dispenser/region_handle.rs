/// Compact handle representing a region in n-dimensional space.
/// Contains a unique identifier and adjacency zone information for efficient
/// region management and future carrying capacity allocation strategies.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct RegionHandle {
    /// Unique, monotonic identifier for this region instance
    id: usize,
    /// Identifier of the adjacency-connected zone this region belongs to
    adjacent_zone_id: usize,
}

impl RegionHandle {
    #[allow(dead_code)]
    pub fn new(id: usize, adjacent_zone_id: usize) -> Self {
        Self { id, adjacent_zone_id }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> usize {
        self.id
    }

    #[allow(dead_code)]
    pub fn adjacent_zone_id(&self) -> usize {
        self.adjacent_zone_id
    }
}
