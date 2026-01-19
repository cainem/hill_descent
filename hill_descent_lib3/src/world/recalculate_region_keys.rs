//! Region key recalculation after dimension subdivision.
//!
//! This module provides a lightweight mechanism to recalculate organism region keys
//! after a dimension has been subdivided, without incrementing organism ages or
//! re-evaluating fitness scores.

use std::sync::Arc;

use rayon::prelude::*;

use super::World;
use crate::world::regions::{OrganismEntry, RegionKey};

impl World {
    /// Recalculates region keys for all organisms after a dimension has been subdivided.
    ///
    /// This is called after `adjust_regions` expands a dimension. Unlike `process_epoch_all`,
    /// this function does NOT increment organism ages or re-evaluate fitness. It only:
    /// 1. Updates each organism's dimensions to the new subdivided dimensions
    /// 2. Recalculates the region key for the affected dimension
    /// 3. Repopulates the regions collection with updated keys
    ///
    /// # Arguments
    ///
    /// * `dimension_index` - The index of the dimension that was subdivided
    pub fn recalculate_region_keys_for_dimension(&mut self, dimension_index: usize) {
        let new_dimensions = Arc::clone(&self.dimensions);

        // Parallel recalculation of region keys
        let entries: Vec<(RegionKey, OrganismEntry)> = self
            .organisms
            .par_iter()
            .filter_map(|(_, org_lock)| {
                let mut org = org_lock.write().unwrap();

                // Update organism's dimensions to the new subdivided dimensions
                org.set_dimensions(Arc::clone(&new_dimensions));

                // Get the organism's current region key and update the subdivided dimension
                if let Some(mut region_key) = org.region_key().cloned() {
                    // Calculate new interval for the subdivided dimension
                    let expressed = org.phenotype().expression_problem_values();
                    if dimension_index < expressed.len() {
                        let value = expressed[dimension_index];
                        let dimension = new_dimensions.get_dimension(dimension_index);

                        if let Some(interval) = dimension.get_interval(value) {
                            // Update the region key with the new interval
                            region_key.update_position(dimension_index, interval);

                            // Update organism's cached region key
                            org.set_region_key(Some(region_key.clone()));

                            // Create entry for region population
                            let entry = OrganismEntry::new(org.id(), org.age(), org.score());

                            return Some((region_key, entry));
                        }
                    }
                }
                None
            })
            .collect();

        // Repopulate regions with updated keys
        self.regions.populate(entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::GlobalConstants;
    use crate::world::single_valued_function::SingleValuedFunction;
    use crate::{TrainingData, setup_world};
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SimpleFunction;

    impl SingleValuedFunction for SimpleFunction {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_with_organisms_when_recalculate_then_region_keys_updated() {
        let param_range = vec![
            RangeInclusive::new(-5.0, 5.0),
            RangeInclusive::new(-5.0, 5.0),
        ];
        let global_constants = GlobalConstants::new(100, 10);
        let mut world = setup_world(&param_range, global_constants, Box::new(SimpleFunction));

        // Run one epoch to populate organisms
        world.training_run(TrainingData::None { floor_value: 0.0 });

        // Record initial region count
        let initial_region_count = world.regions.len();

        // Manually subdivide dimension 0
        let mut new_dims = (*world.dimensions).clone();
        let success = new_dims.divide_dimension(0);
        assert!(success, "Division should succeed");
        world.dimensions = Arc::new(new_dims);

        // Recalculate region keys
        world.recalculate_region_keys_for_dimension(0);

        // After subdivision, we should have more regions (approximately double for one dimension)
        assert!(
            world.regions.len() > initial_region_count,
            "Should have more regions after subdivision: {} vs {}",
            world.regions.len(),
            initial_region_count
        );
    }

    #[test]
    fn given_world_when_recalculate_then_organism_count_unchanged() {
        let param_range = vec![
            RangeInclusive::new(-5.0, 5.0),
            RangeInclusive::new(-5.0, 5.0),
        ];
        let global_constants = GlobalConstants::new(100, 10);
        let mut world = setup_world(&param_range, global_constants, Box::new(SimpleFunction));

        // Run one epoch
        world.training_run(TrainingData::None { floor_value: 0.0 });

        let org_count_before = world.organism_count();

        // Subdivide and recalculate
        let mut new_dims = (*world.dimensions).clone();
        new_dims.divide_dimension(0);
        world.dimensions = Arc::new(new_dims);
        world.recalculate_region_keys_for_dimension(0);

        // Organism count should be unchanged (no aging/death)
        assert_eq!(
            world.organism_count(),
            org_count_before,
            "Organism count should not change during region key recalculation"
        );
    }
}
