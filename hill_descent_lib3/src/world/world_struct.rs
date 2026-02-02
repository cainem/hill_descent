//! World struct - the coordinator that orchestrates training runs.

use std::sync::Arc;

use indexmap::IndexMap;

use super::{Dimensions, Regions, WorldFunction};
use crate::{organism::Organism, parameters::GlobalConstants};

/// The World coordinates all operations in the genetic algorithm.
pub struct World {
    /// Organisms stored by ID for O(1) lookup.
    /// Uses Arc<Organism> with interior mutability (atomics + mutex) for lock-free access.
    /// Uses IndexMap to maintain insertion order for deterministic iteration.
    pub(super) organisms: IndexMap<u64, Arc<Organism>>,

    /// Spatial dimensions (bounds and intervals)
    pub(super) dimensions: Arc<Dimensions>,

    /// Current dimension version for incremental key updates
    pub(super) dimension_version: u64,

    /// Region management
    pub(super) regions: Regions,

    /// Fitness function shared by all organisms
    pub(super) world_function: Arc<dyn WorldFunction + Send + Sync>,

    /// Global configuration
    pub(super) global_constants: GlobalConstants,

    /// Best fitness score seen so far
    pub(super) best_score: f64,

    /// ID of the organism with the best score
    pub(super) best_organism_id: Option<u64>,

    /// Parameters (problem values only) of the best organism.
    pub(super) best_params: Vec<f64>,

    /// Next organism ID to assign
    pub(super) next_organism_id: u64,

    /// World seed for deterministic behavior
    pub(super) world_seed: u64,

    /// Epoch counter for reproduction RNG seeding (increments every epoch)
    pub(super) epoch_count: u64,
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
        self.organisms.len()
    }

    /// Returns all organism IDs currently in the pool.
    ///
    /// Note: Order is not guaranteed. For deterministic ordering,
    /// the caller should sort if needed.
    pub fn organism_ids(&self) -> Vec<u64> {
        self.organisms.keys().copied().collect()
    }

    /// Returns the world seed.
    pub fn world_seed(&self) -> u64 {
        self.world_seed
    }
}
