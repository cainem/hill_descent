//! Training run - the main optimization loop.

use super::World;
use super::adjust_regions::AdjustRegionsResult;
use crate::training_data::TrainingData;

impl World {
    /// Performs a single training run (generation).
    ///
    /// This follows a specific order to maintain consistency between region keys
    /// and death tracking:
    /// 1. Process epoch (age organisms, count deaths per region)
    /// 2. Update carrying capacities based on current region fitness
    /// 3. Process regions (sort, cull, select reproduction pairs)
    /// 4. Remove dead organisms
    /// 5. Perform reproduction
    /// 6. Adjust regions (subdivide dimensions) - AFTER reproduction so region
    ///    keys remain consistent with death tracking throughout the cycle
    ///
    /// # Returns
    ///
    /// Returns `true` if at resolution limit (no further meaningful dimension
    /// subdivisions possible), `false` otherwise.
    ///
    /// The resolution limit is reached when:
    /// - No dimensions can be subdivided due to floating-point precision limits
    /// - There is no diversity in organism values (all organisms have same values)
    /// - Target number of regions has been reached
    pub fn training_run(&mut self, training_data: TrainingData) -> bool {
        // Get training data index (0 for function optimization)
        let training_data_index = match training_data {
            TrainingData::None { .. } => 0,
            TrainingData::Supervised { .. } => 0, // For now, use index 0
        };

        // Step 1: Combined epoch processing
        // This updates organisms, regions, and returns dead organisms with region info
        let (_dimensions_changed, dead_organisms, dead_per_region) =
            self.process_epoch_all(training_data_index);

        // Step 2: Update carrying capacities based on region fitness
        self.regions.update_carrying_capacities();

        // Step 3: Process regions (sort, cull, select reproduction pairs using gap-filling)
        let process_results = self.regions.process_all(self.world_seed, &dead_per_region);

        // Step 4: Collect organisms to remove (exceeded carrying capacity only)
        let capacity_exceeded: Vec<u64> = process_results
            .iter()
            .flat_map(|result| result.organisms_to_remove.iter().copied())
            .collect();

        // Step 5: Collect all reproduction pairs
        // NOTE: We do NOT exclude dead-from-age organisms from reproduction.
        // In lib1's flow, reproduction happens BEFORE aging, so organisms that will
        // die from age still participate in reproduction. We maintain this behavior
        // by allowing dead organisms to be parents - their offspring carry their genes.
        let reproduction_pairs: Vec<(u64, u64)> = process_results
            .into_iter()
            .flat_map(|result| result.reproduction_pairs)
            .collect();

        // Step 6: Remove organisms that exceeded carrying capacity
        // (Done BEFORE reproduction - these organisms cannot participate as they are worst performers)
        if !capacity_exceeded.is_empty() {
            self.remove_organisms(&capacity_exceeded);
        }

        // Step 7: Perform reproduction for selected pairs
        // This happens BEFORE removing dead-from-age organisms, matching lib1's behavior
        // where reproduction occurs before aging. Dead-from-age organisms can still
        // participate in reproduction - they pass on their genes before dying.
        self.perform_reproduction(reproduction_pairs);

        // Step 8: Remove organisms that died from age (AFTER reproduction)
        if !dead_organisms.is_empty() {
            self.remove_organisms(&dead_organisms);
        }

        // Step 9: Adjust regions AFTER reproduction is complete
        // This ensures region keys remain consistent with dead_per_region tracking
        // throughout the reproduction cycle. The new region structure takes effect
        // in the NEXT training run.
        // Return true if at resolution limit (no more useful subdivisions possible)
        self.adjust_regions_loop()
    }

    /// Adjusts regions in a loop - subdivide dimensions until we reach
    /// the target number of regions or hit the resolution limit.
    ///
    /// This is the key mechanism that allows finer-grained search over time.
    ///
    /// # Returns
    ///
    /// Returns `true` if at resolution limit, `false` otherwise.
    fn adjust_regions_loop(&mut self) -> bool {
        loop {
            let adjust_result = self.adjust_regions();

            match adjust_result {
                AdjustRegionsResult::DimensionExpanded { dimension_index } => {
                    // Dimension was expanded - recalculate region keys for all organisms.
                    // IMPORTANT: We don't call process_epoch_all here because that would
                    // increment organism ages multiple times per training_run call.
                    // Instead, we just recalculate region keys and repopulate regions.
                    self.recalculate_region_keys_for_dimension(dimension_index);
                    // Continue loop to potentially expand more dimensions
                    continue;
                }
                AdjustRegionsResult::ExpansionNotNecessary => {
                    // Target regions reached
                    return false;
                }
                AdjustRegionsResult::AtResolutionLimit => {
                    // Can't expand further
                    return true;
                }
            }
        }
    }

    /// Removes organisms by ID from the IndexMap while preserving order.
    /// Uses batch retain for O(N) performance instead of O(M*N).
    fn remove_organisms(&mut self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }

        // For very small numbers of removals, shift_remove is efficient enough.
        // For larger numbers, we use retain with a HashSet to avoid O(N^2) behavior.
        if ids.len() < 10 {
            for id in ids {
                self.organisms.shift_remove(id);
            }
        } else {
            let id_set: std::collections::HashSet<u64> = ids.iter().copied().collect();
            self.organisms.retain(|id, _| !id_set.contains(id));
        }
    }
}
