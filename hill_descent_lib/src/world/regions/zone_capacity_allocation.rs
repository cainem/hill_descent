/// Calculates the allocation of total carrying capacity among zones based on their sizes.
///
/// According to the zone allocation formula, zones receive carrying capacity proportional
/// to their size (number of regions). This encourages exploration across different zones
/// while providing fair representation to all zones based on their size.
///
/// # Arguments
/// * `zone_sizes` - A slice containing the number of regions in each zone
/// * `total_capacity` - The total carrying capacity to distribute among zones
///
/// # Returns
/// A vector containing the carrying capacity allocated to each zone, in the same order
/// as the input zone sizes.
///
/// # Panics
/// * Panics if any zone size is 0 (zones should contain at least one region)
/// * Panics if total_capacity is 0 and there are zones to allocate to
pub fn calculate_zone_capacity_allocation(
    zone_sizes: &[usize],
    total_capacity: usize,
) -> Vec<usize> {
    if zone_sizes.is_empty() {
        return Vec::new();
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

    // Calculate the sum of all zone sizes (linear allocation)
    let total_size: usize = zone_sizes.iter().sum();

    if total_size == 0 {
        panic!(
            "Total of zone sizes is 0, which should not be possible with non-empty zones"
        );
    }

    // Allocate capacity proportional to zone size
    let mut allocations = Vec::with_capacity(zone_sizes.len());
    let mut allocated_so_far = 0;

    for (i, &size) in zone_sizes.iter().enumerate() {
        let allocation = if i == zone_sizes.len() - 1 {
            // For the last zone, allocate remaining capacity to avoid rounding errors
            total_capacity - allocated_so_far
        } else {
            // Calculate proportional allocation and round down
            (total_capacity * size) / total_size
        };

        allocations.push(allocation);
        allocated_so_far += allocation;
    }

    allocations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_zones() {
        let allocations = calculate_zone_capacity_allocation(&[], 100);
        assert!(allocations.is_empty());
    }

    #[test]
    fn test_zero_capacity() {
        let zone_sizes = vec![2, 3, 5];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 0);
        assert_eq!(allocations, vec![0, 0, 0]);
    }

    #[test]
    fn test_single_zone() {
        let zone_sizes = vec![5];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 100);
        assert_eq!(allocations, vec![100]);
    }

    #[test]
    fn test_multiple_zones_equal_size() {
        let zone_sizes = vec![3, 3, 3]; // All zones same size
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 90);
        // Each gets 3/(3+3+3) = 1/3 of capacity = 30
        assert_eq!(allocations, vec![30, 30, 30]);
    }

    #[test]
    fn test_multiple_zones_different_sizes() {
        let zone_sizes = vec![2, 3, 5];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 100);

        // Linear allocation: 2/(2+3+5) = 2/10 = 20%, 3/10 = 30%, 5/10 = 50%
        // Allocations: 20, 30, 50
        assert_eq!(allocations, vec![20, 30, 50]);
        assert_eq!(allocations.iter().sum::<usize>(), 100);
    }

    #[test]
    fn test_rounding_compensation() {
        let zone_sizes = vec![1, 1, 1]; // Three zones of size 1
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 10);

        // Each should get 1/3 * 10 = 3.33..., but we need integer allocation
        // First two get floor(3.33) = 3, last gets remainder = 4
        assert_eq!(allocations, vec![3, 3, 4]);
        assert_eq!(allocations.iter().sum::<usize>(), 10);
    }

    #[test]
    fn test_large_capacity() {
        let zone_sizes = vec![10, 20];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 1000000);

        // Linear allocation: 10/(10+20) = 1/3, 20/(10+20) = 2/3
        // Allocations: 1/3 * 1000000 = 333333, 2/3 * 1000000 = 666667
        assert_eq!(allocations, vec![333333, 666667]);
        assert_eq!(allocations.iter().sum::<usize>(), 1000000);
    }

    #[test]
    #[should_panic(expected = "Zone 1 has size 0")]
    fn test_zero_size_zone() {
        let zone_sizes = vec![2, 0, 3];
        calculate_zone_capacity_allocation(&zone_sizes, 100);
    }

    #[test]
    fn test_allocation_proportions() {
        let zone_sizes = vec![1, 2, 3];
        let allocations = calculate_zone_capacity_allocation(&zone_sizes, 120);

        // Linear allocation: 1/(1+2+3) = 1/6, 2/6, 3/6
        // Proportions: 1/6, 2/6, 3/6 = 1/6, 1/3, 1/2
        // Allocations: 20, 40, 60
        assert_eq!(allocations, vec![20, 40, 60]);

        // Verify the proportions are correct (within rounding)
        let total: usize = allocations.iter().sum();
        assert_eq!(total, 120);
    }
}
