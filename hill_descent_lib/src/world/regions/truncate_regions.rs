use super::Regions;

impl Regions {
    /// Truncates regions that exceed their carrying capacity by marking the worst-scoring organisms as dead.
    ///
    /// This function iterates through all regions and checks if the current population exceeds the region's
    /// carrying capacity. For regions that are over capacity, it marks the worst-scoring organisms as dead,
    /// starting from the end of the sorted organism list (which contains the organisms with the worst scores).
    ///
    /// # Preconditions
    /// - Organisms within each region must be sorted by fitness score (best to worst) followed by age (older first).
    ///   This is typically ensured by calling `sort_regions()` before this function.
    /// - Regions should have their carrying capacity set. If a region does not have a carrying capacity set,
    ///   it will be skipped (this commonly happens on the first training iteration).
    ///
    /// # Side Effects
    /// - Organisms that exceed the carrying capacity are marked as dead using `mark_dead()`.
    /// - The actual removal of dead organisms must be done separately by calling `remove_dead()`.
    ///
    /// # Example Usage
    /// ```rust,no_run
    /// // After sorting organisms and before reproduction
    /// // world.regions.sort_regions();       // Ensure proper sorting
    /// // world.regions.truncate_regions();   // Mark excess organisms as dead
    /// // world.remove_dead();                // Actually remove the dead organisms
    /// ```
    pub fn truncate_regions(&mut self) {
        // Check if this is likely the first training iteration by seeing if all regions
        // have carrying capacity of 0, which typically happens when carrying capacities
        // were calculated before organisms were properly scored.
        let all_regions_zero_capacity = self
            .regions
            .iter()
            .all(|(_, region)| region.carrying_capacity().unwrap_or(1) == 0);

        if all_regions_zero_capacity {
            return;
        }

        for (_region_key, region) in self.regions.iter_mut() {
            // Check if carrying capacity has been set - on the first training iteration,
            // it may not be set yet since capacities are calculated in regions.update().
            // In that case, skip truncation for this iteration.
            let Some(carrying_capacity) = region.carrying_capacity() else {
                continue;
            };

            let current_population = region.organism_count();

            // Skip regions that are within capacity
            if current_population <= carrying_capacity {
                continue;
            }

            let excess = current_population - carrying_capacity;

            crate::debug!(
                "Region {:?}: population {} exceeds capacity {}, culling {} worst organisms",
                _region_key,
                current_population,
                carrying_capacity,
                excess
            );

            // Mark the worst organisms as dead (they're at the end of the sorted list)
            let organisms = region.organisms_mut();
            let start_cull_index = organisms.len() - excess;

            for organism in &organisms[start_cull_index..] {
                organism.mark_dead();
            }
        }
    }
}
