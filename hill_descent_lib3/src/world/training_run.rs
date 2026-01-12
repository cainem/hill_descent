//! Training run - the main optimization loop.

use super::World;
use crate::training_data::TrainingData;

impl World {
    /// Performs a single training run (generation).
    ///
    /// # Returns
    ///
    /// `true` if dimensions were NOT expanded (i.e., at resolution limit),
    /// `false` if dimensions were expanded due to out-of-bounds organisms.
    ///
    /// Note: This matches lib2's return semantics where `true` indicates stability.
    pub fn training_run(&mut self, training_data: TrainingData) -> bool {
        // Get training data index (0 for function optimization)
        let training_data_index = match training_data {
            TrainingData::None { .. } => 0,
            TrainingData::Supervised { .. } => 0, // For now, use index 0
        };

        // Step 1: Combined epoch processing
        // This updates organisms, regions, and returns dead organisms
        let (dimensions_changed, dead_organisms) = self.process_epoch_all(training_data_index);

        // Step 2: Update carrying capacities based on region fitness
        self.regions.update_carrying_capacities();

        // Step 3: Process regions (sort, cull, select reproduction pairs)
        let process_results = self.regions.process_all(self.world_seed);

        // Step 4: Collect organisms to remove (exceeded carrying capacity only)
        let capacity_exceeded: Vec<u64> = process_results
            .iter()
            .flat_map(|result| result.organisms_to_remove.iter().copied())
            .collect();

        // Step 5: Collect all reproduction pairs
        let reproduction_pairs: Vec<(u64, u64)> = process_results
            .into_iter()
            .flat_map(|result| result.reproduction_pairs)
            .collect();

        // Step 6: Remove organisms that exceeded carrying capacity
        // (Done BEFORE reproduction - these organisms cannot participate)
        if !capacity_exceeded.is_empty() {
            self.remove_organisms(&capacity_exceeded);
        }

        // Step 7: Perform reproduction for selected pairs
        // NOTE: Dead-from-age organisms can still participate in reproduction (if not culled for capacity)
        self.perform_reproduction(reproduction_pairs);

        // Step 8: Remove organisms that died from age (AFTER reproduction)
        if !dead_organisms.is_empty() {
            self.remove_organisms(&dead_organisms);
        }

        // Return true if dimensions did NOT change (at resolution limit / stable)
        // This matches lib2's semantics where the return value indicates stability
        !dimensions_changed
    }

    /// Removes organisms by ID from the IndexMap using shift_remove to preserve order.
    fn remove_organisms(&mut self, ids: &[u64]) {
        for id in ids {
            self.organisms.shift_remove(id);
        }
    }
}
