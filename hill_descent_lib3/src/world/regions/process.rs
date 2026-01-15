//! Region processing - sorting, culling, and reproduction selection.
//!
//! Uses "gap-filling" reproduction strategy: only reproduces enough offspring
//! to fill vacancies up to carrying capacity, matching lib1's approach.

use super::{Region, Regions};

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
    /// # Algorithm (Gap-Filling Strategy)
    ///
    /// 1. Sort organisms by score (ascending) and age (older first for ties)
    /// 2. If over carrying capacity, mark excess organisms for removal
    /// 3. Calculate how many offspring are needed to fill gaps to capacity
    /// 4. Select top performers and pair them to produce required offspring
    ///
    /// This "gap-filling" approach only reproduces enough to maintain population
    /// at carrying capacity, rather than reproducing all organisms.
    ///
    /// # Returns
    ///
    /// A `RegionProcessResult` containing organisms to remove and reproduction pairs.
    pub fn process(&mut self, dead_count_in_region: usize) -> RegionProcessResult {
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

        // Step 3: Calculate survivors after truncation
        let survivor_count = capacity.min(organism_count);
        let survivors = &self.organisms()[..survivor_count];

        // Step 4: Calculate gaps to fill (gap-filling strategy)
        // Gaps = organisms removed for capacity + organisms that will die from age
        let removed_for_capacity = organisms_to_remove.len();
        let total_deaths = removed_for_capacity + dead_count_in_region;

        // How many offspring needed to fill back to capacity?
        // Each pair produces 2 offspring, so we need ceil(gaps / 2) pairs
        let offspring_needed = total_deaths.min(capacity); // Can't exceed capacity
        let pairs_needed = offspring_needed.div_ceil(2);

        // Step 5: Select top performers for reproduction and pair them
        let reproduction_pairs = if pairs_needed > 0 && !survivors.is_empty() {
            Self::pair_for_reproduction_gap_filling(survivors, pairs_needed)
        } else {
            Vec::new()
        };

        RegionProcessResult {
            organisms_to_remove,
            reproduction_pairs,
        }
    }

    /// Pairs organisms for reproduction using extreme pairing strategy (gap-filling version).
    ///
    /// Selects the top `pairs_needed` organisms and pairs them using extreme pairing.
    /// For odd selection counts, duplicates the top performer to create even pairing.
    ///
    /// # Arguments
    ///
    /// * `organisms` - Slice of organism entries, assumed already sorted by fitness
    /// * `pairs_needed` - Number of pairs to create
    ///
    /// # Returns
    ///
    /// Vector of (parent1_id, parent2_id) pairs for reproduction.
    fn pair_for_reproduction_gap_filling(
        organisms: &[super::OrganismEntry],
        pairs_needed: usize,
    ) -> Vec<(u64, u64)> {
        if organisms.is_empty() || pairs_needed == 0 {
            return Vec::new();
        }

        // Select top performers for reproduction (at most 2 * pairs_needed organisms)
        // Each pair needs 2 organisms, but with extreme pairing we may need fewer
        let parents_required = (pairs_needed * 2).min(organisms.len());
        let selected = &organisms[..parents_required];

        if selected.len() == 1 {
            // Single organism pairs with itself
            let id = selected[0].id();
            return vec![(id, id)];
        }

        let mut pairs = Vec::with_capacity(pairs_needed);

        if selected.len() % 2 == 1 {
            // Odd count: duplicate the top performer, then use extreme pairing
            // Create working list: [top, original_0, original_1, ..., original_n]
            let mut working_ids: Vec<u64> = Vec::with_capacity(selected.len() + 1);
            working_ids.push(selected[0].id()); // First copy of top performer
            working_ids.extend(selected.iter().map(|e| e.id())); // Original list

            // Pair using extreme strategy: first with last, second with second-to-last
            let len = working_ids.len();
            for i in 0..(len / 2) {
                if pairs.len() >= pairs_needed {
                    break;
                }
                pairs.push((working_ids[i], working_ids[len - 1 - i]));
            }
        } else {
            // Even count: standard extreme pairing
            let len = selected.len();
            for i in 0..(len / 2) {
                if pairs.len() >= pairs_needed {
                    break;
                }
                let first_id = selected[i].id();
                let last_id = selected[len - 1 - i].id();
                pairs.push((first_id, last_id));
            }
        }

        pairs
    }

    /// Legacy pairing function - pairs ALL organisms (generational strategy).
    /// Kept for reference; not used in gap-filling approach.
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
            // Odd count: duplicate the top performer, then use extreme pairing
            // Create working list: [top, original_0, original_1, ..., original_n]
            let mut working_ids: Vec<u64> = Vec::with_capacity(organisms.len() + 1);
            working_ids.push(organisms[0].id()); // First copy of top performer
            working_ids.extend(organisms.iter().map(|e| e.id())); // Original list

            // Pair using extreme strategy: first with last, second with second-to-last
            let len = working_ids.len();
            for i in 0..(len / 2) {
                pairs.push((working_ids[i], working_ids[len - 1 - i]));
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
    /// 3. Calculate gaps to fill (gap-filling strategy)
    /// 4. Determine reproduction pairs for gap-filling
    ///
    /// # Arguments
    ///
    /// * `_region_seed` - Base seed for deterministic reproduction (reserved for future use)
    /// * `dead_per_region` - Map of region key to number of organisms that will die from age
    ///
    /// # Returns
    ///
    /// Combined results from all regions.
    pub fn process_all(
        &mut self,
        _region_seed: u64,
        dead_per_region: &std::collections::HashMap<super::RegionKey, usize>,
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
    fn given_region_under_capacity_when_process_with_no_deaths_then_no_reproduction() {
        let mut region = Region::new();
        region.set_carrying_capacity(10);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0)));

        // No deaths = no gaps to fill
        let result = region.process(0);

        assert!(
            result.organisms_to_remove.is_empty(),
            "No organisms should be removed when under capacity"
        );
        assert!(
            result.reproduction_pairs.is_empty(),
            "No reproduction when no gaps to fill"
        );
    }

    #[test]
    fn given_region_under_capacity_when_process_with_deaths_then_reproduction_fills_gaps() {
        let mut region = Region::new();
        region.set_carrying_capacity(10);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0)));
        region.add_organism(OrganismEntry::new(4, 0, Some(4.0)));

        // 2 deaths = 2 offspring needed = 1 pair
        let result = region.process(2);

        assert!(result.organisms_to_remove.is_empty());
        assert_eq!(
            result.reproduction_pairs.len(),
            1,
            "1 pair needed to produce 2 offspring"
        );
    }

    #[test]
    fn given_region_over_capacity_when_process_then_excess_marked_for_removal() {
        let mut region = Region::new();
        region.set_carrying_capacity(2);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0))); // Best - keep
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0))); // Second - keep
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0))); // Worst - remove

        let result = region.process(0);

        assert_eq!(result.organisms_to_remove.len(), 1);
        assert!(result.organisms_to_remove.contains(&3));
    }

    #[test]
    fn given_region_over_capacity_when_process_then_reproduction_fills_gaps_from_removal() {
        let mut region = Region::new();
        region.set_carrying_capacity(2);
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.add_organism(OrganismEntry::new(3, 0, Some(3.0))); // Will be removed

        // 1 removed for capacity + 0 deaths = 1 gap = 1 pair (produces 2, one fills gap)
        let result = region.process(0);

        assert_eq!(result.organisms_to_remove.len(), 1);
        assert_eq!(
            result.reproduction_pairs.len(),
            1,
            "1 pair to fill the gap from removed organism"
        );
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
    } // ==================== pair_for_reproduction_gap_filling tests ====================

    #[test]
    fn given_empty_organisms_when_gap_fill_pair_then_returns_empty() {
        let organisms: Vec<OrganismEntry> = vec![];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1);
        assert!(pairs.is_empty());
    }

    #[test]
    fn given_organisms_when_zero_pairs_needed_then_returns_empty() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)),
            OrganismEntry::new(2, 0, Some(2.0)),
        ];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 0);
        assert!(pairs.is_empty());
    }

    #[test]
    fn given_single_organism_when_gap_fill_pair_then_pairs_with_itself() {
        let organisms = vec![OrganismEntry::new(1, 0, Some(1.0))];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 1));
    }

    #[test]
    fn given_two_organisms_when_one_pair_needed_then_paired_together() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)), // Best
            OrganismEntry::new(2, 0, Some(2.0)), // Worst
        ];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 2)); // Best with worst
    }

    #[test]
    fn given_four_organisms_when_two_pairs_needed_then_extreme_pairing_applied() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)), // Best
            OrganismEntry::new(2, 0, Some(2.0)),
            OrganismEntry::new(3, 0, Some(3.0)),
            OrganismEntry::new(4, 0, Some(4.0)), // Worst
        ];
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 2);

        // Extreme pairing: (1,4), (2,3)
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&(1, 4))); // Best with worst
        assert!(pairs.contains(&(2, 3))); // Second with third
    }

    #[test]
    fn given_four_organisms_when_one_pair_needed_then_only_top_two_used() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)), // Best
            OrganismEntry::new(2, 0, Some(2.0)),
            OrganismEntry::new(3, 0, Some(3.0)),
            OrganismEntry::new(4, 0, Some(4.0)), // Worst
        ];
        // Only need 1 pair = 2 offspring = select top 2 organisms
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 1);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (1, 2)); // Top 2 paired together
    }

    #[test]
    fn given_three_organisms_when_two_pairs_needed_then_top_performer_duplicated() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)), // Best
            OrganismEntry::new(2, 0, Some(2.0)),
            OrganismEntry::new(3, 0, Some(3.0)), // Worst
        ];
        // 2 pairs needed = 4 offspring. Select all 3 (odd), duplicate top.
        // Working list: [1, 1, 2, 3]
        // Pairs: (1, 3), (1, 2)
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 2);

        assert_eq!(pairs.len(), 2);
        // With corrected algorithm: working_ids = [1, 1, 2, 3]
        // Pair (working_ids[0], working_ids[3]) = (1, 3)
        // Pair (working_ids[1], working_ids[2]) = (1, 2)
        assert!(pairs.contains(&(1, 3)));
        assert!(pairs.contains(&(1, 2)));
    }

    #[test]
    fn given_five_organisms_when_three_pairs_needed_then_correct_pairs_formed() {
        let organisms = vec![
            OrganismEntry::new(1, 0, Some(1.0)),
            OrganismEntry::new(2, 0, Some(2.0)),
            OrganismEntry::new(3, 0, Some(3.0)),
            OrganismEntry::new(4, 0, Some(4.0)),
            OrganismEntry::new(5, 0, Some(5.0)),
        ];
        // 3 pairs needed = 6 offspring. Select all 5 (odd), duplicate top.
        // Working list: [1, 1, 2, 3, 4, 5]
        // Pairs: (1,5), (1,4), (2,3)
        let pairs = Region::pair_for_reproduction_gap_filling(&organisms, 3);

        assert_eq!(pairs.len(), 3);
        assert!(pairs.contains(&(1, 5)));
        assert!(pairs.contains(&(1, 4)));
        assert!(pairs.contains(&(2, 3)));
    }

    // ==================== Regions::process_all tests ====================

    #[test]
    fn given_multiple_regions_when_process_all_then_all_processed() {
        use crate::parameters::GlobalConstants;
        use crate::world::regions::RegionKey;
        use std::collections::HashMap;

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

        let dead_per_region = HashMap::new(); // No deaths from age
        let results = regions.process_all(12345, &dead_per_region);

        assert_eq!(results.len(), 2);

        // One region should have 1 organism to remove
        let total_removed: usize = results.iter().map(|r| r.organisms_to_remove.len()).sum();
        assert_eq!(total_removed, 1);
    }

    #[test]
    fn given_region_with_deaths_when_process_all_then_reproduction_fills_gaps() {
        use crate::parameters::GlobalConstants;
        use crate::world::regions::RegionKey;
        use std::collections::HashMap;

        let constants = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&constants);

        let key1 = RegionKey::new(vec![0]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.set_carrying_capacity(10);
            region1.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
            region1.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
            region1.add_organism(OrganismEntry::new(3, 0, Some(3.0)));
            region1.add_organism(OrganismEntry::new(4, 0, Some(4.0)));
        }

        // Simulate 2 deaths in region
        let mut dead_per_region = HashMap::new();
        dead_per_region.insert(key1, 2);

        let results = regions.process_all(12345, &dead_per_region);

        assert_eq!(results.len(), 1);
        // 2 deaths = 2 offspring needed = 1 pair
        assert_eq!(results[0].reproduction_pairs.len(), 1);
    }
}
