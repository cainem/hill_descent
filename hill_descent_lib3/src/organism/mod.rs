//! Organism module - Shared memory implementation
//!
//! Unlike lib2 which uses actors, this version uses direct method calls on shared mutable structures
//! protected by RwLocks in the World.

mod calculate_region_key_impl;
mod evaluate_fitness_impl;
mod increment_age_impl;
mod process_epoch_impl;
mod reproduce_impl;
mod update_dimensions_impl;

use std::sync::Arc;

use crate::NUM_SYSTEM_PARAMETERS;
use crate::{
    phenotype::Phenotype,
    world::{WorldFunction, dimensions::Dimensions, regions::region_key::RegionKey},
};

// ============================================================================
// Result types (Must be defined here as _impl modules expect them in super)
// ============================================================================

/// Result of process_epoch
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessEpochResult {
    Ok {
        region_key: RegionKey,
        score: f64,
        new_age: usize,
        should_remove: bool,
    },
    OutOfBounds {
        dimensions_exceeded: Vec<usize>,
    },
}

/// Result of reproduction.
#[derive(Debug, Clone, PartialEq)]
pub struct ReproduceResult {
    /// Two offspring phenotypes
    pub offspring_phenotypes: (Arc<Phenotype>, Arc<Phenotype>),
    /// Parent IDs for the new organisms (self.id, partner_id)
    pub parent_ids: (u64, u64),
}

/// Result of region key calculation.
#[derive(Debug, Clone, PartialEq)]
pub enum CalculateRegionKeyResult {
    Ok(RegionKey),
    OutOfBounds(Vec<usize>),
}

/// Result of fitness evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluateFitnessResult {
    pub score: f64,
    pub age: usize,
    pub region_key: RegionKey,
}

/// Result of age increment.
#[derive(Debug, Clone, PartialEq)]
pub struct IncrementAgeResult {
    pub should_remove: bool,
    pub new_age: usize,
}

/// Result of get_web_state
#[derive(Debug, Clone, PartialEq)]
pub struct GetWebStateResult {
    pub params: Vec<f64>,
    pub age: usize,
    pub max_age: usize,
    pub score: Option<f64>,
    pub region_key: Option<RegionKey>,
    pub is_dead: bool,
}

// ============================================================================
// Organism Struct
// ============================================================================

/// An organism in the genetic algorithm population.
///
/// In this shared-memory implementation, Organism is a struct accessed via RwLock
/// by parallel rayon iterators.
#[derive(Debug)]
pub struct Organism {
    /// Unique identifier
    id: u64,

    /// Parent IDs for pedigree tracking (None for initial random organisms)
    _parent_ids: (Option<u64>, Option<u64>),

    /// Current region key (calculated from phenotype + dimensions)
    region_key: Option<RegionKey>,

    /// Cached dimension version (for incremental key updates)
    dimension_version: u64,

    /// Genetic material (immutable after creation)
    phenotype: Arc<Phenotype>,

    /// Current dimensions reference
    dimensions: Arc<Dimensions>,

    /// Fitness function reference
    world_function: Arc<dyn WorldFunction + Send + Sync>,

    /// Current fitness score (None if not yet evaluated)
    score: Option<f64>,

    /// Age in training runs
    age: usize,

    /// Whether organism has exceeded max age
    is_dead: bool,
}

/// Initialization data for creating a new Organism.
#[derive(Debug, Clone)]
pub struct CreateOrganism {
    pub id: u64,
    pub parent_ids: (Option<u64>, Option<u64>),
    pub phenotype: Arc<Phenotype>,
    pub dimensions: Arc<Dimensions>,
    pub world_function: Arc<dyn WorldFunction + Send + Sync>,
}

impl Organism {
    /// Creates a new organism.
    pub fn new(init: CreateOrganism) -> Self {
        Self {
            id: init.id,
            _parent_ids: init.parent_ids,
            region_key: None,
            dimension_version: 0,
            phenotype: init.phenotype,
            dimensions: init.dimensions,
            world_function: init.world_function,
            score: None,
            age: 0,
            is_dead: false,
        }
    }

    /// Accessors
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_dead(&self) -> bool {
        self.is_dead
    }

    pub fn phenotype(&self) -> &Arc<Phenotype> {
        &self.phenotype
    }

    pub fn region_key(&self) -> Option<&RegionKey> {
        self.region_key.as_ref()
    }

    pub fn score(&self) -> Option<f64> {
        self.score
    }

    pub fn age(&self) -> usize {
        self.age
    }

    // Logic Methods

    /// Processes an organism's epoch: calculates region key, evaluates fitness, and increments age.
    pub fn process_epoch(
        &mut self,
        dimensions: Option<Arc<Dimensions>>,
        dimension_version: u64,
        changed_dimensions: Vec<usize>,
        training_data_index: usize,
    ) -> ProcessEpochResult {
        // Update dimensions if provided
        if let Some(dims) = dimensions {
            self.dimensions = dims;
        }

        // Take ownership of the current key
        let current_key = self.region_key.take();

        let result = process_epoch_impl::process_epoch(
            &self.phenotype,
            &self.dimensions,
            &self.world_function,
            self.age,
            training_data_index,
            current_key,
            self.dimension_version,
            dimension_version,
            &changed_dimensions,
        );

        // Update cached state based on result
        self.dimension_version = dimension_version;
        match &result {
            ProcessEpochResult::Ok {
                region_key,
                score,
                new_age,
                should_remove,
            } => {
                self.region_key = Some(region_key.clone());
                self.score = Some(*score);
                self.age = *new_age;
                self.is_dead = *should_remove;
            }
            ProcessEpochResult::OutOfBounds { .. } => {
                // Don't update state if out of bounds
            }
        }

        result
    }

    /// Reproduces with a partner. Does not modify self or partner.
    pub fn reproduce(
        &self,
        partner_phenotype: Arc<Phenotype>,
        reproduction_seed: u64,
    ) -> ReproduceResult {
        reproduce_impl::reproduce(
            &self.phenotype,
            self.id,
            &partner_phenotype,
            reproduction_seed,
        )
    }

    pub fn update_dimensions(&mut self, new_dimensions: Arc<Dimensions>, dimension_version: u64) {
        self.dimensions = update_dimensions_impl::update_dimensions(new_dimensions);
        self.dimension_version = dimension_version;
        // Invalidate cached key as dimensions have changed
        self.region_key = None;
    }

    pub fn get_web_state(&self) -> GetWebStateResult {
        let expressed = self.phenotype.expressed_values();
        // Extract position params (after NUM_SYSTEM_PARAMETERS)
        let params = expressed[NUM_SYSTEM_PARAMETERS..].to_vec();
        let max_age = self.phenotype.system_parameters().max_age();

        GetWebStateResult {
            params,
            age: self.age,
            max_age: max_age.round() as usize,
            score: self.score,
            region_key: self.region_key.clone(),
            is_dead: self.is_dead,
        }
    }
}
