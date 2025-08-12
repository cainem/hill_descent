
use crate::world::organisms::Organisms;

impl super::Regions {
    /// Handles the successful update of all organism region keys.
    ///
    /// This function repopulates the regions with the organisms, prunes any
    /// regions that are now empty, and then determines if the simulation
    /// should continue dividing dimensions or stop.
    ///
    /// # Returns
    ///
    /// Returns `true` if the simulation has reached a stable state and should
    /// stop, `false` otherwise.
    pub(super) fn refill(&mut self, organisms: &mut Organisms) {
        crate::trace!("refill: total organisms before: {}", organisms.len());

        // Before adding the current generation of organisms, clear the regions of any
        // organisms from the previous generation. This ensures the region state is
        // always in sync with the master organism list.
        for region in self.regions.values_mut() {
            region.clear_organisms();
        }
        self.add_organisms(organisms);

        crate::trace!(
            "refill: total organisms after: {} (in regions: {})",
            organisms.len(),
            self.regions
                .values()
                .map(|r| r.organism_count())
                .sum::<usize>()
        );

        self.prune_empty_regions();
    }
}
