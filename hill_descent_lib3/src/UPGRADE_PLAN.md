# Hill Descent Lib3 Storage Upgrade Plan

## Executive Summary

This document outlines a plan to improve the efficiency of `hill_descent_lib3` by changing the organism storage from `Vec<Arc<RwLock<Organism>>>` + `Vec<u64>` to a `HashMap<u64, Arc<RwLock<Organism>>>`. This change enables O(1) lookups by organism ID instead of O(n) scans, significantly improving performance in key operations like reproduction, removal, and ID-based access.

## Current State Analysis

### Current Storage Structure

```rust
pub struct World {
    pub(super) organisms: Vec<Arc<RwLock<Organism>>>,
    pub(super) organism_ids: Vec<u64>,
    // ... other fields
}
```

### Identified Inefficiencies

#### 1. **`perform_reproduction`** ([reproduction.rs](reproduction.rs#L15))
**Current approach:**
- Receives a `Vec<(u64, u64)>` of parent ID pairs
- Scans entire `organisms` vector to find matching IDs: O(n) per unique parent
- Uses `par_iter().filter_map()` to find phenotypes for parents
- Creates temporary `HashMap<u64, Arc<Phenotype>>` for lookup
- Uses intermediate `CreateOrganism` struct unnecessarily

**Issues:**
- O(n) search through all organisms just to find parents
- Double storage overhead: both `Vec<Organism>` and `HashMap<u64, Phenotype>`
- `CreateOrganism` struct is boilerplate that could be eliminated

#### 2. **`remove_organisms`** ([training_run.rs](training_run.rs#L64))
**Current approach:**
```rust
fn remove_organisms(&mut self, ids: &[u64]) {
    self.organisms.retain(|o| !ids.contains(&o.read().unwrap().id()));
    self.organism_ids.retain(|id| !ids.contains(id));
}
```

**Issues:**
- O(n * m) complexity where n = organism count, m = IDs to remove
- Must lock each organism just to read its ID
- Retains items by scanning the entire vector

#### 3. **`process_epoch_all`** ([process_epoch.rs](process_epoch.rs#L17))
**Current approach:**
- Creates a `FxHashMap<u64, usize>` index at start of every call
- Uses this for O(1) lookup of best organism by ID

**Issues:**
- Rebuilds the index every single epoch
- If organisms were in a HashMap, this would be unnecessary

#### 4. **`get_state_for_web`** ([get_state_for_web.rs](get_state_for_web.rs#L89))
**Current approach:**
- Uses `par_iter()` over all organisms to build state

**Issues:**
- Minor: With HashMap, iteration still works but order is non-deterministic
- This is acceptable for web state (order doesn't matter for visualization)

#### 5. **Redundant `organism_ids` Vector**
The separate `organism_ids: Vec<u64>` exists because getting IDs from the Vec storage requires locking each organism. With a HashMap, the keys ARE the IDs, eliminating this redundancy.

---

## Proposed Solution

### New Storage Structure

```rust
use indexmap::IndexMap;

pub struct World {
    pub(super) organisms: IndexMap<u64, Arc<RwLock<Organism>>>,
    // organism_ids removed - IndexMap keys serve this purpose
    // ... other fields
}
```

**Benefits:**
- O(1) lookup by ID
- No separate ID tracking needed
- Natural removal via `IndexMap::shift_remove()` (preserves insertion order)
- Keys can be iterated for ID list when needed
- **Deterministic iteration order** (maintains insertion order)

**Note:** We use `IndexMap` instead of `FxHashMap` because deterministic iteration order is critical for reproducible results with the same seed.

---

## Implementation Plan

### Phase 1: Change Storage Type (Low Risk, High Impact) ✅ COMPLETED

#### Step 1.1: Update `world_struct.rs` ✅
- Changed `organisms: Vec<Arc<RwLock<Organism>>>` to `organisms: IndexMap<u64, Arc<RwLock<Organism>>>`
- Removed `organism_ids: Vec<u64>`
- Updated `organism_count()` to use `self.organisms.len()`
- Added helper method `organism_ids(&self) -> Vec<u64>` that returns keys when needed

**Files affected:** 
- [world_struct.rs](world_struct.rs)

#### Step 1.2: Update `new.rs` ✅
- Changed organism initialization to insert into IndexMap instead of push to Vec
- Removed `organism_ids` initialization

**Files affected:**
- [new.rs](new.rs)

#### Additional Changes Made in Phase 1:
- **[training_run.rs](training_run.rs)**: Changed `remove_organisms` to use `shift_remove()` for O(m) removal
- **[process_epoch.rs](process_epoch.rs)**: Removed redundant `organism_index` HashMap creation, updated par_iter
- **[reproduction.rs](reproduction.rs)**: Updated par_iter and organism insertion
- **[get_state_for_web.rs](get_state_for_web.rs)**: Updated par_iter for IndexMap
- **[get_best_organism.rs](get_best_organism.rs)**: Changed to use direct `get()` lookup instead of `iter().find()`

### Phase 2: Simplify Reproduction (High Impact) ✅ COMPLETED

#### Step 2.1: Refactor `perform_reproduction` ✅

The reproduction function was significantly simplified:

**Before (inefficient):**
- Collected unique parent IDs into a Vec
- Did O(n) parallel scan of ALL organisms to find parent phenotypes
- Built an intermediate `HashMap<u64, Arc<Phenotype>>` for lookup
- Used `ReproduceResult` struct for intermediate results
- Used `CreateOrganism` struct for organism creation

**After (efficient):**
- Direct O(1) lookups via `self.organisms.get(&id)`
- No intermediate HashMap needed
- Uses new `Organism::new_direct()` constructor
- Still maintains parallel reproduction for the actual crossover/mutation
- Sequential ID assignment preserved for determinism

#### Step 2.2: Add `Organism::new_direct()` method ✅

Added a more ergonomic constructor that takes individual parameters:

```rust
pub fn new_direct(
    id: u64,
    parent_ids: (Option<u64>, Option<u64>),
    phenotype: Arc<Phenotype>,
    dimensions: Arc<Dimensions>,
    world_function: Arc<dyn WorldFunction + Send + Sync>,
) -> Self
```

**Note:** `CreateOrganism` struct is kept for backward compatibility (still used in `new.rs`).

**Key improvements achieved:**
- Direct O(1) parent lookup instead of O(n) scan
- No intermediate phenotype HashMap
- Cleaner, more readable code
- Sequential ID assignment preserved for determinism
- Simpler, more readable code

**Files affected:**
- [reproduction.rs](reproduction.rs)

### Phase 3: Simplify Organism Removal (Medium Impact)

#### Step 3.1: Refactor `remove_organisms`

```rust
fn remove_organisms(&mut self, ids: &[u64]) {
    for id in ids {
        self.organisms.remove(id);
    }
}
```

**Key improvements:**
- O(m) complexity (m = number of IDs to remove)
- No locking required - HashMap remove is by key
- Dramatically simpler

**Files affected:**
- [training_run.rs](training_run.rs)

### Phase 4: Simplify Epoch Processing (Medium Impact)

#### Step 4.1: Update `process_epoch_all`
- Remove the `organism_index` HashMap creation at the start
- Direct lookups use `self.organisms.get(&id)`

```rust
// OLD: Created every epoch
let organism_index: FxHashMap<u64, usize> = self
    .organism_ids
    .iter()
    .enumerate()
    .map(|(idx, &id)| (id, idx))
    .collect();

// NEW: Direct lookup
if let Some(org_lock) = self.organisms.get(&best_id) {
    let org = org_lock.read().unwrap();
    // ...
}
```

**Files affected:**
- [process_epoch.rs](process_epoch.rs)

### Phase 5: Update Remaining Code (Low Risk)

#### Step 5.1: Update `get_state_for_web`
- Change `self.organisms.par_iter()` to `self.organisms.par_iter()` (works on HashMap)
- Update the mapping to handle HashMap entries `(k, v)` instead of direct values

#### Step 5.2: Review and update any other usages
- Grep for all `self.organisms` usages
- Update iteration patterns as needed

**Files affected:**
- [get_state_for_web.rs](get_state_for_web.rs)

### Phase 6: Remove `CreateOrganism` Struct (Cleanup) ✅ COMPLETED

#### Step 6.1: Simplify `Organism::new()` ✅

The `new_direct()` method was renamed to simply `new()` as the only constructor:

```rust
impl Organism {
    pub fn new(
        id: u64,
        parent_ids: (Option<u64>, Option<u64>),
        phenotype: Arc<Phenotype>,
        dimensions: Arc<Dimensions>,
        world_function: Arc<dyn WorldFunction + Send + Sync>,
    ) -> Self
```

#### Step 6.2: Removed `CreateOrganism` struct ✅
- Removed the `CreateOrganism` struct entirely
- Updated `new.rs` to use `Organism::new()` directly
- `reproduction.rs` already used the direct constructor

**Files affected:**
- [organism/mod.rs](../organism/mod.rs) - Removed `CreateOrganism`, kept `new()` with direct parameters
- [new.rs](new.rs) - Updated to use `Organism::new()`
- [reproduction.rs](reproduction.rs) - Already using direct constructor

---

## Determinism Considerations

**Critical:** lib3 must produce identical results to lib2 for the same seed.

### Preserved Determinism:
- **ID assignment**: Sequential IDs from `next_organism_id` - unchanged
- **Reproduction order**: Process pairs in order - maintained  
- **RNG usage**: Same seed derivation - unchanged

### Areas to Verify:
- **Parallel iteration order**: `par_iter()` on HashMap doesn't guarantee order
  - For `process_epoch_all`: Order doesn't matter (aggregates results)
  - For `get_state_for_web`: Order doesn't matter (visualization only)
  - **Ensure no operations depend on iteration order**

---

## Testing Strategy

### Unit Tests
1. Verify `organism_count()` returns correct value after add/remove
2. Verify organism lookup by ID works
3. Verify reproduction creates correct offspring with correct IDs

### Integration Tests  
1. Run `golden_determinism_test.rs` - must pass unchanged
2. Run all existing tests in `tests/` directory
3. Compare lib2 and lib3 output for same seed/parameters

### Benchmarks
1. Run `hundred_parameter_benchmark.rs` before and after
2. Compare `perform_reproduction` timings specifically
3. Measure `process_epoch_all` timing improvement

---

## Rollout Checklist

- [x] Phase 1: Storage type change ✅
  - [x] Update `world_struct.rs`
  - [x] Update `new.rs`
  - [x] All tests pass
  
- [x] Phase 2: Reproduction refactor ✅
  - [x] Refactor `perform_reproduction`
  - [x] All tests pass
  - [x] Golden determinism verified
  
- [x] Phase 3: Removal refactor ✅ (Done in Phase 1)
  - [x] Refactor `remove_organisms`
  - [x] All tests pass
  
- [x] Phase 4: Epoch processing ✅ (Done in Phase 1)
  - [x] Update `process_epoch_all`
  - [x] All tests pass
  
- [x] Phase 5: Remaining updates ✅ (Done in Phase 1)
  - [x] Update `get_state_for_web`
  - [x] Review all `self.organisms` usages
  - [x] All tests pass
  
- [x] Phase 6: Cleanup ✅
  - [x] Simplify `Organism::new()` (removed `new_direct`, kept just `new`)
  - [x] Remove `CreateOrganism` struct
  - [x] All tests pass
  
- [x] Final verification ✅
  - [x] `cargo fmt`
  - [x] `cargo clippy --workspace` (zero warnings)
  - [x] `cargo clippy --tests` (zero warnings)
  - [x] `cargo test --workspace`
  - [x] Run benchmarks, document improvements

---

## Benchmark Results (January 12, 2026)

### 100-Dimensional Parameter Benchmark

| Library | Time per Epoch | Relative Performance |
|---------|---------------|---------------------|
| lib2    | ~12.1 ms      | baseline            |
| lib3    | ~2.0 ms       | **~6x faster**      |

The lib3 implementation is approximately **6 times faster** than lib2 for the 100-dimensional benchmark.

### Performance Improvements Achieved

1. **IndexMap storage** - O(1) lookups instead of O(n) scans for parent phenotypes
2. **Simplified reproduction** - Direct parent lookups without intermediate HashMaps
3. **Efficient removal** - `shift_remove()` at O(m) instead of O(n*m) with `retain()`
4. **Eliminated redundancy** - No more `organism_index` HashMap rebuilt each epoch
5. **Cleaner API** - Removed `CreateOrganism` struct boilerplate

---

## Expected Benefits

| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Parent lookup in reproduction | O(n) per parent | O(1) | ~100x for large populations |
| Organism removal | O(n * m) | O(m) | ~n times faster |
| Epoch index creation | O(n) every epoch | O(1) | Eliminated entirely |
| Code complexity | High (multiple data structures) | Low (single HashMap) | Significant reduction |
| Memory | Two collections | One collection | ~50% reduction in overhead |

---

## Alternative Considered

**Keep Vec but add separate HashMap<u64, usize> for index:**
- Still requires keeping two data structures in sync
- Insertion/removal must update both
- More error-prone
- **Rejected** in favor of single HashMap approach

---

## Conclusion

Converting from `Vec<Arc<RwLock<Organism>>>` + `Vec<u64>` to `FxHashMap<u64, Arc<RwLock<Organism>>>` will:

1. **Dramatically improve performance** in reproduction, removal, and lookup operations
2. **Simplify the codebase** by removing redundant data structures
3. **Reduce memory overhead** by eliminating the separate ID vector
4. **Enable cleaner APIs** by removing the `CreateOrganism` boilerplate struct

The change is low-risk because:
- HashMap provides the same capabilities as Vec for our use cases
- Iteration order is not relied upon for correctness
- Determinism is preserved through ID assignment and RNG seeding
- Comprehensive test coverage exists to verify behavior

Estimated implementation effort: **2-3 hours** for a careful, phased implementation with verification at each step.
