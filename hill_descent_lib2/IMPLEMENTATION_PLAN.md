# Implementation Plan: hill_descent_lib2

## Overview

This document outlines the multi-stage implementation plan for `hill_descent_lib2`. The approach is:

1. **Scaffold first**: Create all module structure and function signatures with `todo!()` bodies
2. **Implement leaf nodes**: Start with components that have no internal dependencies
3. **Work toward root**: Progressively implement components that depend on completed ones
4. **Test at each stage**: Achieve 100% branch/condition coverage before moving to next stage

---

## Stage 0: Project Setup ✅ COMPLETE

### 0.1 Create Crate Structure

**Tasks:**
- [x] Create `Cargo.toml` with dependencies
- [x] Create `src/lib.rs` with module declarations
- [x] Add crate to workspace `Cargo.toml`

**Files:**
```
hill_descent_lib2/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Acceptance Criteria:**
- [x] `cargo build -p hill_descent_lib2` succeeds
- [x] `cargo test -p hill_descent_lib2` runs (1 placeholder test passes)

---

## Stage 1: Copy Unchanged Components ✅ COMPLETE

These components are unchanged from `hill_descent_lib` and can be copied directly.

### 1.1 Core Genetic Types

**Tasks:**
- [x] Copy `parameters/` module (GlobalConstants, Parameter, SystemParameters)
- [x] Copy `locus/` module (Locus, LocusAdjustment, DirectionOfTravel)
- [x] Copy `gamete/` module (Gamete, crossover logic)
- [x] Copy `phenotype/` module (Phenotype, sexual_reproduction)
- [x] Copy `training_data.rs`
- [x] Copy `world_function.rs` and `single_valued_function.rs` (into `world/` module)
- [x] Copy `gen_hybrid_range.rs` (required by locus module)

**Files copied:**
```
From hill_descent_lib/src/:
  parameters/               → hill_descent_lib2/src/parameters/
  locus/                    → hill_descent_lib2/src/locus/
  gamete/                   → hill_descent_lib2/src/gamete/
  phenotype/                → hill_descent_lib2/src/phenotype/
  training_data.rs          → hill_descent_lib2/src/training_data.rs
  gen_hybrid_range.rs       → hill_descent_lib2/src/gen_hybrid_range.rs
  world/world_function.rs   → hill_descent_lib2/src/world/world_function.rs
  world/single_valued_function.rs → hill_descent_lib2/src/world/single_valued_function.rs
```

**Acceptance Criteria:**
- [x] All copied tests pass (128 unit tests)
- [x] Doc tests pass (13 passing, 7 ignored pending World implementation)
- [x] No compilation errors
- [x] `cargo clippy` clean (1 expected dead_code warning for enhance_parameters)

---

## Stage 2: Scaffold New Architecture ✅ COMPLETE

### 2.1 Organism Module Scaffold ✅

**Tasks:**
- [x] Create `organism/mod.rs` with Organism struct
- [x] Create result types (CalculateRegionKeyResult, EvaluateFitnessResult, etc.)
- [x] Create `organism/calculate_region_key_impl.rs` with `todo!()`
- [x] Create `organism/evaluate_fitness_impl.rs` with `todo!()`
- [x] Create `organism/reproduce_impl.rs` with `todo!()`
- [x] Create `organism/increment_age_impl.rs` with `todo!()`
- [x] Create `organism/update_dimensions_impl.rs`
- [x] Implement `#[pool_item]` macro on Organism with custom `CreateOrganism` init type

**Note:** Due to `#[pool_item]` macro limitations (single impl block requirement), we use a delegation pattern:
- `mod.rs` contains struct, `#[pool_item]` impl block, and result types
- `*_impl.rs` files contain actual implementation logic as free functions
- Messaging methods delegate to these implementation functions

### 2.2 Dimensions Module Scaffold ✅

**Tasks:**
- [x] Create `dimensions/mod.rs` with Dimensions struct (with version field)
- [x] Create `dimensions/dimension.rs` (copied from lib1, with PartialEq)
- [x] Create `dimensions/expand_bounds.rs` with `todo!()`
- [x] Create `dimensions/calculate_dimensions_key.rs` with `todo!()`
- [x] Create `dimensions/new.rs` with `todo!()`

### 2.3 Regions Module Scaffold ✅

**Tasks:**
- [x] Create `regions/mod.rs` with Regions struct
- [x] Create `regions/region.rs` with Region struct
- [x] Create `regions/region_key.rs` (copied from lib1)
- [x] Create `regions/organism_entry.rs` with OrganismEntry struct
- [x] Create `regions/populate.rs` with `todo!()`
- [x] Create `regions/update_carrying_capacities.rs` with `todo!()`
- [x] Create `regions/process.rs` with `todo!()`

### 2.4 World Module Scaffold ✅

**Tasks:**
- [x] Create `world/mod.rs` with World struct
- [x] Create `world/world_struct.rs` with World implementation
- [x] Create `world/training_run.rs` with `todo!()`
- [x] Create `world/calculate_region_keys.rs` with `todo!()`
- [x] Create `world/evaluate_fitness.rs` with `todo!()`
- [x] Create `world/reproduction.rs` with `todo!()`
- [x] Create `world/age_and_cull.rs` with `todo!()`
- [x] Create `world/get_best_score.rs` with `todo!()`
- [x] Create `world/get_best_params.rs` with `todo!()`
- [x] Create `world/get_best_organism.rs` with `todo!()`
- [x] Create `world/setup_world.rs` with `todo!()`

**Acceptance Criteria:**
- [x] All modules compile (with `todo!()` bodies)
- [x] 158 unit tests pass, 59 ignored (pending implementation)
- [x] 13 doc tests pass, 8 ignored (pending World implementation)
- [x] `cargo clippy` clean (expected warnings for unused variables in todo bodies)

---

## Stage 3: Implement Leaf Components

These have no dependencies on other new components.

### 3.1 Dimensions (with versioning)

**Tasks:**
- [ ] Implement `Dimensions::new()` with initial version = 0
- [ ] Implement `Dimensions::version()` getter
- [ ] Implement `Dimensions::expand_bounds()` with version increment
- [ ] Implement `Dimensions::get_dimension()`
- [ ] Write tests for version increment behavior

**Tests:**
```rust
#[test]
fn given_new_dimensions_when_created_then_version_is_zero() { }

#[test]
fn given_dimensions_when_expand_bounds_then_version_increments() { }

#[test]
fn given_dimensions_when_expand_multiple_then_version_increments_each_time() { }
```

### 3.2 OrganismEntry

**Tasks:**
- [ ] Implement `OrganismEntry::new(id, age, score)`
- [ ] Implement ordering (by score, then by age descending)
- [ ] Write tests for ordering

**Tests:**
```rust
#[test]
fn given_entries_when_sorted_then_ordered_by_score_ascending() { }

#[test]
fn given_entries_with_same_score_when_sorted_then_older_first() { }
```

### 3.3 Reproduction Seed Derivation

**Tasks:**
- [ ] Implement `derive_reproduction_seed(world_seed, parent1_id, parent2_id)`
- [ ] Ensure deterministic output
- [ ] Write tests

**Tests:**
```rust
#[test]
fn given_same_inputs_when_derive_seed_then_same_output() { }

#[test]
fn given_different_parent_order_when_derive_seed_then_different_output() { }
```

---

## Stage 4: Implement Organism Pool Item

### 4.1 Organism Core

**Tasks:**
- [ ] Implement `Organism::new()` 
- [ ] Implement all getters
- [ ] Implement `IdTargeted` trait
- [ ] Write tests

### 4.2 Organism::calculate_region_key

**Tasks:**
- [ ] Implement incremental key calculation logic
- [ ] Handle out-of-bounds detection
- [ ] Return appropriate response variant
- [ ] Write tests for:
  - Fresh calculation (no existing key)
  - Incremental update (existing key, some dimensions changed)
  - Out-of-bounds detection (single dimension)
  - Out-of-bounds detection (multiple dimensions)
  - Dimension version matching

### 4.3 Organism::evaluate_fitness

**Tasks:**
- [ ] Implement fitness evaluation using world_function
- [ ] Store score on organism
- [ ] Return response with score, age, region_key
- [ ] Write tests

### 4.4 Organism::reproduce

**Tasks:**
- [ ] Implement reproduction using partner phenotype
- [ ] Use provided reproduction_seed for RNG
- [ ] Return two offspring phenotypes
- [ ] Write tests for determinism

### 4.5 Organism::increment_age

**Tasks:**
- [ ] Implement age increment
- [ ] Check against max_age from phenotype's system parameters
- [ ] Set is_dead if max age exceeded
- [ ] Return should_remove flag
- [ ] Write tests

### 4.6 Organism::update_dimensions

**Tasks:**
- [ ] Implement dimensions reference update
- [ ] Clear cached region_key
- [ ] Write tests

**Acceptance Criteria:**
- All organism methods implemented
- 100% test coverage on organism module
- `#[pool_item]` macro generates correct message types

---

## Stage 5: Implement Region Processing

### 5.1 Region Core

**Tasks:**
- [ ] Implement `Region::new()`
- [ ] Implement `Region::add_organism_entry()`
- [ ] Implement `Region::clear()`
- [ ] Implement min_score tracking
- [ ] Write tests

### 5.2 Region::process (Sort, Truncate, Pair)

**Tasks:**
- [ ] Implement sorting by OrganismEntry ordering
- [ ] Implement truncation to carrying capacity
- [ ] Implement extreme pairing logic
- [ ] Return reproduction pairs as `Vec<(u64, u64)>`
- [ ] Write tests

### 5.3 Regions Container

**Tasks:**
- [ ] Implement `Regions::new()`
- [ ] Implement `Regions::clear_all()`
- [ ] Implement `Regions::get_or_create_region()`
- [ ] Implement `Regions::iter()`
- [ ] Write tests

### 5.4 Carrying Capacity

**Tasks:**
- [ ] Implement `Regions::calculate_carrying_capacities()`
- [ ] Handle zero-score regions (infinite inverse fitness)
- [ ] Write tests

### 5.5 Parallel Region Processing

**Tasks:**
- [ ] Implement parallel processing via Rayon
- [ ] Collect reproduction pairs from all regions
- [ ] Write tests

**Acceptance Criteria:**
- All region methods implemented
- 100% test coverage on regions module

---

## Stage 6: Implement World Coordinator

### 6.1 World Core

**Tasks:**
- [ ] Implement `World::new()` (creates thread pool, initializes state)
- [ ] Implement organism ID tracking
- [ ] Implement best score/id tracking
- [ ] Write tests

### 6.2 World::calculate_region_keys

**Tasks:**
- [ ] Implement batch message sending
- [ ] Implement response collection and partitioning
- [ ] Implement dimension expansion on out-of-bounds
- [ ] Implement retry loop
- [ ] Send UpdateDimensions on change
- [ ] Write tests for:
  - All organisms fit (no retry)
  - Some organisms out of bounds (single retry)
  - Multiple dimension expansion (multiple retries)

### 6.3 World::evaluate_fitness

**Tasks:**
- [ ] Implement batch EvaluateFitnessRequest
- [ ] Track best score/id during collection
- [ ] Populate regions with OrganismEntry data
- [ ] Write tests

### 6.4 World::perform_reproduction

**Tasks:**
- [ ] Implement GetPhenotype requests
- [ ] Implement ReproduceRequest with deterministic seeds
- [ ] Create new organisms from offspring phenotypes
- [ ] Add new organism IDs to tracking
- [ ] Write tests for determinism

### 6.5 World::age_and_cull

**Tasks:**
- [ ] Implement batch IncrementAgeRequest
- [ ] Collect dead organism IDs
- [ ] Remove dead organisms from pool
- [ ] Update organism_ids list
- [ ] Write tests

### 6.6 World::training_run

**Tasks:**
- [ ] Orchestrate full training run flow
- [ ] Return resolution limit flag
- [ ] Write integration tests

**Acceptance Criteria:**
- Full training run works end-to-end
- Deterministic results with same seed

---

## Stage 7: Implement Public API

### 7.1 setup_world

**Tasks:**
- [ ] Implement `setup_world()` function
- [ ] Create initial population
- [ ] Match API signature from lib1
- [ ] Write tests

### 7.2 get_best_score

**Tasks:**
- [ ] Implement using cached best_score
- [ ] Return f64::MAX if no organisms scored
- [ ] Write tests

### 7.3 get_best_params

**Tasks:**
- [ ] Send GetPhenotype to best_organism_id
- [ ] Extract problem parameters from phenotype
- [ ] Write tests

### 7.4 get_best_organism

**Tasks:**
- [ ] Re-evaluate fitness with provided training data
- [ ] Return organism data (may need to query pool)
- [ ] Write tests

### 7.5 get_state (if needed for visualization)

**Tasks:**
- [ ] Implement state extraction
- [ ] Match lib1 format for server compatibility

**Acceptance Criteria:**
- Public API matches lib1 exactly
- All doc examples compile and pass

---

## Stage 8: Integration Tests

### 8.1 Determinism Tests

**Tasks:**
- [ ] Test same seed produces same results
- [ ] Test across multiple training runs
- [ ] Compare with lib1 results (where behavior should match)

### 8.2 Benchmark Function Tests

**Tasks:**
- [ ] Sphere function optimization
- [ ] Rosenbrock function optimization
- [ ] Himmelblau function optimization
- [ ] Ackley function optimization
- [ ] (Copy test patterns from lib1)

### 8.3 Edge Case Tests

**Tasks:**
- [ ] Single organism population
- [ ] Single dimension
- [ ] High dimensional (100+)
- [ ] Rapid dimension expansion

---

## Stage 9: Performance Validation

### 9.1 Benchmarks

**Tasks:**
- [ ] Create comparative benchmarks vs lib1
- [ ] Measure trivial function overhead
- [ ] Measure complex function improvement
- [ ] Document results

### 9.2 Profiling

**Tasks:**
- [ ] Profile message passing overhead
- [ ] Profile memory usage
- [ ] Identify any remaining bottlenecks

---

## Implementation Order Summary

```
Stage 0: Project Setup
    │
    ▼
Stage 1: Copy Unchanged Components
    │
    ▼
Stage 2: Scaffold All Modules
    │
    ▼
Stage 3: Leaf Components
    ├── Dimensions (versioned)
    ├── OrganismEntry
    └── Reproduction Seed
    │
    ▼
Stage 4: Organism Pool Item
    ├── Core + getters
    ├── calculate_region_key
    ├── evaluate_fitness
    ├── reproduce
    ├── increment_age
    └── update_dimensions
    │
    ▼
Stage 5: Region Processing
    ├── Region (sort, truncate, pair)
    ├── Regions container
    ├── Carrying capacity
    └── Parallel processing
    │
    ▼
Stage 6: World Coordinator
    ├── Core + tracking
    ├── calculate_region_keys
    ├── evaluate_fitness
    ├── perform_reproduction
    ├── age_and_cull
    └── training_run
    │
    ▼
Stage 7: Public API
    ├── setup_world
    ├── get_best_score
    ├── get_best_params
    └── get_best_organism
    │
    ▼
Stage 8: Integration Tests
    │
    ▼
Stage 9: Performance Validation
```

---

## Checkpoints

After each stage, verify:

1. **Compilation**: `cargo build -p hill_descent_lib2`
2. **Tests**: `cargo test -p hill_descent_lib2`
3. **Linting**: `cargo clippy -p hill_descent_lib2`
4. **Formatting**: `cargo fmt -p hill_descent_lib2`
5. **Coverage**: Verify 100% branch/condition coverage for completed modules

---

## Notes

### Copying vs. Rewriting

For modules copied from lib1 (phenotype, gamete, locus, etc.), we copy rather than share because:
1. Avoids cross-crate dependency complexity
2. Allows lib2-specific modifications if needed
3. Keeps both libraries fully independent

### Testing Strategy

- Unit tests in each module file
- Integration tests in `tests/` directory
- Use `#[cfg(test)]` helper functions following lib1 patterns
- Maintain test naming convention: `given_xxx_when_yyy_then_zzz`

### Incremental Validation

At stages 4, 5, and 6, write mini-integration tests that exercise the completed components together before moving on.
