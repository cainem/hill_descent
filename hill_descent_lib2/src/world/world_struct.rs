//! World struct - the coordinator that orchestrates training runs.

use std::sync::Arc;

use messaging_thread_pool::ThreadPool;

use super::{Dimensions, Regions, WorldFunction};
use crate::{organism::Organism, parameters::GlobalConstants};

/// The World coordinates all operations in the genetic algorithm.
///
/// It owns the thread pool containing organisms and orchestrates training runs
/// via message batches.
pub struct World {
    /// Thread pool containing all organisms
    pub(super) organism_pool: ThreadPool<Organism>,

    /// Spatial dimensions (bounds and intervals)
    pub(super) dimensions: Arc<Dimensions>,

    /// Current dimension version for incremental key updates
    pub(super) dimension_version: u64,

    /// Region management (not in thread pool)
    pub(super) regions: Regions,

    /// Fitness function shared by all organisms
    pub(super) world_function: Arc<dyn WorldFunction>,

    /// Global configuration
    pub(super) global_constants: GlobalConstants,

    /// Best fitness score seen so far
    pub(super) best_score: f64,

    /// ID of the organism with the best score
    pub(super) best_organism_id: Option<u64>,

    /// All organism IDs currently in the pool
    pub(super) organism_ids: Vec<u64>,

    /// Next organism ID to assign
    pub(super) next_organism_id: u64,

    /// World seed for deterministic behavior
    pub(super) world_seed: u64,
}

impl World {
    /// Returns a reference to the global constants.
    pub fn global_constants(&self) -> &GlobalConstants {
        &self.global_constants
    }

    /// Returns the current dimension version.
    pub fn dimension_version(&self) -> u64 {
        self.dimension_version
    }

    /// Returns a reference to the dimensions.
    pub fn dimensions(&self) -> &Arc<Dimensions> {
        &self.dimensions
    }

    /// Returns the number of organisms in the pool.
    pub fn organism_count(&self) -> usize {
        self.organism_ids.len()
    }

    /// Returns the world seed.
    pub fn world_seed(&self) -> u64 {
        self.world_seed
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("dimension_version", &self.dimension_version)
            .field("best_score", &self.best_score)
            .field("best_organism_id", &self.best_organism_id)
            .field("organism_count", &self.organism_ids.len())
            .field("next_organism_id", &self.next_organism_id)
            .field("world_seed", &self.world_seed)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn given_world_module_when_compiled_then_succeeds() {
        // Placeholder test - actual tests in submodules
        assert!(true);
    }
}
