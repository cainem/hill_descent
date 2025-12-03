//! Populate regions with organism entries.

use super::{OrganismEntry, RegionKey, Regions};

impl Regions {
    /// Populates regions with organism entries from fitness evaluation responses.
    ///
    /// # Arguments
    ///
    /// * `entries` - Tuples of (region_key, organism_entry)
    ///
    /// # Side Effects
    ///
    /// - Clears existing regions
    /// - Creates regions as needed
    /// - Adds entries to appropriate regions
    pub fn populate(&mut self, entries: Vec<(RegionKey, OrganismEntry)>) {
        todo!("Implement populate")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_entries_when_populate_then_regions_created() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_entries_with_same_key_when_populate_then_grouped_in_same_region() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_existing_regions_when_populate_then_cleared_first() {
        todo!()
    }
}
