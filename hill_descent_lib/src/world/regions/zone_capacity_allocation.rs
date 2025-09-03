/// Calculates the allocation of total carrying capacity among zones using a hybrid approach.
///
/// This function uses a two-fund allocation system:
/// 1. Global fund (50%): Allocated based on region scores, treating all regions as a single zone
/// 2. Zone-proportional fund (50%): Allocated proportionally to zone sizes
///
/// This hybrid approach balances exploitation (rewarding high-scoring regions) with
/// exploration (ensuring fair representation across zones).
///
/// # Arguments
/// * `zone_sizes` - A slice containing the number of regions in each zone
/// * `zone_scores` - A slice containing the total scores for each zone
/// * `total_capacity` - The total carrying capacity to distribute among zones
///
/// # Returns
/// A vector containing the carrying capacity allocated to each zone, in the same order
/// as the input zone sizes.
///
/// # Panics
/// * Panics if zone_sizes and zone_scores have different lengths
/// * Panics if any zone size is 0 (zones should contain at least one region)
/// * Panics if total_capacity is 0 and there are zones to allocate to
pub fn calculate_zone_capacity_allocation(
    zone_sizes: &[usize],
    zone_scores: &[f64],
    total_capacity: usize,
) -> Vec<usize> {
    if zone_sizes.is_empty() {
        return Vec::new();
    }

    if zone_sizes.len() != zone_scores.len() {
        panic!(
            "zone_sizes and zone_scores must have the same length: {} vs {}",
            zone_sizes.len(),
            zone_scores.len()
        );
    }

    if total_capacity == 0 {
        return vec![0; zone_sizes.len()];
    }

    // Validate that all zones have at least one region
    for (i, &size) in zone_sizes.iter().enumerate() {
        if size == 0 {
            panic!(
                "Zone {} has size 0, but zones must contain at least one region",
                i
            );
        }
    }

    // Split capacity into two funds: 50% for global score-based allocation, 50% for zone-proportional
    let global_fund = total_capacity / 2;
    let zone_fund = total_capacity - global_fund; // Handle odd numbers gracefully

    // Fund 1: Global score-based allocation (ignoring zones)
    let total_score: f64 = zone_scores.iter().sum();
    let mut global_allocations = vec![0; zone_sizes.len()];

    if total_score > 0.0 {
        let mut allocated_so_far = 0;
        for (i, &score) in zone_scores.iter().enumerate() {
            let allocation = if i == zone_scores.len() - 1 {
                // For the last zone, allocate remaining capacity to avoid rounding errors
                global_fund.saturating_sub(allocated_so_far)
            } else {
                // Calculate proportional allocation based on score
                ((global_fund as f64 * score) / total_score).round() as usize
            };
            global_allocations[i] = allocation;
            allocated_so_far += allocation;
        }
    }

    // Fund 2: Zone size-proportional allocation
    let total_size: usize = zone_sizes.iter().sum();
    let mut zone_allocations = vec![0; zone_sizes.len()];

    if total_size > 0 {
        let mut allocated_so_far = 0;
        for (i, &size) in zone_sizes.iter().enumerate() {
            let allocation = if i == zone_sizes.len() - 1 {
                // For the last zone, allocate remaining capacity to avoid rounding errors
                zone_fund.saturating_sub(allocated_so_far)
            } else {
                // Calculate proportional allocation based on zone size
                (zone_fund * size) / total_size
            };
            zone_allocations[i] = allocation;
            allocated_so_far += allocation;
        }
    }

    // Combine both funds
    let mut final_allocations = Vec::with_capacity(zone_sizes.len());
    for i in 0..zone_sizes.len() {
        final_allocations.push(global_allocations[i] + zone_allocations[i]);
    }

    final_allocations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_zones() {
        let allocations = calculate_zone_capacity_allocation(&[], &[], 100);
        assert!(allocations.is_empty());
    }

    #[test]
    fn test_zero_capacity() {
        let zone_sizes = vec![2, 3, 5];
        let zone_scores = vec![10.0, 15.0, 25.0];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 0);
        assert_eq!(allocations, vec![0, 0, 0]);
    }

    #[test]
    fn test_single_zone() {
        let zone_sizes = vec![5];
        let zone_scores = vec![50.0];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100);
        assert_eq!(allocations, vec![100]);
    }

    #[test]
    fn test_multiple_zones_equal_size_equal_scores() {
        let zone_sizes = vec![3, 3, 3]; // All zones same size
        let zone_scores = vec![30.0, 30.0, 30.0]; // All zones same score
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 90);
        // Both funds should distribute equally: 45/3 + 45/3 = 15 + 15 = 30 each
        assert_eq!(allocations, vec![30, 30, 30]);
    }

    #[test]
    fn test_multiple_zones_different_sizes() {
        let zone_sizes = vec![2, 3, 5];
        let zone_scores = vec![20.0, 30.0, 50.0]; // Scores proportional to sizes
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100);

        // Global fund (50): scores are 20/100, 30/100, 50/100 -> 10, 15, 25
        // Zone fund (50): sizes are 2/10, 3/10, 5/10 -> 10, 15, 25
        // Total: 20, 30, 50
        assert_eq!(allocations, vec![20, 30, 50]);
        assert_eq!(allocations.iter().sum::<usize>(), 100);
    }

    #[test]
    fn test_rounding_compensation() {
        let zone_sizes = vec![1, 1, 1]; // Three zones of size 1
        let zone_scores = vec![10.0, 10.0, 10.0]; // Equal scores
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 10);

        // Global fund (5): each gets 5/3 ≈ 1.67, rounded to 2, 2, 1
        // Zone fund (5): each gets 5/3 ≈ 1.67, truncated to 1, 1, 3 (last gets remainder)
        // Total: 3, 3, 4
        assert_eq!(allocations, vec![3, 3, 4]);
        assert_eq!(allocations.iter().sum::<usize>(), 10);
    }

    #[test]
    fn test_large_capacity() {
        let zone_sizes = vec![10, 20];
        let zone_scores = vec![100.0, 200.0]; // Scores proportional to sizes
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 1000000);

        // Global fund (500000): 100/(100+200) = 1/3, 200/300 = 2/3 -> 166667, 333333
        // Zone fund (500000): 10/(10+20) = 1/3, 20/30 = 2/3 -> 166666, 333334
        // Total: 333333, 666667
        assert_eq!(allocations, vec![333333, 666667]);
        assert_eq!(allocations.iter().sum::<usize>(), 1000000);
    }

    #[test]
    #[should_panic(expected = "Zone 1 has size 0")]
    fn test_zero_size_zone() {
        let zone_sizes = vec![2, 0, 3];
        let zone_scores = vec![20.0, 0.0, 30.0];
        calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100);
    }

    #[test]
    fn test_allocation_proportions() {
        let zone_sizes = vec![1, 2, 3];
        let zone_scores = vec![10.0, 20.0, 30.0]; // Scores proportional to sizes
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 120);

        // Global fund (60): 10/60, 20/60, 30/60 = 1/6, 1/3, 1/2 -> 10, 20, 30
        // Zone fund (60): 1/6, 2/6, 3/6 = 1/6, 1/3, 1/2 -> 10, 20, 30
        // Total: 20, 40, 60
        assert_eq!(allocations, vec![20, 40, 60]);

        // Verify the total
        let total: usize = allocations.iter().sum();
        assert_eq!(total, 120);
    }

    #[test]
    #[should_panic(expected = "zone_sizes and zone_scores must have the same length")]
    fn test_mismatched_lengths() {
        let zone_sizes = vec![1, 2, 3];
        let zone_scores = vec![10.0, 20.0]; // Different length
        calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100);
    }

    #[test]
    fn test_hybrid_allocation_extreme_scores() {
        let zone_sizes = vec![1, 1, 1]; // Equal sizes
        let zone_scores = vec![0.0, 0.0, 100.0]; // All score in one zone
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 90);

        // Global fund (45): 0%, 0%, 100% -> 0, 0, 45
        // Zone fund (45): 33.33% each -> 15, 15, 15
        // Total: 15, 15, 60
        assert_eq!(allocations, vec![15, 15, 60]);
        assert_eq!(allocations.iter().sum::<usize>(), 90);
    }

    #[test]
    fn test_hybrid_allocation_zero_scores() {
        let zone_sizes = vec![2, 3, 5];
        let zone_scores = vec![0.0, 0.0, 0.0]; // All zones have zero score
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100);

        // Global fund (50): no allocation since all scores are 0 -> 0, 0, 0
        // Zone fund (50): proportional to size -> 10, 15, 25
        // Total: 10, 15, 25
        assert_eq!(allocations, vec![10, 15, 25]);
        assert_eq!(allocations.iter().sum::<usize>(), 50);
    }
}
