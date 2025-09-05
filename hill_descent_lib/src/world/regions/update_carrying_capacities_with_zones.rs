use crate::world::regions::Regions;
use crate::world::regions::zone_calculator::ZoneCalculator;
use crate::world::regions::zone_capacity_allocation::calculate_zone_capacity_allocation;

impl Regions {
    /// Updates the carrying capacity for all regions using zone-based allocation.
    ///
    /// This method implements a hybrid zone-based carrying capacity calculation:
    ///
    /// 1. Calculates zones using adjacency (Chebyshev distance = 1)
    /// 2. Splits carrying capacity between global and zone-proportional funds based on FRACTIONAL_ZONE_ALLOCATION
    /// 3. Global fund: allocated based on zone scores (sum of inverse min_scores)
    /// 4. Zone-proportional fund: allocated proportionally to zone sizes
    /// 5. Within each zone, distributes capacity based on relative min_scores
    ///
    /// The allocation split is controlled by the FRACTIONAL_ZONE_ALLOCATION constant:
    /// - 0.0: All capacity allocated based on global score performance
    /// - 1.0: All capacity allocated proportionally to zone sizes  
    /// - 0.5: Equal split between global and zone-proportional allocation (current default)
    ///
    /// This hybrid approach balances exploitation (rewarding high-scoring regions)
    /// with exploration (ensuring fair representation across zones).
    ///
    /// The zone cache is invalidated at the start of each call (temporary approach).
    ///
    /// # Panics
    /// * Panics if zone calculation fails for any reason
    /// * Panics if any zone has size 0 (should not happen with valid zone calculation)
    /// * Panics if min_score calculations produce infinite or NaN values
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self))
    )]
    pub fn update_carrying_capacities_with_zones(&mut self) {
        // Temporarily invalidate cache at start (as requested)
        // TODO: Implement proper cache invalidation based on structural changes
        self.zone_cache.invalidate();

        // Collect region keys for zone calculation
        let region_keys: Vec<Vec<usize>> = self.regions.keys().cloned().collect();

        if region_keys.is_empty() {
            // No regions to process
            return;
        }

        // Calculate zones using cache if available
        let zones = if let Some(cached_zones) = self.zone_cache.get_zones(1) {
            // Cache hit - use cached zones
            cached_zones.clone()
        } else {
            // Cache miss - calculate zones
            let mut zone_calculator = ZoneCalculator::new();
            let calculated_zones = zone_calculator.calculate_zones(&region_keys);

            // Update cache with calculated zones
            self.zone_cache.update_cache(calculated_zones.clone(), 1);
            calculated_zones
        };

        if zones.is_empty() {
            panic!("Zone calculation returned empty zones for non-empty regions - this is a bug");
        }

        // Create zone mapping for web visualization
        let mut zone_mapping = std::collections::HashMap::new();
        for (zone_idx, zone_regions) in zones.iter().enumerate() {
            for region_key in zone_regions {
                zone_mapping.insert(region_key.clone(), zone_idx);
            }
        }
        self.set_zone_mapping(zone_mapping);

        // Calculate zone sizes and scores
        let zone_sizes: Vec<usize> = zones.iter().map(|zone| zone.len()).collect();

        // Calculate zone scores (sum of inverse min_scores for regions in each zone)
        let zone_scores: Vec<f64> = zones
            .iter()
            .map(|zone_regions| {
                let mut zone_score = 0.0;
                for region_key in zone_regions {
                    if let Some(region) = self.regions.get(region_key)
                        && let Some(min_score) = region.min_score()
                        && min_score > 0.0
                    {
                        // Use inverse fitness as the score (lower min_score = higher attractiveness)
                        zone_score += 1.0 / min_score;
                    }
                }
                zone_score
            })
            .collect();

        // Allocate total capacity among zones using hybrid approach
        let total_capacity = self.population_size;
        let zone_capacities = calculate_zone_capacity_allocation(
            &zone_sizes,
            &zone_scores,
            total_capacity,
            Self::FRACTIONAL_ZONE_ALLOCATION,
        );

        // Distribute capacity within each zone based on min_scores
        for (zone_idx, zone_regions) in zones.iter().enumerate() {
            let zone_capacity = zone_capacities[zone_idx];
            self.distribute_capacity_within_zone(zone_regions, zone_capacity);
        }
    }

    /// Distributes carrying capacity within a single zone based on region min_scores.
    ///
    /// Uses the same inverse fitness formula as the original carrying capacity calculation,
    /// but applied only within the regions of this zone.
    ///
    /// # Arguments
    /// * `zone_regions` - The region keys that belong to this zone
    /// * `zone_capacity` - The total carrying capacity allocated to this zone
    fn distribute_capacity_within_zone(
        &mut self,
        zone_regions: &[Vec<usize>],
        zone_capacity: usize,
    ) {
        if zone_regions.is_empty() {
            return;
        }

        let mut sum_inverse_min_fitness = 0.0;

        // First pass: calculate sum of inverse fitnesses within this zone
        for region_key in zone_regions {
            if let Some(region) = self.regions.get(region_key) {
                if let Some(min_score) = region.min_score()
                    && min_score > 0.0
                {
                    let mut inverse_fitness = 1.0 / min_score;
                    if inverse_fitness.is_infinite() {
                        // Cap infinite values to prevent NaN calculations
                        inverse_fitness = f64::MAX / 10.0;
                    }
                    sum_inverse_min_fitness += inverse_fitness;
                }
            } else {
                panic!(
                    "Zone contains region key {:?} that doesn't exist in regions map - this is a bug",
                    region_key
                );
            }
        }

        // Second pass: allocate capacity within the zone
        for region_key in zone_regions {
            let mut capacity = 0;

            if let Some(region) = self.regions.get_mut(region_key) {
                if sum_inverse_min_fitness > 0.0
                    && let Some(min_score) = region.min_score()
                    && min_score > 0.0
                {
                    let mut inverse_fitness = 1.0 / min_score;
                    if inverse_fitness.is_infinite() {
                        inverse_fitness = f64::MAX / 10.0;
                    }

                    // Calculate proportional capacity within the zone
                    let capacity_float =
                        zone_capacity as f64 * (inverse_fitness / sum_inverse_min_fitness);
                    capacity = capacity_float.floor() as usize;
                }

                region.set_carrying_capacity(Some(capacity));
            } else {
                panic!(
                    "Region key {:?} disappeared between passes - this is a bug",
                    region_key
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::world::regions::Region;

    fn create_test_regions(population_size: usize) -> Regions {
        let global_constants = GlobalConstants::new(population_size, 4);
        Regions::new(&global_constants)
    }

    fn setup_region_with_min_score(min_score: Option<f64>) -> Region {
        let mut region = Region::new();
        region.set_min_score(min_score);
        region
    }

    #[test]
    fn test_empty_regions() {
        let mut regions = create_test_regions(100);

        // Should not panic with empty regions
        regions.update_carrying_capacities_with_zones();
    }

    #[test]
    fn test_single_region() {
        let mut regions = create_test_regions(100);
        let key = vec![1, 2];

        regions.insert_region(key.clone(), setup_region_with_min_score(Some(10.0)));
        regions.update_carrying_capacities_with_zones();

        // Single region should get all capacity
        assert_eq!(
            regions.get_region(&key).unwrap().carrying_capacity(),
            Some(100)
        );
    }

    #[test]
    fn test_multiple_adjacent_regions_single_zone() {
        let mut regions = create_test_regions(100);

        // Create adjacent regions that should form one zone
        let key1 = vec![1, 1];
        let key2 = vec![1, 2]; // Adjacent to key1

        regions.insert_region(key1.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key2.clone(), setup_region_with_min_score(Some(20.0)));

        regions.update_carrying_capacities_with_zones();

        // Both regions are in same zone, so capacity distributed by inverse fitness
        // Inverse fitnesses: 1/10 = 0.1, 1/20 = 0.05, sum = 0.15
        // key1 gets: 100 * (0.1/0.15) = 66.67 -> 66
        // key2 gets: 100 * (0.05/0.15) = 33.33 -> 33

        let cap1 = regions
            .get_region(&key1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2 = regions
            .get_region(&key2)
            .unwrap()
            .carrying_capacity()
            .unwrap();

        assert!(cap1 > cap2); // Better min_score gets more capacity
        assert_eq!(cap1 + cap2, 99); // Total should be close to 100 (rounding)
    }

    #[test]
    fn test_multiple_non_adjacent_regions_separate_zones() {
        let mut regions = create_test_regions(100);

        // Create non-adjacent regions that should form separate zones
        let key1 = vec![1, 1];
        let key2 = vec![5, 5]; // Not adjacent to key1

        regions.insert_region(key1.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key2.clone(), setup_region_with_min_score(Some(10.0)));

        regions.update_carrying_capacities_with_zones();

        // Each region forms its own zone of size 1
        // Zone allocation: each zone gets 1²/(1²+1²) = 1/2 = 50 capacity
        // Within each zone: single region gets all zone capacity

        assert_eq!(
            regions.get_region(&key1).unwrap().carrying_capacity(),
            Some(50)
        );
        assert_eq!(
            regions.get_region(&key2).unwrap().carrying_capacity(),
            Some(50)
        );
    }

    #[test]
    fn test_zones_with_different_sizes() {
        let mut regions = create_test_regions(190); // 190 = 38 * 5 for clean division

        // Create zones of different sizes: zone1 = 2 regions, zone2 = 3 regions
        // Zone 1: [1,1] and [1,2] (adjacent)
        let key1_1 = vec![1, 1];
        let key1_2 = vec![1, 2];

        // Zone 2: [5,5], [5,6], [6,5] (all adjacent to each other)
        let key2_1 = vec![5, 5];
        let key2_2 = vec![5, 6];
        let key2_3 = vec![6, 5];

        regions.insert_region(key1_1.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key1_2.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key2_1.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key2_2.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key2_3.clone(), setup_region_with_min_score(Some(10.0)));

        regions.update_carrying_capacities_with_zones();

        // Zone allocations: 2² = 4, 3² = 9, total = 13
        // Zone 1 gets: 4/13 * 190 = 58.46 -> 58
        // Zone 2 gets: 9/13 * 190 = 131.54 -> 132

        let cap1_1 = regions
            .get_region(&key1_1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap1_2 = regions
            .get_region(&key1_2)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2_1 = regions
            .get_region(&key2_1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2_2 = regions
            .get_region(&key2_2)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2_3 = regions
            .get_region(&key2_3)
            .unwrap()
            .carrying_capacity()
            .unwrap();

        let zone1_total = cap1_1 + cap1_2;
        let zone2_total = cap2_1 + cap2_2 + cap2_3;

        // Zone 2 should get more capacity due to larger size
        assert!(zone2_total > zone1_total);

        // All regions in same zone should get equal capacity (same min_score)
        assert_eq!(cap1_1, cap1_2);
        assert_eq!(cap2_1, cap2_2);
        assert_eq!(cap2_2, cap2_3);
    }

    #[test]
    fn test_regions_with_no_min_score() {
        let mut regions = create_test_regions(100);

        let key1 = vec![1, 1];
        let key2 = vec![1, 2]; // Adjacent

        regions.insert_region(key1.clone(), setup_region_with_min_score(Some(10.0)));
        regions.insert_region(key2.clone(), setup_region_with_min_score(None)); // No min_score

        regions.update_carrying_capacities_with_zones();

        // key1 should get all capacity, key2 should get 0
        assert_eq!(
            regions.get_region(&key1).unwrap().carrying_capacity(),
            Some(100)
        );
        assert_eq!(
            regions.get_region(&key2).unwrap().carrying_capacity(),
            Some(0)
        );
    }
}
