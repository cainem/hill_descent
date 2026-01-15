//! Region processing - sorting, culling, and reproduction selection.

use std::collections::HashMap;

use super::{Region, RegionKey, Regions};

/// Result of processing a single region.
#[derive(Debug, Clone)]
pub struct RegionProcessResult {
    /// IDs of organisms that should be removed (exceeded capacity)
    pub organisms_to_remove: Vec<u64>,
    /// Reproduction pairs (parent1_id, parent2_id) for this region
    pub reproduction_pairs: Vec<(u64, u64)>,
}

impl Region {
    /// Processes a single region: sort, truncate, and determine reproduction pairs.
    ///
    /// Uses gap-filling strategy: only creates enough offspring to replace dead organisms
    /// and those removed for capacity, rather than pairing all survivors.
    ///
    /// # Arguments
    ///
    /// * `dead_count` - Number of organisms that died from age in this region
    ///
    /// # Algorithm
    ///
    /// 1. Sort organisms by score (ascending) and age (older first for ties)
    /// 2. If over carrying capacity, mark excess organisms for removal
    /// 3. Use gap-filling pairing to create only enough offspring to fill gaps
    ///
    /// # Returns
    ///
    /// A `RegionProcessResult` containing organisms to remove and reproduction pairs.
    pub fn process(&mut self, dead_count: usize) -> RegionProcessResult {
        // Step 1: Sort organisms by score (ascending), age (descending for ties)
        self.organisms_mut().sort();

        let capacity = self.carrying_capacity().unwrap_or(self.organism_count());
        let organism_count = self.organism_count();

        // Step 2: Mark organisms beyond carrying capacity for removal
        let organisms_to_remove: Vec<u64> = if organism_count > capacity {
            self.organisms()[capacity..]
                .iter()
                .map(|entry| entry.id())
                .collect()
        } else {
            Vec::new()
        };

        let removed_for_capacity = organisms_to_remove.len();

        // Step 3: Get remaining organisms for reproduction (up to capacity)
        let survivors = &self.organisms()[..capacity.min(organism_count)];

        // Step 4: Determine reproduction pairs using gap-filling strategy
        // Only create enough pairs to replace dead + capacity-removed organisms
        let reproduction_pairs =
            Self::pair_for_reproduction_gap_filling(survivors, dead_count, removed_for_capacity);

        RegionProcessResult {
            organisms_to_remove,
            reproduction_pairs,
        }
    }

    /// Pairs organisms for reproduction using gap-filling strategy.
    ///
    /// Only creates enough pairs to produce `dead_count + removed_for_capacity` offspring,
    /// rather than pairing all survivors (which would be generational reproduction).
    ///
    /// Uses extreme pairing: best with worst, second-best with second-worst, etc.
    ///
    /// # Arguments
    ///
    /// * `organisms` - Slice of organism entries, assumed already sorted by fitness
    /// * `dead_count` - Number of organisms that died from age
    /// * `removed_for_capacity` - Number removed for exceeding carrying capacity
    ///
    /// # Returns
    ///
    /// Vector of (parent1_id, parent2_id) pairs for reproduction.
    fn pair_for_reproduction_gap_filling(
        organisms: &[super::OrganismEntry],
        dead_count: usize,
        removed_for_capacity: usize,
    ) -> Vec<(u64, u64)> {
        if organisms.is_empty() {
            return Vec::new();
        }

        // Calculate how many offspring we need
        let offspring_needed = dead_count + removed_for_capacity;
        if offspring_needed == 0 {
            return Vec::new();
        }

        // Each pair produces one offspring, so pairs_needed = offspring_needed
        // Use ceiling division to ensure we have enough pairs
        let pairs_needed = offspring_needed;

        // Can't create more pairs than we have organisms / 2 (each organism used once per pair)
        // But we allow an organism to be used in multiple pairs if needed
        let max_pairs = organisms.len(); // Allow up to N pairs from N organisms

        let actual_pairs = pairs_needed.min(max_pairs);

        if actual_pairs == 0 {
            return Vec::new();
        }

        // Build working list for pairing
        // For odd survivor count, duplicate top performer to make even
        let mut working_ids: Vec<u64> = Vec::with_capacity(organisms.len() + 1);

        if organisms.len() % 2 == 1 {
            // Odd: add top performer first, then all originals
            working_ids.push(organisms[0].id());
        }
        working_ids.extend(organisms.iter().map(|o| o.id()));

        // Now do extreme pairing on working_ids until we have enough pairs
        let mut pairs = Vec::with_capacity(actual_pairs);
        let working_len = working_ids.len();

        for i in 0..actual_pairs {
            // Wrap around if we need more pairs than half the working list
            let first_idx = i % (working_len / 2 + working_len % 2);
            let last_idx = working_len - 1 - (i % (working_len / 2 + working_len % 2));

            // Avoid pairing with self if indices collide
            if first_idx >= last_idx && working_len > 1 {
                // Use modular approach for wrap-around
                let first_id = working_ids[i % working_len];
                let last_id = working_ids[(working_len - 1 - (i % working_len)) % working_len];
                if first_id != last_id || working_len == 1 {
                    pairs.push((first_id, last_id));
                } else {
                    // Single organism pairs with itself
                    pairs.push((first_id, first_id));
                }
            } else {
                pairs.push((working_ids[first_idx], working_ids[last_idx]));
            }
        }

        pairs
    }

    /// Legacy pairing function - pairs ALL survivors (generational reproduction).
    /// Kept for reference but no longer used.
    #[allow(dead_code)]
    fn pair_for_reproduction(organisms: &[super::OrganismEntry]) -> Vec<(u64, u64)> {
        if organisms.is_empty() {
            return Vec::new();
        }

        if organisms.len() == 1 {
            // Single organism pairs with itself
            let id = organisms[0].id();
            return vec![(id, id)];
        }

        let mut pairs = Vec::new();

        if organisms.len() % 2 == 1 {
            // Odd count: duplicate the top performer
            // Working list: [top, original_0, original_1, ..., original_n]
            // This gives us an even count to pair
            let top_id = organisms[0].id();

            // Pair top performer with last
            pairs.push((top_id, organisms[organisms.len() - 1].id()));

            // Now pair the original list using extreme pairing
            // This is equivalent to adding top performer at front and using standard pairing
            let original_len = organisms.len();
            for i in 0..(original_len / 2) {
                let first_id = organisms[i].id();
                let last_id = organisms[original_len - 1 - i].id();
                pairs.push((first_id, last_id));
            }
        } else {
            // Even count: standard extreme pairing
            let len = organisms.len();
            for i in 0..(len / 2) {
                let first_id = organisms[i].id();
                let last_id = organisms[len - 1 - i].id();
                pairs.push((first_id, last_id));
            }
        }

        pairs
    }
}

impl Regions {
    /// Processes all regions in parallel using Rayon.
    ///
    /// For each region:
    /// 1. Sort organisms by score (primary), age (secondary)
    /// 2. Mark excess organisms for removal (truncate to capacity)
    /// 3. Determine reproduction pairs using gap-filling strategy
    ///
    /// # Arguments
    ///
    /// * `_region_seed` - Base seed for deterministic reproduction (reserved for future use)
    /// * `dead_per_region` - Map of region key to count of dead organisms in that region
    ///
    /// # Returns
    ///
    /// Combined results from all regions.
    pub fn process_all(
        &mut self,
        _region_seed: u64,
        dead_per_region: &HashMap<RegionKey, usize>,
    ) -> Vec<RegionProcessResult> {
        use rayon::prelude::*;

        // Process regions in parallel and collect results
        self.regions
            .par_iter_mut()
            .map(|(key, region)| {
                let dead_count = dead_per_region.get(key).copied().unwrap_or(0);
                region.process(dead_count)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::regions::OrganismEntry;

    // ==================== Region::process tests ====================

    #[test]
    fn given_region_under_capacity_when_process_with_no_dead_then_no_organisms_removed() {
        let mut region = Region::new();
        region.set_carrying_capacity(10);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0)));

        let result = region.process(0); // No dead organisms

        assert!(
            result.organisms_to_remove.is_empty(),
            "No organisms should be removed when under capacity"
        );
        // With gap-filling and 0 dead, no reproduction pairs
        assert!(
            result.reproduction_pairs.is_empty(),
            "No reproduction needed when no gaps to fill"
        );
    }

    #[test]
    fn given_region_under_capacity_when_process_with_dead_then_pairs_created() {
        let mut region = Region::new();
        region.set_carrying_capacity(10);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0)));

        let result = region.process(2); // 2 dead organisms to replace

        assert!(
            result.organisms_to_remove.is_empty(),
            "No organisms should be removed when under capacity"
        );
        // Need 2 pairs to replace 2 dead
        assert_eq!(result.reproduction_pairs.len(), 2);
    }

    #[test]
    fn given_region_over_capacity_when_process_then_excess_marked_for_removal() {
        let mut region = Region::new();
        region.set_carrying_capacity(2);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0))); // Best - keep
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0))); // Second - keep
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0))); // Worst - remove

        let result = region.process(0); // No dead, but 1 removed for capacity

        assert_eq!(result.organisms_to_remove.len(), 1);
        assert!(result.organisms_to_remove.contains(&3));
        // With 1 removed for capacity and 0 dead, need 1 pair
        assert_eq!(result.reproduction_pairs.len(), 1);
    }

    #[test]
    fn given_region_when_process_then_organisms_sorted_by_score() {
        let mut region = Region::new();
        region.set_carrying_capacity(10);
        // Add in non-sorted order
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0)));
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));

        let _ = region.process(0);

        // After processing, organisms should be sorted
        let organisms = region.organisms();
        assert_eq!(organisms[0].id(), 1); // Score 1.0 (best)
        assert_eq!(organisms[1].id(), 2); // Score 2.0
        assert_eq!(organisms[2].id(), 3); // Score 3.0 (worst)
    }

    #[test]
    fn given_region_with_dead_when_process_then_reproduction_pairs_for_gaps() {
        let mut region = Region::new();
        region.set_carrying_capacity(10);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0)));
        region.add_organism(OrganismEntry::new(4, 0, Some(4.0)));

        // With 2 dead organisms, need 2 pairs
        let result = region.process(2);

        assert_eq!(result.reproduction_pairs.len(), 2);
        // Should use extreme pairing: best with worst, etc.
        assert!(result.reproduction_pairs.contains(&(1, 4)));
        assert!(result.reproduction_pairs.contains(&(2, 3)));
    }

    // ==================== pair_for_reproduction_gap_filling tests ====================

    #[test]
    fn given_empty_organisms_when_gap_fill_then_returns_empty() {
        let organisms: Vec<OrganismEntry> = vec![];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 0, 0);
        assert!(pairs.is_empty());
    }

    #[test]
    fn given_organisms_with_no_gaps_when_gap_fill_then_returns_empty() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)),
            OrganismEntry::new(2, 0, Some(2.0)),
        ];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 0, 0);
        assert!(pairs.is_empty());
    }

    #[test]
    fn given_organisms_with_one_dead_when_gap_fill_then_one_pair() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)),
            OrganismEntry::new(2, 0, Some(2.0)),
        ];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1, 0);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 2)); // Best with worst
    }

    #[test]
    fn given_four_organisms_with_two_gaps_when_gap_fill_then_two_pairs() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)),
            OrganismEntry::new(2, 0, Some(2.0)),
            OrganismEntry::new(3, 0, Some(3.0)),
            OrganismEntry::new(4, 0, Some(4.0)),
        ];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1, 1); // 1 dead + 1 capacity
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&(1, 4)));
        assert!(pairs.contains(&(2, 3)));
    }

    #[test]
    fn given_single_organism_with_gap_when_gap_fill_then_pairs_with_itself() {
        let organisms = vec![OrganismEntry::new(1, 0, Some(1.0))];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1, 0);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 1));
    }

    #[test]
    fn given_more_gaps_than_organisms_when_gap_fill_then_limited_to_organisms() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)),
            OrganismEntry::new(2, 0, Some(2.0)),
        ];
        // Request 10 offspring but only 2 organisms available
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 10, 0);
        // Should create up to 2 pairs (one per organism)
        assert_eq!(pairs.len(), 2);
    }

    // ==================== Regions::process_all tests ====================

    #[test]
    fn given_multiple_regions_when_process_all_then_all_processed() {
        use crate::parameters::GlobalConstants;

        let constants = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&constants);

        // Create two regions with different keys
        let key1 = RegionKey::new(vec![0]);
        let key2 = RegionKey::new(vec![1]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.set_carrying_capacity(2);
            region1.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
            region1.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
            region1.add_organism(OrganismEntry::new(3, 0, Some(3.0))); // Will be removed
        }

        {
            let region2 = regions.get_or_insert(key2.clone());
            region2.set_carrying_capacity(2);
            region2.add_organism(OrganismEntry::new(4, 0, Some(1.0)));
            region2.add_organism(OrganismEntry::new(5, 0, Some(2.0)));
        }

        // Simulate 1 dead in key1
        let mut dead_per_region = HashMap::new();
        dead_per_region.insert(key1.clone(), 1);

        let results = regions.process_all(12345, &dead_per_region);

        assert_eq!(results.len(), 2);

        // One region should have 1 organism to remove
        let total_removed: usize = results.iter().map(|r| r.organisms_to_remove.len()).sum();
        assert_eq!(total_removed, 1);

        // Total reproduction pairs: region1 has 1 dead + 1 capacity = 2 pairs needed
        // region2 has 0 dead + 0 capacity = 0 pairs
        let total_pairs: usize = results.iter().map(|r| r.reproduction_pairs.len()).sum();
        assert_eq!(total_pairs, 2); // Only region1 needs reproduction
    }
}
