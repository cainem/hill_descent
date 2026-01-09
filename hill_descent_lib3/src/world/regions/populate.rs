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
        // Clear existing regions
        self.clear();

        // Add each entry to its region (creating regions as needed)
        for (key, entry) in entries {
            let region = self.get_or_insert(key);
            region.add_organism(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::GlobalConstants;

    fn make_regions() -> Regions {
        Regions::new(&GlobalConstants::new(100, 10))
    }

    #[test]
    fn given_entries_when_populate_then_regions_created() {
        let mut regions = make_regions();
        let key1 = RegionKey::new(vec![0, 0]);
        let key2 = RegionKey::new(vec![1, 0]);

        let entries = vec![
            (key1.clone(), OrganismEntry::new(1, 0, Some(1.0))),
            (key2.clone(), OrganismEntry::new(2, 0, Some(2.0))),
        ];

        regions.populate(entries);

        assert_eq!(regions.len(), 2);
        assert!(regions.get_region(&key1).is_some());
        assert!(regions.get_region(&key2).is_some());
    }

    #[test]
    fn given_entries_with_same_key_when_populate_then_grouped_in_same_region() {
        let mut regions = make_regions();
        let key = RegionKey::new(vec![0, 0]);

        let entries = vec![
            (key.clone(), OrganismEntry::new(1, 0, Some(1.0))),
            (key.clone(), OrganismEntry::new(2, 0, Some(2.0))),
            (key.clone(), OrganismEntry::new(3, 0, Some(3.0))),
        ];

        regions.populate(entries);

        assert_eq!(regions.len(), 1);
        let region = regions.get_region(&key).unwrap();
        assert_eq!(region.organism_count(), 3);
    }

    #[test]
    fn given_existing_regions_when_populate_then_cleared_first() {
        let mut regions = make_regions();

        // First populate
        let key1 = RegionKey::new(vec![0]);
        let entries1 = vec![(key1.clone(), OrganismEntry::new(1, 0, Some(1.0)))];
        regions.populate(entries1);
        assert_eq!(regions.len(), 1);

        // Second populate with different key
        let key2 = RegionKey::new(vec![1]);
        let entries2 = vec![(key2.clone(), OrganismEntry::new(2, 0, Some(2.0)))];
        regions.populate(entries2);

        // Should only have the new region
        assert_eq!(regions.len(), 1);
        assert!(regions.get_region(&key1).is_none());
        assert!(regions.get_region(&key2).is_some());
    }

    #[test]
    fn given_empty_entries_when_populate_then_no_regions() {
        let mut regions = make_regions();

        // First add some data
        let key = RegionKey::new(vec![0]);
        regions.get_or_insert(key);
        assert_eq!(regions.len(), 1);

        // Populate with empty
        regions.populate(vec![]);

        assert_eq!(regions.len(), 0);
    }
}
