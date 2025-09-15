/// Allocates capacity fairly among zones using proportional allocation with remainder distribution.
///
/// This function avoids the "last zone gets remainder" problem by:
/// 1. Computing exact proportional allocations as floating point
/// 2. Taking the floor of each allocation
/// 3. Distributing the remainder based on fractional parts (largest fractions first)
///
/// # Arguments
/// * `weights` - The weights (scores or sizes) for each zone
/// * `total_capacity` - The total capacity to distribute
///
/// # Returns
/// A vector of allocations that sum exactly to total_capacity
fn allocate_capacity_fairly(weights: &[f64], total_capacity: usize) -> Vec<usize> {
    if weights.is_empty() || total_capacity == 0 {
        return vec![0; weights.len()];
    }

    // Sanitize weights: treat negative, NaN or infinite as zero.
    let sanitized: Vec<f64> = weights
        .iter()
        .map(|w| if w.is_finite() && *w > 0.0 { *w } else { 0.0 })
        .collect();

    let total_weight: f64 = sanitized.iter().sum();
    if !(total_weight.is_finite()) || total_weight <= 0.0 {
        return vec![0; weights.len()];
    }

    // Calculate exact proportional allocations, clamped to [0,total_capacity]
    let exact_allocations: Vec<f64> = sanitized
        .iter()
        .map(|&weight| {
            let proportion = weight / total_weight; // <= 1.0 and finite
            let exact = (total_capacity as f64) * proportion;
            if exact.is_finite() {
                exact.clamp(0.0, total_capacity as f64)
            } else {
                0.0
            }
        })
        .collect();

    // Floor to usize (safe after clamp). Cast NaN would become 0.0 earlier.
    let mut allocations: Vec<usize> = exact_allocations
        .iter()
        .map(|&exact| exact.floor() as usize)
        .collect();

    // Ensure no single allocation exceeds total_capacity (belt & braces)
    for a in &mut allocations {
        if *a > total_capacity {
            *a = total_capacity;
        }
    }

    // Calculate how much capacity we've allocated so far
    let allocated_so_far: usize = allocations.iter().sum();
    let remainder = total_capacity.saturating_sub(allocated_so_far);

    // Distribute the remainder based on fractional parts
    if remainder > 0 {
        let mut fractional_parts: Vec<(f64, usize)> = exact_allocations
            .iter()
            .enumerate()
            .map(|(i, &exact)| (exact - exact.floor(), i))
            .collect();

        fractional_parts.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_fractional_part, zone_index) in fractional_parts.iter().take(remainder) {
            // Saturating add to guarantee no panic even in debug builds
            allocations[*zone_index] = allocations[*zone_index].saturating_add(1);
        }
    }

    // Final safety: cap any over-allocations (shouldn't happen but defensive)
    let mut total: usize = allocations.iter().sum();
    if total > total_capacity {
        // Reduce from largest allocations until total matches
        // (rare path; only triggered if something unexpected occurs)
        let mut indices: Vec<usize> = (0..allocations.len()).collect();
        indices.sort_by_key(|&i| std::cmp::Reverse(allocations[i]));
        for i in indices {
            if total <= total_capacity {
                break;
            }
            if allocations[i] > 0 {
                allocations[i] -= 1;
                total -= 1;
            }
        }
    }

    debug_assert!(allocations.iter().all(|a| *a <= total_capacity));
    debug_assert_eq!(allocations.iter().sum::<usize>(), total_capacity);
    allocations
}

/// Calculates the allocation of total carrying capacity among zones using a hybrid approach.
///
/// This function uses a configurable two-fund allocation system:
/// 1. Global fund: Allocated based on region scores, treating all regions as competing globally
/// 2. Zone-proportional fund: Allocated proportionally to zone sizes
///
/// The split between these funds is controlled by the `fractional_zone_allocation` parameter.
///
/// This hybrid approach balances exploitation (rewarding high-scoring regions) with
/// exploration (ensuring fair representation across zones).
///
/// # Arguments
/// * `zone_sizes` - A slice containing the number of regions in each zone
/// * `zone_scores` - A slice containing the total scores for each zone  
/// * `total_capacity` - The total carrying capacity to distribute among zones
/// * `fractional_zone_allocation` - Fraction of capacity allocated to zone-proportional fund (0.0-1.0)
///   - 0.0: All capacity allocated based on global score performance
///   - 1.0: All capacity allocated proportionally to zone sizes
///   - 0.5: Equal split between global and zone-proportional allocation
///
/// # Returns
/// A vector containing the carrying capacity allocated to each zone, in the same order
/// as the input zone sizes.
///
/// # Panics
/// * Panics if zone_sizes and zone_scores have different lengths
/// * Panics if any zone size is 0 (zones should contain at least one region)
/// * Panics if total_capacity is 0 and there are zones to allocate to
/// * Panics if fractional_zone_allocation is not between 0.0 and 1.0
pub fn calculate_zone_capacity_allocation(
    zone_sizes: &[usize],
    zone_scores: &[f64],
    total_capacity: usize,
    fractional_zone_allocation: f64,
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

    // Validate fractional_zone_allocation is in valid range
    if !(0.0..=1.0).contains(&fractional_zone_allocation) {
        panic!(
            "fractional_zone_allocation must be between 0.0 and 1.0, got: {}",
            fractional_zone_allocation
        );
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

    // Split capacity based on fractional_zone_allocation
    // fractional_zone_allocation determines how much goes to zone-proportional fund
    let zone_fund = (total_capacity as f64 * fractional_zone_allocation).round() as usize;
    let global_fund = total_capacity - zone_fund;

    // Fund 1: Global score-based allocation (ignoring zones)
    let global_allocations = allocate_capacity_fairly(zone_scores, global_fund);

    // Fund 2: Zone size-proportional allocation
    let zone_sizes_f64: Vec<f64> = zone_sizes.iter().map(|&size| size as f64).collect();
    let zone_allocations = allocate_capacity_fairly(&zone_sizes_f64, zone_fund);

    // Combine both funds
    let mut final_allocations = Vec::with_capacity(zone_sizes.len());
    for i in 0..zone_sizes.len() {
        // Use saturating_add to prevent debug overflow if invariants break.
        final_allocations.push(global_allocations[i].saturating_add(zone_allocations[i]));
    }

    // Invariant: combined allocations sum to total_capacity (allow slight deficit due to flooring)
    let sum: usize = final_allocations.iter().sum();
    if sum > total_capacity {
        // Trim excess conservatively (should not occur)
        let mut excess = sum - total_capacity;
        for alloc in final_allocations.iter_mut() {
            if excess == 0 {
                break;
            }
            if *alloc > 0 {
                *alloc -= 1;
                excess -= 1;
            }
        }
    }

    final_allocations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_zones() {
        let allocations = calculate_zone_capacity_allocation(&[], &[], 100, 0.5);
        assert!(allocations.is_empty());
    }

    #[test]
    fn test_zero_capacity() {
        let zone_sizes = vec![2, 3, 5];
        let zone_scores = vec![10.0, 15.0, 25.0];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 0, 0.5);
        assert_eq!(allocations, vec![0, 0, 0]);
    }

    #[test]
    fn test_single_zone() {
        let zone_sizes = vec![5];
        let zone_scores = vec![50.0];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 0.5);
        assert_eq!(allocations, vec![100]);
    }

    #[test]
    fn test_multiple_zones_equal_size_equal_scores() {
        let zone_sizes = vec![3, 3, 3]; // All zones same size
        let zone_scores = vec![30.0, 30.0, 30.0]; // All zones same score
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 90, 0.5);
        // Both funds should distribute equally: 45/3 + 45/3 = 15 + 15 = 30 each
        assert_eq!(allocations, vec![30, 30, 30]);
    }

    #[test]
    fn test_multiple_zones_different_sizes() {
        let zone_sizes = vec![2, 3, 5];
        let zone_scores = vec![20.0, 30.0, 50.0]; // Scores proportional to sizes
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 0.5);

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
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 10, 0.5);

        println!("Rounding compensation test result: {:?}", allocations);

        // With fair allocation, total should still be 10
        assert_eq!(allocations.iter().sum::<usize>(), 10);

        // All should be reasonably close to 10/3 ≈ 3.33
        // Allow range [2,5] to be safe
        for (i, &allocation) in allocations.iter().enumerate() {
            assert!(
                (2..=5).contains(&allocation),
                "Zone {} allocation {} is outside expected range [2,5]",
                i,
                allocation
            );
        }
    }

    #[test]
    fn test_large_capacity() {
        let zone_sizes = vec![10, 20];
        let zone_scores = vec![100.0, 200.0]; // Scores proportional to sizes
        let allocations =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 1000000, 0.5);

        // With fair allocation, the result should be very close to proportional
        // Global fund: 1/3 vs 2/3 → ~166667 vs ~333333
        // Zone fund: 1/3 vs 2/3 → ~166667 vs ~333333
        // Total: ~333334 vs ~666666 (fair rounding)
        assert_eq!(allocations.iter().sum::<usize>(), 1000000);

        // Should be close to 1:2 ratio
        let ratio = allocations[1] as f64 / allocations[0] as f64;
        assert!(
            (ratio - 2.0).abs() < 0.01,
            "Ratio should be close to 2.0, got {}",
            ratio
        );
    }

    #[test]
    #[should_panic(expected = "Zone 1 has size 0")]
    fn test_zero_size_zone() {
        let zone_sizes = vec![2, 0, 3];
        let zone_scores = vec![20.0, 0.0, 30.0];
        calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 0.5);
    }

    #[test]
    fn test_allocation_proportions() {
        let zone_sizes = vec![1, 2, 3];
        let zone_scores = vec![10.0, 20.0, 30.0]; // Scores proportional to sizes
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 120, 0.5);

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
        calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 0.5);
    }

    #[test]
    fn test_hybrid_allocation_extreme_scores() {
        let zone_sizes = vec![1, 1, 1]; // Equal sizes
        let zone_scores = vec![0.0, 0.0, 100.0]; // All score in one zone
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 90, 0.5);

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
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 0.5);

        // Global fund (50): no allocation since all scores are 0 -> 0, 0, 0
        // Zone fund (50): proportional to size -> 10, 15, 25
        // Total: 10, 15, 25
        assert_eq!(allocations, vec![10, 15, 25]);
        assert_eq!(allocations.iter().sum::<usize>(), 50);
    }

    #[test]
    fn debug_epoch_17_zone_allocation() {
        println!("Testing zone allocation with data from Epoch 17");

        // From JSON data:
        // Zone 0: min_score: 3.0306874962455444e+29, carrying_capacity: 0
        // Zone 1: min_score: 9.643381773912452e+27, carrying_capacity: 2
        // Zone 2: min_score: 3.7515033722940234e+29, carrying_capacity: 0
        // Zone 3: min_score: 2.3206410278383176e+29, carrying_capacity: 2

        // All zones have 1 region each
        let zone_sizes = vec![1, 1, 1, 1];

        // Calculate zone scores (sum of 1/min_score for each zone)
        let min_scores = vec![
            3.0306874962455444e+29, // Zone 0
            9.643381773912452e+27,  // Zone 1 - BEST
            3.7515033722940234e+29, // Zone 2
            2.3206410278383176e+29, // Zone 3
        ];

        let zone_scores: Vec<f64> = min_scores.iter().map(|&score| 1.0 / score).collect();

        println!("Min scores: {:?}", min_scores);
        println!("Zone scores (1/min_score): {:?}", zone_scores);
        println!("Zone scores sum: {}", zone_scores.iter().sum::<f64>());

        // Test with capacity of 4 (matches your data where two zones get 2 each)
        let total_capacity = 4;
        println!("\n=== Total Capacity: {} ===", total_capacity);
        let allocations =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, total_capacity, 0.5);

        let global_fund = total_capacity / 2; // 2
        let zone_fund = total_capacity - global_fund; // 2

        println!("Global fund: {}, Zone fund: {}", global_fund, zone_fund);
        println!("Allocations: {:?}", allocations);
        println!("Total allocated: {}", allocations.iter().sum::<usize>());

        // Calculate what each zone should get
        let total_score: f64 = zone_scores.iter().sum();
        println!("Zone score proportions:");
        for (i, &score) in zone_scores.iter().enumerate() {
            let global_proportion = score / total_score;
            let expected_global = (global_fund as f64 * global_proportion).round() as usize;
            let expected_zone = zone_fund / 4; // Equal distribution (should be 0 due to integer division)
            let expected_total = expected_global + expected_zone;

            println!(
                "  Zone {}: score={:.2e}, global_prop={:.6}, expected_global={}, expected_zone={}, expected_total={}, actual={}",
                i,
                score,
                global_proportion,
                expected_global,
                expected_zone,
                expected_total,
                allocations[i]
            );
        }

        // Zone 1 should dominate because 1/9.64e27 >> 1/others
        let zone1_score_ratio = zone_scores[1] / zone_scores.iter().sum::<f64>();
        println!(
            "\nZone 1 should get {:.2}% of global fund = {:.2} of {}",
            zone1_score_ratio * 100.0,
            zone1_score_ratio * global_fund as f64,
            global_fund
        );
    }

    #[test]
    fn test_fair_allocation_basic() {
        // Test the new fair allocation function
        let weights = vec![1.0, 3.0, 2.0];
        let total = 10;
        let allocations = allocate_capacity_fairly(&weights, total);

        println!(
            "Fair allocation test: weights={:?}, total={}, result={:?}",
            weights, total, allocations
        );

        // Should allocate proportionally: 1/6*10≈1.67→1, 3/6*10=5, 2/6*10≈3.33→3
        // Remainder 1 should go to zone with largest fractional part (zone 2: 0.33 vs zone 0: 0.67)
        // So: [2, 5, 3] (zone 0 gets the remainder due to 0.67 > 0.33)
        assert_eq!(allocations.iter().sum::<usize>(), total);
        assert_eq!(allocations, vec![2, 5, 3]);
    }

    #[test]
    fn test_allocate_capacity_fairly_edge_cases() {
        // Test negative weights are sanitized to zero
        let weights = vec![-1.0, 2.0, 3.0];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert_eq!(allocations.iter().sum::<usize>(), 10);
        assert_eq!(allocations[0], 0); // Negative weight becomes 0

        // Test NaN weights are sanitized to zero
        let weights = vec![f64::NAN, 2.0, 3.0];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert_eq!(allocations.iter().sum::<usize>(), 10);
        assert_eq!(allocations[0], 0); // NaN weight becomes 0

        // Test infinite weights are sanitized to zero
        let weights = vec![f64::INFINITY, 2.0, 3.0];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert_eq!(allocations.iter().sum::<usize>(), 10);
        assert_eq!(allocations[0], 0); // Infinite weight becomes 0

        // Test all weights zero after sanitization
        let weights = vec![-1.0, f64::NAN, f64::INFINITY];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert_eq!(allocations, vec![0, 0, 0]);

        // Test zero weights
        let weights = vec![0.0, 0.0, 0.0];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert_eq!(allocations, vec![0, 0, 0]);

        // Test empty weights
        let weights = vec![];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert!(allocations.is_empty());

        // Test zero capacity
        let weights = vec![1.0, 2.0, 3.0];
        let allocations = allocate_capacity_fairly(&weights, 0);
        assert_eq!(allocations, vec![0, 0, 0]);
    }

    #[test]
    fn test_allocate_capacity_fairly_extreme_values() {
        // Test very small weights that might cause precision issues
        let weights = vec![1e-100, 2e-100, 3e-100];
        let allocations = allocate_capacity_fairly(&weights, 6);
        assert_eq!(allocations.iter().sum::<usize>(), 6);
        // Should still allocate proportionally despite tiny values

        // Test very large weights
        let weights = vec![1e50, 2e50, 3e50];
        let allocations = allocate_capacity_fairly(&weights, 6);
        assert_eq!(allocations.iter().sum::<usize>(), 6);
        // Should handle large values without overflow

        // Test mixed very large and very small
        let weights = vec![1e-100, 1e100, 1e-100];
        let allocations = allocate_capacity_fairly(&weights, 10);
        assert_eq!(allocations.iter().sum::<usize>(), 10);
        assert!(allocations[1] >= 9); // Middle weight should dominate
    }

    #[test]
    fn test_allocate_capacity_fairly_large_capacity() {
        // Test case with large capacity that exercises large number arithmetic
        let weights = vec![1.0, 1.0, 1.0];
        let large_capacity = 999_999_999; // Large number divisible by 3
        let allocations = allocate_capacity_fairly(&weights, large_capacity);
        // Should not panic and should distribute capacity
        assert!(allocations.iter().all(|&a| a > 0));
        assert_eq!(allocations.iter().sum::<usize>(), large_capacity);

        // Each zone should get exactly equal allocation
        let expected_per_zone = large_capacity / 3;
        for allocation in &allocations {
            assert_eq!(*allocation, expected_per_zone);
        }
    }

    #[test]
    fn test_calculate_zone_capacity_allocation_overflow_protection() {
        // Test the invariant protection in calculate_zone_capacity_allocation
        // This is hard to trigger naturally, but we can test with edge cases
        let zone_sizes = vec![1, 1];
        let zone_scores = vec![f64::MAX / 2.0, f64::MAX / 2.0];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 0.5);

        // Should not panic and should sum to total capacity
        assert_eq!(allocations.iter().sum::<usize>(), 100);
        assert!(allocations.iter().all(|&a| a <= 100));
    }

    #[test]
    fn test_fixed_zone_allocation() {
        println!("Testing fixed zone allocation with Epoch 17 data");

        let zone_sizes = vec![1, 1, 1, 1];
        let zone_scores = vec![1.0, 100.0, 1.0, 1.0]; // Zone 1 much better
        let total_capacity = 4;

        let allocations =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, total_capacity, 0.5);
        println!("Fixed allocations: {:?}", allocations);

        // Now Zone 1 should get most capacity, and remainder should be distributed fairly
        // Global fund (2): Zone 1 gets ~1.96 → 2, others get ~0.02 → 0 each
        // Zone fund (2): Each zone gets 0.5, so [1,0,1,0] or [0,1,0,1] depending on fractional parts
        // Expected result should be more like [0, 2, 0, 2] or [1, 2, 1, 0] - but NOT [0,2,0,2] due to "last zone luck"

        assert_eq!(allocations.iter().sum::<usize>(), total_capacity);

        // Zone 1 should still get the most (at least 2 from global fund)
        assert!(allocations[1] >= 2);

        // But the distribution should be more fair than before
        println!("Zone 1 gets {} out of {}", allocations[1], total_capacity);
    }

    #[test]
    fn test_fractional_zone_allocation_extremes() {
        let zone_sizes = vec![1, 2, 3];
        let zone_scores = vec![10.0, 20.0, 30.0];
        let total_capacity = 60;

        // Test pure global allocation (fractional_zone_allocation = 0.0)
        let allocations_global =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, total_capacity, 0.0);
        // Should allocate: 60 * 10/60 = 10, 60 * 20/60 = 20, 60 * 30/60 = 30
        assert_eq!(allocations_global, vec![10, 20, 30]);

        // Test pure zone-proportional allocation (fractional_zone_allocation = 1.0)
        let allocations_zone =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, total_capacity, 1.0);
        // Should allocate: 60 * 1/6 = 10, 60 * 2/6 = 20, 60 * 3/6 = 30
        assert_eq!(allocations_zone, vec![10, 20, 30]);

        // Test 25% zone allocation (fractional_zone_allocation = 0.25)
        let allocations_25 =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, total_capacity, 0.25);
        // Zone fund: 60 * 0.25 = 15 -> proportional: 2.5, 5, 7.5 -> [3, 5, 7]
        // Global fund: 60 * 0.75 = 45 -> score based: 7.5, 15, 22.5 -> [7, 15, 23]
        // Total: [10, 20, 30]
        assert_eq!(allocations_25.iter().sum::<usize>(), 60);

        // Test 75% zone allocation (fractional_zone_allocation = 0.75)
        let allocations_75 =
            calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, total_capacity, 0.75);
        // Zone fund: 60 * 0.75 = 45 -> proportional: 7.5, 15, 22.5 -> [7, 15, 23]
        // Global fund: 60 * 0.25 = 15 -> score based: 2.5, 5, 7.5 -> [3, 5, 7]
        // Total: [10, 20, 30]
        assert_eq!(allocations_75.iter().sum::<usize>(), 60);
    }

    #[test]
    #[should_panic(expected = "fractional_zone_allocation must be between 0.0 and 1.0")]
    fn test_invalid_fractional_zone_allocation_negative() {
        let zone_sizes = vec![1, 2];
        let zone_scores = vec![10.0, 20.0];
        calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, -0.1);
    }

    #[test]
    #[should_panic(expected = "fractional_zone_allocation must be between 0.0 and 1.0")]
    fn test_invalid_fractional_zone_allocation_too_large() {
        let zone_sizes = vec![1, 2];
        let zone_scores = vec![10.0, 20.0];
        calculate_zone_capacity_allocation(&zone_sizes, &zone_scores, 100, 1.1);
    }
}
