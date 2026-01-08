# Product Definition Document: hill_descent_lib2

## 1. Overview

### 1.1 Purpose

`hill_descent_lib2` is a re-architected version of `hill_descent_lib` that uses a message-passing concurrency model based on the `messaging_thread_pool` crate. The goal is to improve performance for complex fitness functions by:

1. Eliminating shared mutable state and locking overhead
2. Enabling better CPU cache locality through thread affinity
3. Parallelizing dimension bound checking in a single pass
4. Providing a cleaner, more maintainable architecture

### 1.2 Relationship to hill_descent_lib

- **API Compatibility**: The public API surface remains unchanged
- **Coexistence**: Both libraries will exist side-by-side during development
- **Behavioral Equivalence**: Given the same seed and inputs, both libraries should produce identical results

### 1.3 Core Architectural Change

**Old Model (hill_descent_lib)**:
- Organisms stored in collections with `Arc<Organism>`
- Shared mutable state protected by `Mutex` and atomics
- Parallelism via Rayon's work-stealing

**New Model (hill_descent_lib2)**:
- Organisms as pool items in a `messaging_thread_pool`
- Each organism lives on a single thread (determined by `id % thread_count`)
- Communication via typed request/response messages
- Coordinator orchestrates training runs via message batches

---

## 2. System Components

### 2.1 World (Coordinator)

The `World` struct owns the thread pool and coordinates all operations.

```rust
pub struct World {
    /// Thread pool containing all organisms
    organism_pool: ThreadPool<Organism>,
    
    /// Spatial dimensions (bounds and intervals)
    dimensions: Arc<Dimensions>,
    
    /// Dimension version for incremental key updates
    dimension_version: u64,
    
    /// Region management (not in thread pool initially)
    regions: Regions,
    
    /// Fitness function shared by all organisms
    world_function: Arc<dyn WorldFunction>,
    
    /// Global configuration
    global_constants: GlobalConstants,
    
    /// Tracking for best organism
    best_score: f64,
    best_organism_id: Option<u64>,
    
    /// Parameters of the best organism (cached)
    best_params: Vec<f64>,
    
    /// All organism IDs currently in the pool
    organism_ids: Vec<u64>,
    
    /// Next organism ID to assign
    next_organism_id: u64,

    /// World seed for deterministic behavior
    world_seed: u64,
}
```

### 2.2 Organism (Pool Item)

Organisms are pool items that process messages on their assigned thread.

```rust
#[derive(Debug)]
pub struct Organism {
    /// Unique identifier (also used for thread routing)
    id: u64,
    
    /// Parent IDs for pedigree tracking
    parent_ids: (Option<u64>, Option<u64>),
    
    /// Current region key (calculated from phenotype + dimensions)
    region_key: Option<RegionKey>,
    
    /// Cached dimension version (for incremental updates)
    dimension_version: u64,
    
    /// Genetic material (immutable after creation)
    phenotype: Arc<Phenotype>,
    
    /// Current dimensions reference (updated via message)
    dimensions: Arc<Dimensions>,
    
    /// Fitness function reference
    world_function: Arc<dyn WorldFunction>,
    
    /// Current fitness score
    score: Option<f64>,
    
    /// Age in training runs
    age: usize,
    
    /// Whether organism has exceeded max age
    is_dead: bool,
}
```

### 2.3 Region (Non-Pooled)

Regions are managed via Rayon parallelism (not in a thread pool initially).

```rust
#[derive(Debug)]
pub struct Region {
    /// Organisms in this region (transient, rebuilt each training run)
    organisms: Vec<OrganismEntry>,
    
    /// Minimum fitness score in this region
    min_score: Option<f64>,
    
    /// Carrying capacity (calculated from relative fitness)
    carrying_capacity: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct OrganismEntry {
    /// Organism ID (for sending messages)
    id: u64,
    
    /// Age (for sorting tie-breaker)
    age: usize,
    
    /// Fitness score (for sorting and capacity calculation)
    score: Option<f64>,
}
```

### 2.4 Dimensions

Dimensions track the spatial bounds with versioning for incremental updates.

```rust
#[derive(Debug, Clone)]
pub struct Dimensions {
    dimensions: Vec<Dimension>,
    version: u64,
}
```

---

## 3. Message Types

### 3.1 Organism Messages

#### 3.1.1 ProcessEpoch

Aggregated request to process a full epoch step for an organism (Region Key + Fitness + Age).
This combines what would otherwise be separate sequential calls to reduce synchronization overhead.

```rust
struct ProcessEpochRequest(
    u64,                              // id
    Option<Arc<Dimensions>>,          // dimensions (if updated)
    u64,                              // dimension_version
    Vec<usize>,                       // changed_dimensions
    usize                             // training_data_index
);

enum ProcessEpochResult {
    /// Epoch processed successfully - organism was in bounds.
    Ok {
        region_key: RegionKey,
        score: f64,
        new_age: usize,
        should_remove: bool,
    },
    /// Organism is outside dimension bounds - needs dimension expansion.
    OutOfBounds {
        dimensions_exceeded: Vec<usize>,
    },
}
```

#### 3.1.2 GetPhenotype

Request an organism's phenotype for reproduction.

```rust
struct GetPhenotypeRequest {
    id: u64,
}

struct GetPhenotypeResponse {
    id: u64,
    phenotype: Arc<Phenotype>,
}
```

#### 3.1.3 Reproduce

Request an organism to perform reproduction with a partner's phenotype.

```rust
struct ReproduceRequest {
    id: u64,
    partner_phenotype: Arc<Phenotype>,
    reproduction_seed: u64,
}

struct ReproduceResponse {
    id: u64,
    offspring_phenotypes: (Arc<Phenotype>, Arc<Phenotype>),
    parent_ids: (u64, u64),  // For new organism creation
}
```

#### 3.1.4 UpdateDimensions

Update the organism's dimensions reference.

```rust
struct UpdateDimensionsRequest {
    id: u64,
    dimensions: Arc<Dimensions>,
}

struct UpdateDimensionsResponse {
    id: u64,
}
```

---

## 4. Training Run Flow

### 4.1 High-Level Flow

```
training_run(data: TrainingData) -> bool
│
├── 1. Process Epoch (Unified Step)
│   ├── Send ProcessEpochRequest to all organisms
│   ├── Collect responses
│   ├── If any OutOfBounds:
│   │   ├── Expand dimensions (union of all exceeded)
│   │   ├── Increment dimension_version
│   │   ├── Retry process_step for all organisms with new dimensions
│   │   └── Repeat until all Ok
│   └── All Ok: proceed
│
├── 2. Update Region Capacities
│   ├── Calculate min_score per region
│   ├── Calculate capacity using inverse fitness formula
│   └── Assign capacities to regions
│
├── 3. Region Processing (parallel via Rayon)
│   ├── Populate regions with organisms based on responses
│   ├── Sort organisms by score (primary), age (secondary)
│   ├── Determine organisms to remove (exceeded capacity)
│   ├── Determine reproduction pairs (extreme pairing)
│   └── Return processing results
│
├── 4. Cull Over-Capacity
│   ├── Remove organisms marked for removal given capacity constraints
│   └── Update organism_ids list
│
├── 5. Reproduction
│   ├── For each pair:
│   │   ├── Send GetPhenotypeRequest to parent2
│   │   ├── Send ReproduceRequest to parent1 with parent2's phenotype
│   │   ├── Receive offspring phenotypes
│   │   └── Create new organisms in pool
│   └── Collect all new organism IDs
│
├── 6. Cull Aged-Out
│   ├── Remove organisms that signalled should_remove (due to max age) in step 1
│   └── Update organism_ids list
│
└── 8. Return
    └── Return true if at resolution limit, false otherwise
```

### 4.2 Detailed Step: Reproduction

```
perform_reproduction(pairs: Vec<(u64, u64)>) -> Vec<u64>
│
├── new_organism_ids = []
│
├── For each (parent1_id, parent2_id) in pairs:
│   │
│   ├── Get parent2's phenotype:
│   │   └── Send GetPhenotypeRequest(parent2_id)
│   │   └── Receive GetPhenotypeResponse { phenotype }
│   │
│   ├── Calculate deterministic seed:
│   │   └── reproduction_seed = derive_seed(world_seed, parent1_id, parent2_id)
│   │
│   ├── Request reproduction:
│   │   └── Send ReproduceRequest {
│   │       id: parent1_id,
│   │       partner_phenotype: phenotype,
│   │       reproduction_seed,
│   │   }
│   │   └── Receive ReproduceResponse {
│   │       offspring_phenotypes: (child1, child2),
│   │       parent_ids,
│   │   }
│   │
│   └── Create new organisms:
│       ├── new_id_1 = create_organism(child1, parent_ids)
│       ├── new_id_2 = create_organism(child2, parent_ids)
│       └── new_organism_ids.extend([new_id_1, new_id_2])
│
└── Return new_organism_ids
```

---

## 5. Determinism

### 5.1 Requirements

Given the same:
- Initial random seed (`GlobalConstants.world_seed`)
- Parameter bounds
- Fitness function
- Training data sequence

The system must produce identical results across runs.

### 5.2 Mechanisms

1. **Message Send Order**: Coordinator sends messages in deterministic order (by organism ID or region key)

2. **Thread-Local RNG Not Used for Reproduction**: Each reproduction uses a seed derived deterministically from:
   ```rust
   fn derive_reproduction_seed(world_seed: u64, parent1_id: u64, parent2_id: u64) -> u64 {
       // Deterministic hash combining all three values
   }
   ```

3. **Region Processing Order**: Regions are processed in deterministic order (by RegionKey hash)

4. **Organism Creation Order**: New organisms created in deterministic order based on reproduction pair order

---

## 6. Public API

The public API remains unchanged from `hill_descent_lib`:

```rust
/// Creates and initializes a new optimization world.
pub fn setup_world(
    params: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    function: Box<dyn WorldFunction>,
) -> World;

impl World {
    /// Runs a single epoch of genetic algorithm evolution.
    pub fn training_run(&mut self, data: TrainingData) -> bool;
    
    /// Returns the best (lowest) fitness score found.
    pub fn get_best_score(&self) -> f64;
    
    /// Returns the problem parameters of the current best organism.
    pub fn get_best_params(&self) -> Vec<f64>;
    
    /// Returns the best organism after evaluation.
    pub fn get_best_organism(&mut self, data: TrainingData) -> Arc<Organism>;
    
    /// Returns the current state for analysis/visualization.
    pub fn get_state(&self) -> WorldState;
}
```

---

## 7. Error Handling

### 7.1 Thread Pool Errors

If the thread pool becomes unavailable (threads panic), the coordinator should:
1. Log the error
2. Panic with a descriptive message (unrecoverable)

### 7.2 Fitness Function Errors

Same as current behavior:
- Panic if fitness returns NaN, Infinity, or below floor
- These indicate bugs in the user's fitness function

---

## 8. Performance Considerations

### 8.1 Expected Improvements

1. **Parallel Dimension Checking**: All organisms check bounds simultaneously; all exceeded dimensions discovered in one pass

2. **Reduced Lock Contention**: No `Mutex` on region_key, no atomics for score/age/is_dead

3. **Cache Locality**: Organisms stay on same thread, improving L1/L2 cache hit rates

4. **Simplified Ownership**: No `Arc<Organism>` cloning during region redistribution

### 8.2 Expected Overhead

1. **Message Passing**: Each operation requires message send/receive
2. **Trivial Fitness Functions**: May be slower than current implementation due to message overhead
3. **Region Recreation**: Still requires full rebuild when dimensions change

### 8.3 Target Use Case

Optimized for:
- Complex fitness functions (neural network evaluation)
- High-dimensional problems (100+ parameters)
- Large populations (1000+ organisms)

---

## 9. Testing Requirements

### 9.1 Coverage

- 100% branch and condition coverage for all functions
- Tests follow `given_xxx_when_yyy_then_zzz` naming convention

### 9.2 Determinism Tests

- Same seed produces same results across multiple runs
- Results match `hill_descent_lib` for identical inputs (where applicable)

### 9.3 Integration Tests

- Full training run cycles
- Standard benchmark functions (Sphere, Rosenbrock, Himmelblau, etc.)

---

## 10. Dependencies

### 10.1 New Dependencies

```toml
[dependencies]
messaging_thread_pool = "5.0"  # Thread pool with message passing
mimalloc = "0.1"               # High-performance allocator
```

### 10.2 Retained Dependencies

```toml
[dependencies]
rand = "0.8"
xxhash-rust = { version = "0.8", features = ["xxh3"] }
rayon = "1.10"  # For region parallel processing
serde = { version = "1.0", features = ["derive"], optional = true }
```

---

## 11. File Structure

```
hill_descent_lib2/
├── Cargo.toml
├── pdd.md                          # This document
├── README.md
├── src/
│   ├── lib.rs                      # Public API exports
│   ├── world/
│   │   ├── mod.rs                  # World struct and core impl
│   │   ├── training_run.rs         # Main training loop
│   │   ├── single_valued_function.rs
│   │   ├── world_function.rs
│   │   ├── process_epoch.rs        # Unified epoch processing
│   │   ├── regions/
│   │   │   ├── mod.rs              # Regions container
│   │   │   ├── region.rs           # Single region
│   │   │   ├── region_key.rs
│   │   │   ├── populate.rs
│   │   │   ├── carrying_capacity.rs
│   │   │   └── process.rs          # Sort, truncate, pair
│   │   ├── dimensions/
│   │   │   ├── mod.rs              # Dimensions with versioning
│   │   │   ├── dimension.rs        # Single dimension
│   │   │   └── expand.rs
│   │   ├── reproduction.rs
│   │   ├── age_and_cull.rs
│   │   ├── get_best_score.rs
│   │   ├── get_best_params.rs
│   │   ├── get_best_organism.rs
│   │   └── setup_world.rs
│   ├── organism/
│   │   ├── mod.rs                  # Organism pool item
│   │   ├── process_epoch_impl.rs   # Unified epoch implementation
│   │   ├── calculate_region_key_impl.rs
│   │   ├── evaluate_fitness_impl.rs
│   │   ├── reproduce_impl.rs
│   │   └── increment_age_impl.rs
│   ├── phenotype/                  # Copied from lib1
│   ├── gamete/                     # Copied from lib1
│   ├── locus/                      # Copied from lib1
│   └── parameters/                 # Copied from lib1
└── tests/
    ├── determinism_test.rs
    ├── training_run_test.rs
    └── benchmark_functions/
```

---

## 12. Glossary

| Term | Definition |
|------|------------|
| Coordinator | The `World` struct that owns the thread pool and orchestrates training runs |
| Pool Item | An object managed by `messaging_thread_pool`, lives on a single thread |
| Organism Entry | Lightweight struct (id, age, score) stored in regions for sorting |
| Dimension Version | Counter incremented when dimension bounds change |
| Reproduction Seed | Deterministic seed derived from world_seed and parent IDs |
