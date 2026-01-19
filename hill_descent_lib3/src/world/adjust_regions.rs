//! Region adjustment - dimension subdivision based on organism diversity.
//!
//! This module provides the logic to analyze organism diversity and subdivide
//! dimensions to create more regions, improving search granularity.

use std::sync::Arc;

use super::World;
use crate::NUM_SYSTEM_PARAMETERS;
use crate::world::regions::RegionKey;
use crate::world::regions::calculate_dimension_stats::calculate_dimension_stats;
use crate::world::regions::find_most_diverse_index::find_most_diverse_index;

/// Result of adjusting regions.
#[derive(Debug, PartialEq)]
pub enum AdjustRegionsResult {
    /// A dimension was successfully divided
    DimensionExpanded { dimension_index: usize },
    /// Target number of regions already reached, no expansion needed
    ExpansionNotNecessary,
    /// Cannot expand further due to precision limits or lack of variation
    AtResolutionLimit,
}

impl World {
    /// Adjusts regions by potentially subdividing dimensions.
    ///
    /// This function checks if we need more regions and, if so, identifies the
    /// most diverse dimension in the most populous region and subdivides it.
    ///
    /// # Returns
    ///
    /// * `DimensionExpanded { dimension_index }` - A dimension was successfully divided
    /// * `ExpansionNotNecessary` - Target number of regions already reached
    /// * `AtResolutionLimit` - Cannot expand further due to precision limits or lack of variation
    pub fn adjust_regions(&mut self) -> AdjustRegionsResult {
        // Check if we already have enough regions
        if self.regions.len() >= self.regions.target_regions() {
            return AdjustRegionsResult::ExpansionNotNecessary;
        }

        // Find the most populous region
        let most_populous_key = self.get_most_populous_region_key();
        if most_populous_key.is_none() {
            return AdjustRegionsResult::AtResolutionLimit;
        }
        let most_populous_key = most_populous_key.unwrap();

        // Get organism IDs from the most populous region
        let organism_ids: Vec<u64> = self
            .regions
            .get_region(&most_populous_key)
            .map(|r| r.organisms().iter().map(|e| e.id()).collect())
            .unwrap_or_default();

        // Need at least 2 organisms to measure diversity
        if organism_ids.len() < 2 {
            return AdjustRegionsResult::AtResolutionLimit;
        }

        // Collect expressed problem values for these organisms
        // Organism uses interior mutability so no locks needed
        let expressed_values: Vec<Vec<f64>> = organism_ids
            .iter()
            .filter_map(|id| {
                self.organisms.get(id).map(|org| {
                    let expressed = org.phenotype().expressed_values();
                    if expressed.len() > NUM_SYSTEM_PARAMETERS {
                        expressed[NUM_SYSTEM_PARAMETERS..].to_vec()
                    } else {
                        Vec::new()
                    }
                })
            })
            .collect();

        // Convert to slices for the stats function
        let expressed_refs: Vec<&[f64]> = expressed_values.iter().map(|v| v.as_slice()).collect();

        // Calculate dimension statistics
        let num_dimensions = expressed_refs.first().map_or(0, |v| v.len());
        if num_dimensions == 0 {
            return AdjustRegionsResult::AtResolutionLimit;
        }

        let dimension_stats = calculate_dimension_stats(&expressed_refs, num_dimensions);

        // Find the most diverse dimension
        let most_diverse_index = find_most_diverse_index(dimension_stats);

        if let Some(dim_idx) = most_diverse_index {
            // Try to divide this dimension
            let mut new_dimensions = (*self.dimensions).clone();
            if new_dimensions.divide_dimension(dim_idx) {
                self.dimensions = Arc::new(new_dimensions);
                self.dimension_version += 1;
                AdjustRegionsResult::DimensionExpanded {
                    dimension_index: dim_idx,
                }
            } else {
                // Division failed due to precision loss - try adjust_limits fallback
                // This shrinks the dimension range based on actual organism values,
                // potentially allowing further subdivision after the adjustment.
                if new_dimensions.adjust_limits(dim_idx, self) {
                    self.dimensions = Arc::new(new_dimensions);
                    self.dimension_version += 1;
                    AdjustRegionsResult::DimensionExpanded {
                        dimension_index: dim_idx,
                    }
                } else {
                    AdjustRegionsResult::AtResolutionLimit
                }
            }
        } else {
            // No diversity found - no point continuing
            AdjustRegionsResult::AtResolutionLimit
        }
    }

    /// Returns the key of the most populous region.
    fn get_most_populous_region_key(&self) -> Option<RegionKey> {
        self.regions
            .iter()
            .max_by_key(|(_, region)| region.organism_count())
            .map(|(key, _)| key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::GlobalConstants;
    use crate::world::single_valued_function::SingleValuedFunction;
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SimpleFunction;

    impl SingleValuedFunction for SimpleFunction {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_few_regions_when_adjust_regions_then_dimension_expanded() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        // Set up with target of 10 regions
        let constants = GlobalConstants::new_with_seed(100, 10, 42);

        let mut world = World::new(&bounds, constants, Box::new(SimpleFunction));

        // Run one epoch to populate organisms and regions
        world.process_epoch_all(0);

        // Should have 1 region initially (all organisms in same region with 0 doublings)
        assert_eq!(world.regions.len(), 1);
        assert!(world.regions.len() < world.regions.target_regions());

        // Adjust regions should expand a dimension
        let result = world.adjust_regions();
        assert!(matches!(
            result,
            AdjustRegionsResult::DimensionExpanded { .. }
        ));
    }

    #[test]
    fn given_enough_regions_when_adjust_regions_then_expansion_not_necessary() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
        // Set up with target of 1 region (always satisfied)
        let constants = GlobalConstants::new_with_seed(10, 1, 42);

        let mut world = World::new(&bounds, constants, Box::new(SimpleFunction));
        world.process_epoch_all(0);

        let result = world.adjust_regions();
        assert_eq!(result, AdjustRegionsResult::ExpansionNotNecessary);
    }

    #[test]
    fn given_no_organisms_when_adjust_regions_then_at_resolution_limit() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(10, 10, 42);

        let mut world = World::new(&bounds, constants, Box::new(SimpleFunction));
        // Don't run process_epoch - regions are empty

        let result = world.adjust_regions();
        assert_eq!(result, AdjustRegionsResult::AtResolutionLimit);
    }
}
