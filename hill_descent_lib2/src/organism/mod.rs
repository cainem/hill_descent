//! Organism module - pool items that process messages on their assigned thread.
//!
//! Each organism lives on a single thread (determined by `id % thread_count`)
//! and processes messages via the `messaging_thread_pool` crate.
//!
//! # Architecture
//!
//! The `#[pool_item]` macro requires all `#[messaging]` methods in the same impl block.
//! To keep code organized, we use a delegation pattern:
//! - `mod.rs` contains the struct, `#[pool_item]` impl block, and result types
//! - `*_impl.rs` files contain the actual implementation logic as free functions
//! - Messaging methods in the impl block delegate to these implementation functions
//!
//! # Generated Types
//!
//! The `#[pool_item]` macro generates:
//! - `OrganismInit` request type for creating organisms
//! - Request/Response types for each `#[messaging]` method
//! - The `OrganismApi` enum containing all message types
//! - `PoolItem` trait implementation

// Implementation modules - contain the actual logic as free functions
mod calculate_region_key_impl;
mod evaluate_fitness_impl;
mod increment_age_impl;
mod reproduce_impl;
mod update_dimensions_impl;

use std::sync::Arc;

use messaging_thread_pool::{
    AddResponse, IdTargeted, RequestWithResponse, ThreadRequestResponse, pool_item,
};

use crate::{
    phenotype::Phenotype,
    world::{WorldFunction, dimensions::Dimensions, regions::region_key::RegionKey},
};

/// An organism in the genetic algorithm population.
///
/// Organisms are pool items that live on a single thread and process messages.
/// Each organism has:
/// - A unique ID used for thread routing
/// - Genetic material (phenotype)
/// - Current fitness score
/// - Region key for spatial partitioning
/// - Age tracking for generational culling
#[derive(Debug)]
pub struct Organism {
    /// Unique identifier (also used for thread routing via `id % thread_count`)
    id: u64,

    /// Parent IDs for pedigree tracking (None for initial random organisms)
    _parent_ids: (Option<u64>, Option<u64>),

    /// Current region key (calculated from phenotype + dimensions)
    region_key: Option<RegionKey>,

    /// Cached dimension version (for incremental key updates)
    dimension_version: u64,

    /// Genetic material (immutable after creation)
    phenotype: Arc<Phenotype>,

    /// Current dimensions reference (updated via message)
    dimensions: Arc<Dimensions>,

    /// Fitness function reference (must be Send+Sync for thread pool)
    world_function: Arc<dyn WorldFunction + Send + Sync>,

    /// Current fitness score (None if not yet evaluated)
    score: Option<f64>,

    /// Age in training runs
    age: usize,

    /// Whether organism has exceeded max age
    is_dead: bool,
}

/// Initialization data for creating a new Organism.
///
/// This struct is used as the custom Init type for the `#[pool_item]` macro.
/// It contains all the data needed to construct an organism on its target thread.
#[derive(Debug, Clone)]
pub struct CreateOrganism {
    /// The unique identifier for the organism (also used for thread routing)
    pub id: u64,
    /// Parent IDs for pedigree tracking (None for initial random organisms)
    pub parent_ids: (Option<u64>, Option<u64>),
    /// The organism's genetic material
    pub phenotype: Arc<Phenotype>,
    /// Reference to the current dimensions
    pub dimensions: Arc<Dimensions>,
    /// The fitness function to evaluate with
    pub world_function: Arc<dyn WorldFunction + Send + Sync>,
}

impl IdTargeted for CreateOrganism {
    fn id(&self) -> u64 {
        self.id
    }
}

impl RequestWithResponse<Organism> for CreateOrganism {
    type Response = AddResponse;
}

impl From<CreateOrganism> for ThreadRequestResponse<Organism> {
    fn from(request: CreateOrganism) -> Self {
        use messaging_thread_pool::RequestResponse;
        ThreadRequestResponse::AddPoolItem(RequestResponse::new(request))
    }
}

impl IdTargeted for Organism {
    fn id(&self) -> u64 {
        self.id
    }
}

#[pool_item(Init = "CreateOrganism")]
impl Organism {
    /// Creates a new organism from a CreateOrganism request.
    ///
    /// This method is called by the thread pool when processing a CreateOrganism
    /// initialization request.
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

    /// Calculates the region key for this organism based on its phenotype and dimensions.
    ///
    /// Returns the region key or indicates which dimensions are out of bounds.
    #[messaging(CalculateRegionKeyRequest, CalculateRegionKeyResponse)]
    pub fn calculate_region_key(
        &mut self,
        dimension_version: u64,
        changed_dimensions: Vec<usize>,
    ) -> CalculateRegionKeyResult {
        let (result, new_version) = calculate_region_key_impl::calculate_region_key(
            &self.phenotype,
            &self.dimensions,
            self.region_key.as_ref(),
            self.dimension_version,
            dimension_version,
            &changed_dimensions,
        );

        // Update cached state
        self.dimension_version = new_version;
        if let CalculateRegionKeyResult::Ok(ref key) = result {
            self.region_key = Some(key.clone());
        }

        result
    }

    /// Evaluates the organism's fitness using the world function.
    ///
    /// For function optimization, training_data_index is ignored.
    /// For supervised learning, it references a row in the shared training data.
    #[messaging(EvaluateFitnessRequest, EvaluateFitnessResponse)]
    pub fn evaluate_fitness(&mut self, training_data_index: usize) -> EvaluateFitnessResult {
        let region_key = self
            .region_key
            .clone()
            .expect("Region key must be calculated before fitness evaluation");

        let (result, score) = evaluate_fitness_impl::evaluate_fitness(
            &self.phenotype,
            &self.world_function,
            &region_key,
            self.age,
            training_data_index,
        );

        // Cache the score
        self.score = Some(score);

        result
    }

    /// Returns the organism's phenotype for reproduction pairing.
    #[messaging(GetPhenotypeRequest, GetPhenotypeResponse)]
    pub fn get_phenotype(&self) -> Arc<Phenotype> {
        self.phenotype.clone()
    }

    /// Reproduces with a partner's phenotype to create offspring.
    ///
    /// # Arguments
    /// * `partner_phenotype` - The partner's genetic material
    /// * `reproduction_seed` - Seed for deterministic reproduction
    #[messaging(ReproduceRequest, ReproduceResponse)]
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

    /// Increments the organism's age and checks if it should die.
    #[messaging(IncrementAgeRequest, IncrementAgeResponse)]
    pub fn increment_age(&mut self) -> IncrementAgeResult {
        let max_age = self.phenotype.system_parameters().max_age();

        let (result, new_age, is_dead) = increment_age_impl::increment_age(self.age, max_age);

        // Update state
        self.age = new_age;
        self.is_dead = is_dead;

        result
    }

    /// Updates the organism's dimensions reference when bounds expand.
    #[messaging(UpdateDimensionsRequest, UpdateDimensionsResponse)]
    pub fn update_dimensions(&mut self, new_dimensions: Arc<Dimensions>) {
        self.dimensions = update_dimensions_impl::update_dimensions(new_dimensions);
    }

    /// Returns organism state for web visualization.
    ///
    /// This provides a snapshot of the organism's current state including
    /// position, score, age, and other data needed for 2D visualization.
    #[messaging(GetWebStateRequest, GetWebStateResponse)]
    pub fn get_web_state(&self) -> GetWebStateResult {
        let expressed = self.phenotype.expressed_values();
        // Extract position params (after NUM_SYSTEM_PARAMETERS)
        let params = expressed[crate::NUM_SYSTEM_PARAMETERS..].to_vec();
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

// ============================================================================
// Result types for messaging methods
// ============================================================================

/// Result of region key calculation.
#[derive(Debug, Clone, PartialEq)]
pub enum CalculateRegionKeyResult {
    /// Key calculated successfully.
    Ok(RegionKey),
    /// Organism is outside bounds of specified dimensions.
    OutOfBounds(Vec<usize>),
}

/// Result of fitness evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluateFitnessResult {
    /// Calculated fitness score
    pub score: f64,
    /// Current age of organism
    pub age: usize,
    /// Current region key
    pub region_key: RegionKey,
}

/// Result of reproduction.
#[derive(Debug, Clone, PartialEq)]
pub struct ReproduceResult {
    /// Two offspring phenotypes
    pub offspring_phenotypes: (Arc<Phenotype>, Arc<Phenotype>),
    /// Parent IDs for the new organisms (self.id, partner_id)
    pub parent_ids: (u64, u64),
}

/// Result of age increment.
#[derive(Debug, Clone, PartialEq)]
pub struct IncrementAgeResult {
    /// Whether the organism should be removed (exceeded max age)
    pub should_remove: bool,
    /// The organism's new age
    pub new_age: usize,
}

/// Result of get_web_state - data needed for web visualization.
#[derive(Debug, Clone, PartialEq)]
pub struct GetWebStateResult {
    /// Position parameters (problem space coordinates)
    pub params: Vec<f64>,
    /// Current age in training runs
    pub age: usize,
    /// Maximum age before death
    pub max_age: usize,
    /// Fitness score (None if not evaluated)
    pub score: Option<f64>,
    /// Current region key
    pub region_key: Option<RegionKey>,
    /// Whether organism has exceeded max age
    pub is_dead: bool,
}

// ============================================================================
// Additional accessors (not messaging methods)
// ============================================================================

impl Organism {
    /// Returns whether the organism is dead.
    pub fn is_dead(&self) -> bool {
        self.is_dead
    }

    /// Returns a reference to the organism's phenotype.
    pub fn phenotype(&self) -> &Arc<Phenotype> {
        &self.phenotype
    }

    /// Returns the organism's current region key, if calculated.
    pub fn region_key(&self) -> Option<&RegionKey> {
        self.region_key.as_ref()
    }

    /// Returns the organism's current score, if evaluated.
    pub fn score(&self) -> Option<f64> {
        self.score
    }

    /// Returns the organism's age in training runs.
    pub fn age(&self) -> usize {
        self.age
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn given_organism_module_when_compiled_then_succeeds() {
        // Placeholder test - actual tests are in *_impl.rs files
        assert!(true);
    }
}
