/// Cache for zone calculations to avoid expensive recalculation when regions haven't changed structurally.
///
/// The cache tracks a generation number to determine when the cached zones are still valid.
/// This provides significant performance benefits when zones are queried multiple times
/// without structural changes to the regions.
#[derive(Debug, Clone)]
pub struct ZoneCache {
    /// Cached zone calculation results
    cached_zones: Option<Vec<Vec<Vec<usize>>>>,
    /// Generation number of the cached zones
    cache_generation: u64,
    /// Generation number when regions were last structurally modified
    regions_generation: u64,
}

impl ZoneCache {
    /// Creates a new empty zone cache.
    pub fn new() -> Self {
        Self {
            cached_zones: None,
            cache_generation: 0,
            regions_generation: 0,
        }
    }

    /// Checks if the cached zones are still valid for the current regions generation.
    ///
    /// # Arguments
    /// * `current_regions_generation` - The current generation number of the regions structure
    ///
    /// # Returns
    /// `true` if the cache is valid and can be used, `false` if recalculation is needed
    pub fn is_valid(&self, current_regions_generation: u64) -> bool {
        self.cached_zones.is_some() && self.cache_generation == current_regions_generation
    }

    /// Retrieves the cached zones if they are valid for the current regions generation.
    ///
    /// # Arguments
    /// * `current_regions_generation` - The current generation number of the regions structure
    ///
    /// # Returns
    /// `Some(zones)` if the cache is valid, `None` if recalculation is needed
    pub fn get_zones(&self, current_regions_generation: u64) -> Option<&Vec<Vec<Vec<usize>>>> {
        if self.is_valid(current_regions_generation) {
            self.cached_zones.as_ref()
        } else {
            None
        }
    }

    /// Updates the cache with new zone calculation results.
    ///
    /// # Arguments
    /// * `zones` - The newly calculated zones
    /// * `current_regions_generation` - The current generation number of the regions structure
    pub fn update_cache(&mut self, zones: Vec<Vec<Vec<usize>>>, current_regions_generation: u64) {
        self.cached_zones = Some(zones);
        self.cache_generation = current_regions_generation;
        self.regions_generation = current_regions_generation;
    }

    /// Invalidates the cache, forcing recalculation on next access.
    ///
    /// This should be called when regions undergo structural changes such as:
    /// - Region boundaries are redefined
    /// - Dimension splits or adjustments occur
    /// - Coordinate system changes
    pub fn invalidate(&mut self) {
        self.cached_zones = None;
        self.cache_generation = 0;
    }

    /// Marks that regions have undergone structural changes, incrementing the generation counter.
    ///
    /// This automatically invalidates any existing cache since the cache generation
    /// will no longer match the regions generation.
    ///
    /// # Returns
    /// The new regions generation number
    #[allow(dead_code)]
    pub fn mark_regions_changed(&mut self) -> u64 {
        self.regions_generation += 1;
        // Note: We don't explicitly invalidate here since the generation mismatch
        // will automatically make is_valid() return false
        self.regions_generation
    }

    /// Gets the current regions generation number.
    ///
    /// This can be useful for tracking when regions were last modified
    #[allow(dead_code)]
    /// and coordinating with other systems that need to know about structural changes.
    pub fn current_regions_generation(&self) -> u64 {
        self.regions_generation
    }

    /// Returns cache statistics for monitoring and debugging.
    ///
    /// # Returns
    /// A tuple containing (cache_generation, regions_generation, has_cached_zones)
    #[allow(dead_code)]
    pub fn cache_stats(&self) -> (u64, u64, bool) {
        (
            self.cache_generation,
            self.regions_generation,
            self.cached_zones.is_some(),
        )
    }

    /// Clears all cached data and resets generation counters.
    ///
    /// This provides a complete reset of the cache state, useful for
    /// testing or when starting fresh calculations.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cached_zones = None;
        self.cache_generation = 0;
        self.regions_generation = 0;
    }
}

impl Default for ZoneCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_is_empty() {
        let cache = ZoneCache::new();
        assert!(!cache.is_valid(0));
        assert!(cache.get_zones(0).is_none());
        assert_eq!(cache.current_regions_generation(), 0);
    }

    #[test]
    fn test_cache_update_and_retrieval() {
        let mut cache = ZoneCache::new();
        let zones = vec![vec![vec![1, 2]], vec![vec![3, 4]]];
        let generation = 1;

        cache.update_cache(zones.clone(), generation);

        assert!(cache.is_valid(generation));
        assert_eq!(cache.get_zones(generation), Some(&zones));
        assert_eq!(cache.current_regions_generation(), generation);
    }

    #[test]
    fn test_cache_invalidation_by_generation_mismatch() {
        let mut cache = ZoneCache::new();
        let zones = vec![vec![vec![1, 2]]];

        cache.update_cache(zones.clone(), 1);
        assert!(cache.is_valid(1));

        // Different generation should invalidate
        assert!(!cache.is_valid(2));
        assert!(cache.get_zones(2).is_none());
    }

    #[test]
    fn test_explicit_invalidation() {
        let mut cache = ZoneCache::new();
        let zones = vec![vec![vec![1, 2]]];

        cache.update_cache(zones, 1);
        assert!(cache.is_valid(1));

        cache.invalidate();
        assert!(!cache.is_valid(1));
        assert!(cache.get_zones(1).is_none());
    }

    #[test]
    fn test_mark_regions_changed() {
        let mut cache = ZoneCache::new();
        let zones = vec![vec![vec![1, 2]]];

        cache.update_cache(zones, 1);
        assert!(cache.is_valid(1));

        let new_generation = cache.mark_regions_changed();
        assert_eq!(new_generation, 2);
        assert_eq!(cache.current_regions_generation(), 2);

        // Cache should now be invalid for the new generation since cache_generation != regions_generation
        assert!(cache.is_valid(1)); // Still valid for old generation (cache was created for gen 1)
        assert!(!cache.is_valid(2)); // Invalid for new generation (cache generation is 1, regions is 2)
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ZoneCache::new();

        let (cache_gen, regions_gen, has_cache) = cache.cache_stats();
        assert_eq!(cache_gen, 0);
        assert_eq!(regions_gen, 0);
        assert!(!has_cache);

        cache.update_cache(vec![vec![vec![1]]], 5);
        let (cache_gen, regions_gen, has_cache) = cache.cache_stats();
        assert_eq!(cache_gen, 5);
        assert_eq!(regions_gen, 5);
        assert!(has_cache);
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = ZoneCache::new();
        cache.update_cache(vec![vec![vec![1]]], 5);
        cache.mark_regions_changed(); // Make regions_generation = 6

        cache.clear();

        let (cache_gen, regions_gen, has_cache) = cache.cache_stats();
        assert_eq!(cache_gen, 0);
        assert_eq!(regions_gen, 0);
        assert!(!has_cache);
    }

    #[test]
    fn test_multiple_generation_increments() {
        let mut cache = ZoneCache::new();

        assert_eq!(cache.mark_regions_changed(), 1);
        assert_eq!(cache.mark_regions_changed(), 2);
        assert_eq!(cache.mark_regions_changed(), 3);
        assert_eq!(cache.current_regions_generation(), 3);
    }

    #[test]
    fn test_cache_with_empty_zones() {
        let mut cache = ZoneCache::new();
        let empty_zones: Vec<Vec<Vec<usize>>> = vec![];

        cache.update_cache(empty_zones.clone(), 1);
        assert!(cache.is_valid(1));
        assert_eq!(cache.get_zones(1), Some(&empty_zones));
    }
}
