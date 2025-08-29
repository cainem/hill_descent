use std::collections::{HashMap, HashSet};

/// Calculator for determining zones of adjacent regions using Union-Find algorithm.
///
/// Zones are collections of regions where all regions in a zone are connected
/// through adjacency (Chebyshev distance of 1). The algorithm finds the minimum
/// number of zones such that no two zones are adjacent to each other.
///
/// This implementation includes spatial indexing optimization that reduces
/// adjacency checking from O(n²) to O(n×k) where k is the maximum number of
/// potential neighbors (3^d for d dimensions).
#[derive(Debug)]
pub struct ZoneCalculator {
    /// Union-Find parent pointers for each region key
    parent: HashMap<Vec<usize>, Vec<usize>>,
    /// Union-Find ranks for path compression optimization
    rank: HashMap<Vec<usize>, usize>,
    /// Spatial index: maps region keys to their potential neighbors
    spatial_index: HashMap<Vec<usize>, Vec<Vec<usize>>>,
}

impl ZoneCalculator {
    /// Creates a new ZoneCalculator instance.
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
            spatial_index: HashMap::new(),
        }
    }

    /// Calculates zones from a collection of region keys.
    ///
    /// # Arguments
    /// * `region_keys` - A slice of region keys to group into zones
    ///
    /// # Returns
    /// A vector of zones, where each zone is a vector of region keys that are
    /// connected through adjacency relationships.
    pub fn calculate_zones(&mut self, region_keys: &[Vec<usize>]) -> Vec<Vec<Vec<usize>>> {
        if region_keys.is_empty() {
            return Vec::new();
        }

        // Initialize Union-Find structure
        self.initialize_union_find(region_keys);

        // Build spatial index for efficient neighbor lookup
        self.build_spatial_index(region_keys);

        // Find all adjacent pairs and union them using spatial indexing
        self.union_adjacent_regions_optimized(region_keys);

        // Group regions by their root parent (zone)
        self.group_regions_by_zone(region_keys)
    }

    /// Initializes the Union-Find structure with each region as its own parent.
    fn initialize_union_find(&mut self, region_keys: &[Vec<usize>]) {
        self.parent.clear();
        self.rank.clear();

        for key in region_keys {
            self.parent.insert(key.clone(), key.clone());
            self.rank.insert(key.clone(), 0);
        }
    }

    /// Builds a spatial index for efficient neighbor lookup.
    /// For each region, precomputes all potential neighbors that could be adjacent.
    fn build_spatial_index(&mut self, region_keys: &[Vec<usize>]) {
        self.spatial_index.clear();

        // Create a set for fast lookup of existing regions
        let region_set: HashSet<Vec<usize>> = region_keys.iter().cloned().collect();

        for region in region_keys {
            let potential_neighbors = Self::generate_potential_neighbors(region);

            // Filter to only include neighbors that actually exist
            let existing_neighbors: Vec<Vec<usize>> = potential_neighbors
                .into_iter()
                .filter(|neighbor| region_set.contains(neighbor) && neighbor != region)
                .collect();

            self.spatial_index
                .insert(region.clone(), existing_neighbors);
        }
    }

    /// Generates all potential neighbors for a given region key.
    /// A potential neighbor differs by at most 1 in each dimension.
    fn generate_potential_neighbors(region_key: &[usize]) -> Vec<Vec<usize>> {
        if region_key.is_empty() {
            return vec![vec![]];
        }

        let mut neighbors = Vec::new();
        let dimensions = region_key.len();

        // Generate all combinations of {-1, 0, +1} for each dimension
        // Total combinations: 3^dimensions
        let total_combinations = 3_usize.pow(dimensions as u32);

        for i in 0..total_combinations {
            let mut neighbor = region_key.to_vec();
            let mut combination_index = i;
            let mut has_change = false;

            for neighbor_coord in neighbor.iter_mut().take(dimensions) {
                let delta = (combination_index % 3) as i32 - 1; // -1, 0, or 1
                combination_index /= 3;

                if delta != 0 {
                    has_change = true;
                    // Apply delta, ensuring we don't underflow
                    if delta == -1 && *neighbor_coord > 0 {
                        *neighbor_coord -= 1;
                    } else if delta == 1 {
                        *neighbor_coord += 1;
                    } else if delta == -1 && *neighbor_coord == 0 {
                        // Skip this combination as it would underflow
                        has_change = false;
                        break;
                    }
                }
            }

            // Only add if there was at least one change (not the original region)
            if has_change {
                neighbors.push(neighbor);
            }
        }

        neighbors
    }

    /// Optimized version of union_adjacent_regions using spatial indexing.
    /// Instead of checking all pairs O(n²), only checks known neighbors O(n×k).
    fn union_adjacent_regions_optimized(&mut self, _region_keys: &[Vec<usize>]) {
        // Clone the spatial index to avoid borrow checker issues
        let spatial_index = self.spatial_index.clone();

        for (region, neighbors) in &spatial_index {
            for neighbor in neighbors {
                // Double-check adjacency (should always be true due to our generation logic)
                if Self::are_adjacent(region, neighbor) {
                    self.union(region, neighbor);
                }
            }
        }
    }

    /// Legacy method for finding and unioning all adjacent regions using brute force.
    /// Kept for comparison and potential fallback.
    #[allow(dead_code)]
    fn union_adjacent_regions(&mut self, region_keys: &[Vec<usize>]) {
        for i in 0..region_keys.len() {
            for j in (i + 1)..region_keys.len() {
                if Self::are_adjacent(&region_keys[i], &region_keys[j]) {
                    self.union(&region_keys[i], &region_keys[j]);
                }
            }
        }
    }

    /// Groups regions by their root parent to form zones.
    fn group_regions_by_zone(&mut self, region_keys: &[Vec<usize>]) -> Vec<Vec<Vec<usize>>> {
        let mut zones: HashMap<Vec<usize>, Vec<Vec<usize>>> = HashMap::new();

        for key in region_keys {
            let root = self.find(key);
            zones.entry(root).or_default().push(key.clone());
        }

        zones.into_values().collect()
    }

    /// Finds the root parent of a region with path compression.
    ///
    /// # Arguments
    /// * `key` - The region key to find the root for
    ///
    /// # Returns
    /// The root parent key of the region's connected component
    fn find(&mut self, key: &[usize]) -> Vec<usize> {
        let key_vec = key.to_vec();

        // If this key is its own parent, it's the root
        if self.parent[&key_vec] == key_vec {
            return key_vec;
        }

        // Path compression: make the parent point directly to the root
        let root = self.find(&self.parent[&key_vec].clone());
        self.parent.insert(key_vec.clone(), root.clone());
        root
    }

    /// Unions two regions by rank to keep the tree balanced.
    ///
    /// # Arguments
    /// * `key1` - First region key
    /// * `key2` - Second region key
    fn union(&mut self, key1: &[usize], key2: &[usize]) {
        let root1 = self.find(key1);
        let root2 = self.find(key2);

        // If already in the same set, no need to union
        if root1 == root2 {
            return;
        }

        let rank1 = self.rank[&root1];
        let rank2 = self.rank[&root2];

        // Union by rank: attach smaller tree under root of larger tree
        if rank1 < rank2 {
            self.parent.insert(root1, root2);
        } else if rank1 > rank2 {
            self.parent.insert(root2, root1);
        } else {
            // Equal ranks: make root2 parent of root1 and increment rank
            self.parent.insert(root1, root2.clone());
            *self.rank.get_mut(&root2).unwrap() += 1;
        }
    }

    /// Checks if two region keys are adjacent using Chebyshev distance.
    ///
    /// Two regions are adjacent if their Chebyshev distance is exactly 1.
    /// Chebyshev distance is the maximum absolute difference across all dimensions.
    ///
    /// # Arguments
    /// * `key1` - First region key
    /// * `key2` - Second region key
    ///
    /// # Returns
    /// `true` if the regions are adjacent, `false` otherwise
    fn are_adjacent(key1: &[usize], key2: &[usize]) -> bool {
        if key1.len() != key2.len() {
            return false;
        }

        let max_diff = key1
            .iter()
            .zip(key2.iter())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);

        max_diff == 1
    }
}

impl Default for ZoneCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_are_adjacent_same_region() {
        assert!(!ZoneCalculator::are_adjacent(&[1, 2], &[1, 2]));
    }

    #[test]
    fn test_are_adjacent_chebyshev_distance_1() {
        // Adjacent in first dimension
        assert!(ZoneCalculator::are_adjacent(&[1, 2], &[2, 2]));
        // Adjacent in second dimension
        assert!(ZoneCalculator::are_adjacent(&[1, 2], &[1, 3]));
        // Adjacent diagonally (max diff = 1)
        assert!(ZoneCalculator::are_adjacent(&[1, 2], &[2, 3]));
    }

    #[test]
    fn test_are_adjacent_chebyshev_distance_greater_than_1() {
        // Distance 2 in first dimension
        assert!(!ZoneCalculator::are_adjacent(&[1, 2], &[3, 2]));
        // Distance 2 diagonally
        assert!(!ZoneCalculator::are_adjacent(&[1, 1], &[3, 3]));
    }

    #[test]
    fn test_are_adjacent_different_dimensions() {
        assert!(!ZoneCalculator::are_adjacent(&[1, 2], &[1]));
        assert!(!ZoneCalculator::are_adjacent(&[1], &[1, 2]));
    }

    #[test]
    fn test_single_region() {
        let mut calculator = ZoneCalculator::new();
        let regions = vec![vec![1, 2]];
        let zones = calculator.calculate_zones(&regions);

        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0], vec![vec![1, 2]]);
    }

    #[test]
    fn test_two_adjacent_regions() {
        let mut calculator = ZoneCalculator::new();
        let regions = vec![vec![1, 2], vec![2, 2]];
        let zones = calculator.calculate_zones(&regions);

        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].len(), 2);
        assert!(zones[0].contains(&vec![1, 2]));
        assert!(zones[0].contains(&vec![2, 2]));
    }

    #[test]
    fn test_two_non_adjacent_regions() {
        let mut calculator = ZoneCalculator::new();
        let regions = vec![vec![1, 1], vec![3, 3]];
        let zones = calculator.calculate_zones(&regions);

        assert_eq!(zones.len(), 2);
        // Each region should be in its own zone
        for zone in &zones {
            assert_eq!(zone.len(), 1);
        }
    }

    #[test]
    fn test_complex_adjacency_chain() {
        let mut calculator = ZoneCalculator::new();
        let regions = vec![
            vec![1, 1], // Connected to [1, 2]
            vec![1, 2], // Connected to [1, 1] and [1, 3]
            vec![1, 3], // Connected to [1, 2]
            vec![5, 5], // Isolated region
        ];
        let zones = calculator.calculate_zones(&regions);

        assert_eq!(zones.len(), 2);

        // Find the zone with 3 regions and the zone with 1 region
        let mut zone_sizes: Vec<usize> = zones.iter().map(|z| z.len()).collect();
        zone_sizes.sort();
        assert_eq!(zone_sizes, vec![1, 3]);
    }

    #[test]
    fn test_empty_regions() {
        let mut calculator = ZoneCalculator::new();
        let regions: Vec<Vec<usize>> = vec![];
        let zones = calculator.calculate_zones(&regions);

        assert!(zones.is_empty());
    }

    #[test]
    fn test_3d_adjacency() {
        let mut calculator = ZoneCalculator::new();
        let regions = vec![
            vec![1, 1, 1],
            vec![1, 1, 2], // Adjacent to [1, 1, 1] - Chebyshev distance = 1
            vec![3, 3, 3], // Not adjacent to either above - Chebyshev distance = 2
        ];
        let zones = calculator.calculate_zones(&regions);

        assert_eq!(zones.len(), 2);

        // The first two should be in one zone, the third in another
        let mut zone_sizes: Vec<usize> = zones.iter().map(|z| z.len()).collect();
        zone_sizes.sort();
        assert_eq!(zone_sizes, vec![1, 2]);
    }

    #[test]
    fn test_generate_potential_neighbors_2d() {
        let region = vec![5, 5];
        let neighbors = ZoneCalculator::generate_potential_neighbors(&region);

        // Should generate 8 neighbors in 2D (3²-1 = 8)
        assert_eq!(neighbors.len(), 8);

        // Check that all expected neighbors are present
        let expected = vec![
            vec![4, 4],
            vec![4, 5],
            vec![4, 6],
            vec![5, 4],
            vec![5, 6],
            vec![6, 4],
            vec![6, 5],
            vec![6, 6],
        ];

        for expected_neighbor in expected {
            assert!(
                neighbors.contains(&expected_neighbor),
                "Missing neighbor: {:?}",
                expected_neighbor
            );
        }
    }

    #[test]
    fn test_generate_potential_neighbors_boundary_case() {
        let region = vec![0, 1];
        let neighbors = ZoneCalculator::generate_potential_neighbors(&region);

        // Should handle boundary case where region[0] = 0
        // Can't generate neighbor with coordinate -1, but should generate others
        assert!(neighbors.len() <= 8); // Might be less due to underflow prevention

        // Should not contain any neighbors with negative coordinates (underflow)
        for neighbor in &neighbors {
            assert!(
                neighbor.iter().all(|&coord| coord < usize::MAX),
                "Neighbor contains underflow: {:?}",
                neighbor
            );
        }
    }

    #[test]
    fn test_spatial_indexing_performance() {
        let mut calculator = ZoneCalculator::new();

        // Create a larger grid to test performance difference
        let mut regions = Vec::new();
        for x in 0..20 {
            for y in 0..20 {
                regions.push(vec![x, y]);
            }
        }

        // This should complete quickly with spatial indexing
        let zones = calculator.calculate_zones(&regions);

        // All regions should be in one big connected zone
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].len(), 400); // 20×20 = 400
    }

    #[test]
    fn test_spatial_index_correctness() {
        let mut calculator = ZoneCalculator::new();
        let regions = vec![
            vec![1, 1],
            vec![1, 2],
            vec![2, 1], // Connected triangle
            vec![5, 5], // Isolated
        ];

        // Test that spatial indexing gives same result as brute force
        let zones_optimized = calculator.calculate_zones(&regions);

        // Reset and use brute force approach for comparison
        calculator.parent.clear();
        calculator.rank.clear();
        calculator.spatial_index.clear();
        calculator.initialize_union_find(&regions);
        calculator.union_adjacent_regions(&regions); // Brute force method
        let zones_brute_force = calculator.group_regions_by_zone(&regions);

        // Results should be identical
        assert_eq!(zones_optimized.len(), zones_brute_force.len());

        // Sort both to ensure consistent comparison
        let mut opt_sizes: Vec<usize> = zones_optimized.iter().map(|z| z.len()).collect();
        let mut brute_sizes: Vec<usize> = zones_brute_force.iter().map(|z| z.len()).collect();
        opt_sizes.sort();
        brute_sizes.sort();

        assert_eq!(opt_sizes, brute_sizes);
    }
}
