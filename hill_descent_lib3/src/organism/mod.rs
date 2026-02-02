//! Organism module - Lock-free implementation using atomics
//!
//! This version uses atomic operations for mutable fields, eliminating the need
//! for RwLock wrappers in the World's organism storage. Only the region_key
//! uses a Mutex since it requires Clone operations.

mod calculate_region_key_impl;
mod evaluate_fitness_impl;
mod increment_age_impl;
mod process_epoch_impl;
mod reproduce_impl;
mod update_dimensions_impl;

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwapOption;

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
/// This implementation uses atomic operations for mutable fields to enable
/// lock-free concurrent access during parallel processing. The World stores
/// organisms as `Arc<Organism>` (no RwLock wrapper needed).
///
/// Thread-safety model:
/// - Immutable fields (id, parent_ids, phenotype, world_function): No synchronization needed
/// - Region key: Protected by Mutex (requires Clone for access)
/// - Other mutable fields: Atomic operations
/// - Dimensions: ArcSwap for lock-free updates
pub struct Organism {
    /// Unique identifier (immutable after creation)
    id: u64,

    /// Parent IDs for pedigree tracking (immutable after creation)
    _parent_ids: (Option<u64>, Option<u64>),

    /// Current region key (calculated from phenotype + dimensions)
    /// Protected by Mutex since RegionKey doesn't implement atomic operations
    region_key: Mutex<Option<RegionKey>>,

    /// Cached dimension version (for incremental key updates)
    /// Uses atomic for lock-free read/write
    dimension_version: AtomicU64,

    /// Genetic material (immutable after creation)
    phenotype: Arc<Phenotype>,

    /// Current dimensions reference (updated when dimensions are subdivided)
    /// Uses ArcSwap for lock-free updates
    dimensions: ArcSwapOption<Dimensions>,

    /// Fitness function reference (immutable after creation)
    world_function: Arc<dyn WorldFunction + Send + Sync>,

    /// Current fitness score stored as f64 bit representation (u64::MAX = None)
    /// Uses atomic for lock-free concurrent access during parallel processing
    score: AtomicU64,

    /// Age in training runs
    /// Uses atomic for lock-free increments
    age: AtomicUsize,

    /// Whether organism has exceeded max age
    /// Uses atomic for lock-free flag access
    is_dead: AtomicBool,
}

// Manual Debug implementation since ArcSwapOption doesn't implement Debug
impl std::fmt::Debug for Organism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Organism")
            .field("id", &self.id)
            .field("_parent_ids", &self._parent_ids)
            .field("region_key", &self.region_key)
            .field(
                "dimension_version",
                &self.dimension_version.load(Ordering::Relaxed),
            )
            .field("phenotype", &self.phenotype)
            .field("dimensions", &"<ArcSwapOption<Dimensions>>")
            .field("world_function", &"<dyn WorldFunction>")
            .field("score", &self.score())
            .field("age", &self.age())
            .field("is_dead", &self.is_dead())
            .finish()
    }
}

impl Organism {
    /// Creates a new organism with the given parameters.
    pub fn new(
        id: u64,
        parent_ids: (Option<u64>, Option<u64>),
        phenotype: Arc<Phenotype>,
        dimensions: Arc<Dimensions>,
        world_function: Arc<dyn WorldFunction + Send + Sync>,
    ) -> Self {
        Self {
            id,
            _parent_ids: parent_ids,
            region_key: Mutex::new(None),
            dimension_version: AtomicU64::new(0),
            phenotype,
            dimensions: ArcSwapOption::new(Some(dimensions)),
            world_function,
            score: AtomicU64::new(u64::MAX), // u64::MAX represents None
            age: AtomicUsize::new(0),
            is_dead: AtomicBool::new(false),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Returns the organism's unique identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns whether the organism has been marked as dead.
    /// Thread-safe atomic read.
    pub fn is_dead(&self) -> bool {
        self.is_dead.load(Ordering::Relaxed)
    }

    /// Returns a reference to the organism's phenotype.
    pub fn phenotype(&self) -> &Arc<Phenotype> {
        &self.phenotype
    }

    /// Returns a clone of the current region key.
    /// Thread-safe via Mutex.
    pub fn region_key(&self) -> Option<RegionKey> {
        self.region_key.lock().unwrap().clone()
    }

    /// Takes the region key from the organism, leaving None in its place.
    ///
    /// This is more efficient than `region_key()` when you need to mutate the key,
    /// as it avoids incrementing the Arc refcount and thus prevents `Arc::make_mut`
    /// from cloning the underlying Vec during `update_position()` calls.
    ///
    /// Thread-safe via Mutex.
    pub fn take_region_key(&self) -> Option<RegionKey> {
        self.region_key.lock().unwrap().take()
    }

    /// Returns the current fitness score.
    /// Thread-safe atomic read.
    pub fn score(&self) -> Option<f64> {
        let bits = self.score.load(Ordering::Acquire);
        if bits == u64::MAX {
            None
        } else {
            Some(f64::from_bits(bits))
        }
    }

    /// Returns the current age.
    /// Thread-safe atomic read.
    pub fn age(&self) -> usize {
        self.age.load(Ordering::Relaxed)
    }

    /// Returns the cached dimension version.
    /// Thread-safe atomic read.
    pub fn cached_dimension_version(&self) -> u64 {
        self.dimension_version.load(Ordering::Relaxed)
    }

    /// Returns a clone of the current dimensions Arc.
    /// Thread-safe via ArcSwap.
    pub fn dimensions(&self) -> Option<Arc<Dimensions>> {
        self.dimensions.load_full()
    }

    // ========================================================================
    // Mutators (thread-safe atomic operations)
    // ========================================================================

    /// Sets the organism's dimensions (used when dimensions are subdivided).
    /// Thread-safe via ArcSwap.
    pub fn set_dimensions(&self, dimensions: Arc<Dimensions>) {
        self.dimensions.store(Some(dimensions));
    }

    /// Sets the organism's region key.
    /// Thread-safe via Mutex.
    pub fn set_region_key(&self, region_key: Option<RegionKey>) {
        *self.region_key.lock().unwrap() = region_key;
    }

    /// Sets the fitness score.
    /// Thread-safe atomic write.
    fn set_score(&self, score: Option<f64>) {
        let bits = score.map(|s| s.to_bits()).unwrap_or(u64::MAX);
        self.score.store(bits, Ordering::Release);
    }

    /// Sets the age.
    /// Thread-safe atomic write.
    fn set_age(&self, age: usize) {
        self.age.store(age, Ordering::Relaxed);
    }

    /// Marks the organism as dead.
    /// Thread-safe atomic write.
    fn mark_dead(&self) {
        self.is_dead.store(true, Ordering::Relaxed);
    }

    /// Sets the dimension version.
    /// Thread-safe atomic write.
    fn set_dimension_version(&self, version: u64) {
        self.dimension_version.store(version, Ordering::Relaxed);
    }

    // ========================================================================
    // Logic Methods
    // ========================================================================

    /// Returns whether this organism has already been successfully processed in the current epoch.
    /// Used to skip re-evaluation during retry loops.
    ///
    /// Note: This is a caching optimization. There's a theoretical race condition between
    /// checking dimension_version and region_key, but the fallback (full recompute) is safe.
    pub fn is_epoch_complete(&self, dimension_version: u64) -> bool {
        // Check version first (atomic), then region_key (mutex)
        self.dimension_version.load(Ordering::Acquire) == dimension_version
            && self.region_key.lock().unwrap().is_some()
    }

    /// Returns the cached result from a previously successful epoch processing.
    /// Only valid when `is_epoch_complete` returns true.
    pub fn get_cached_epoch_result(&self) -> Option<ProcessEpochResult> {
        let region_key = self.region_key.lock().unwrap().clone();
        region_key.map(|key| ProcessEpochResult::Ok {
            region_key: key,
            score: self.score().unwrap_or(f64::MAX),
            new_age: self.age(),
            should_remove: self.is_dead(),
        })
    }

    /// Returns whether this organism has a valid cached region key for the given dimension version.
    /// Used to determine if we can skip region key calculation (but still need fitness eval).
    pub fn has_valid_region_key(&self, dimension_version: u64) -> bool {
        self.dimension_version.load(Ordering::Acquire) == dimension_version
            && self.region_key.lock().unwrap().is_some()
    }

    /// Processes an epoch using cached region key but fresh fitness evaluation and age increment.
    ///
    /// This is an optimization for when dimensions haven't changed - we reuse the cached
    /// region key but still evaluate fitness (training data may vary) and increment age.
    ///
    /// # Panics
    ///
    /// Panics if called without a valid cached region key.
    pub fn process_epoch_with_cached_region_key(
        &self,
        training_data_index: usize,
    ) -> ProcessEpochResult {
        // Get cached region key (must exist - caller should check has_valid_region_key first)
        let region_key = self
            .region_key
            .lock()
            .unwrap()
            .clone()
            .expect("process_epoch_with_cached_region_key called without valid cached key");

        // Get current age before incrementing
        let current_age = self.age();

        // Evaluate fitness (training data may have changed)
        let (fitness_result, score) = evaluate_fitness_impl::evaluate_fitness(
            &self.phenotype,
            &self.world_function,
            &region_key,
            current_age,
            training_data_index,
        );

        // Increment age and check death
        let max_age = self.phenotype.system_parameters().max_age();
        let (age_result, new_age, _) =
            increment_age_impl::increment_age(fitness_result.age, max_age);

        // Update cached state
        self.set_score(Some(score));
        self.set_age(new_age);
        if age_result.should_remove {
            self.mark_dead();
        }

        ProcessEpochResult::Ok {
            region_key,
            score,
            new_age,
            should_remove: age_result.should_remove,
        }
    }

    /// Processes an organism's epoch: calculates region key, evaluates fitness, and increments age.
    ///
    /// This method uses interior mutability (atomics and mutex) rather than &mut self,
    /// enabling lock-free access from the World.
    pub fn process_epoch(
        &self,
        dimensions: Option<&Arc<Dimensions>>,
        dimension_version: u64,
        changed_dimensions: &[usize],
        training_data_index: usize,
    ) -> ProcessEpochResult {
        // Update dimensions if provided (atomic via ArcSwap)
        if let Some(dims) = dimensions {
            self.dimensions.store(Some(Arc::clone(dims)));
        }

        // Get current dimensions
        let current_dims = self.dimensions.load_full().expect("Dimensions not set");

        // Take the region key rather than cloning - this avoids incrementing the Arc refcount,
        // which prevents expensive Arc::make_mut cloning during update_position calls.
        // On OOB, the key stays None which triggers a full recalculation on retry.
        let current_key = self.take_region_key();
        let cached_dim_version = self.dimension_version.load(Ordering::Acquire);

        let result = process_epoch_impl::process_epoch(
            &self.phenotype,
            &current_dims,
            &self.world_function,
            self.age(),
            training_data_index,
            current_key, // Move ownership - no clone needed
            cached_dim_version,
            dimension_version,
            changed_dimensions,
        );

        // Update cached state based on result (using atomic operations)
        match &result {
            ProcessEpochResult::Ok {
                region_key,
                score,
                new_age,
                should_remove,
            } => {
                self.set_dimension_version(dimension_version);
                self.set_region_key(Some(region_key.clone()));
                self.set_score(Some(*score));
                self.set_age(*new_age);
                if *should_remove {
                    self.mark_dead();
                }
            }
            ProcessEpochResult::OutOfBounds { .. } => {
                // Key stays None (taken above) which triggers full recalculation on retry.
                // Don't update dimension_version so incremental update works on retry.
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

    /// Updates dimensions and invalidates cached region key.
    pub fn update_dimensions(&self, new_dimensions: Arc<Dimensions>, dimension_version: u64) {
        self.dimensions
            .store(Some(update_dimensions_impl::update_dimensions(
                new_dimensions,
            )));
        self.set_dimension_version(dimension_version);
        // Invalidate cached key as dimensions have changed
        self.set_region_key(None);
    }

    /// Returns state for web visualization.
    pub fn get_web_state(&self) -> GetWebStateResult {
        let expressed = self.phenotype.expressed_values();
        // Extract position params (after NUM_SYSTEM_PARAMETERS)
        let params = expressed[NUM_SYSTEM_PARAMETERS..].to_vec();
        let max_age = self.phenotype.system_parameters().max_age();

        GetWebStateResult {
            params,
            age: self.age(),
            max_age: max_age.round() as usize,
            score: self.score(),
            region_key: self.region_key(),
            is_dead: self.is_dead(),
        }
    }
}
