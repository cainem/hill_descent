# Performance Fix Plan: Eliminate Mutex Lock Contention

**Branch**: `copilot/implement-thread-per-region-parallelization`  
**Issue**: 2.2x performance REGRESSION instead of 2-5x improvement  
**Root Cause**: Mutex lock contention in `Organism` structure  
**Expected Outcome**: 5-10x speedup for small populations, 2-5x for large populations

---

## Context & Problem Statement

PR #10 implemented thread-per-region parallelization but benchmarks reveal a **2.2x slowdown** instead of the expected 2-5x speedup. Root cause analysis identified Mutex lock contention as the bottleneck:

### Current Implementation (Problematic)
```rust
// hill_descent_lib/src/world/organisms/organism/mod.rs (lines 18-32)
pub struct Organism {
    region_key: Mutex<Option<Vec<usize>>>,  // ⚠️ MUTEX 1
    score: Mutex<Option<f64>>,              // ⚠️ MUTEX 2
    age: AtomicUsize,                       // ✓ Lock-free
    is_dead: AtomicBool,                    // ✓ Lock-free
    phenotype: Arc<Phenotype>,
}
```

### Lock Contention Impact
- **Pop:10**: ~233 mutex locks/iteration → 5-9x slower than master
- **Pop:100**: ~66,400 mutex locks/iteration → 2-3x slower than master  
- **Pop:10000**: ~13.2M mutex locks/iteration → 1.2-1.6x faster (work finally dominates overhead)

### Critical Paths with Locks
1. **Fitness evaluation** (lines 17-19 in `process_region.rs`): 1 lock per organism
2. **Sorting** (lines 22-28 in `process_region.rs`): 2 locks per comparison = O(N log N) locks
3. **Region key access**: Additional lock overhead during organism management

---

## Implementation Tasks

### ✅ Prerequisites (Already Complete)
- All tests passing (430/430)
- Zero clippy warnings
- Code properly formatted
- Dead code removed
- Branch: `copilot/implement-thread-per-region-parallelization` is clean

---

## Solution 1: Replace Mutex<Option<f64>> with AtomicU64 for Score

**Priority**: HIGHEST  
**Effort**: ~2 hours  
**Expected Impact**: 5-10x speedup for Pop:10-100, enables parallel sorting

### Files to Modify

#### 1. `hill_descent_lib/src/world/organisms/organism/mod.rs`

**Current (lines 18-32)**:
```rust
pub struct Organism {
    region_key: Mutex<Option<Vec<usize>>>,
    score: Mutex<Option<f64>>,              // ← CHANGE THIS
    age: AtomicUsize,
    is_dead: AtomicBool,
    phenotype: Arc<Phenotype>,
}
```

**New**:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Organism {
    region_key: Mutex<Option<Vec<usize>>>,
    score: AtomicU64,                       // ← NEW: Store f64::to_bits(), u64::MAX = None
    age: AtomicUsize,
    is_dead: AtomicBool,
    phenotype: Arc<Phenotype>,
}
```

#### 2. Update `Organism::new()` in same file

**Find (around line 40)**:
```rust
pub fn new(
    phenotype: Arc<Phenotype>,
    age: usize,
    initial_state: (Option<Vec<usize>>, Option<f64>),
) -> Self {
    let (region_key, score) = initial_state;
    Self {
        region_key: Mutex::new(region_key),
        score: Mutex::new(score),           // ← CHANGE THIS
        age: AtomicUsize::new(age),
        is_dead: AtomicBool::new(false),
        phenotype,
    }
}
```

**Replace with**:
```rust
pub fn new(
    phenotype: Arc<Phenotype>,
    age: usize,
    initial_state: (Option<Vec<usize>>, Option<f64>),
) -> Self {
    let (region_key, score) = initial_state;
    let score_bits = score.map(|s| s.to_bits()).unwrap_or(u64::MAX);
    Self {
        region_key: Mutex::new(region_key),
        score: AtomicU64::new(score_bits),  // ← NEW
        age: AtomicUsize::new(age),
        is_dead: AtomicBool::new(false),
        phenotype,
    }
}
```

#### 3. `hill_descent_lib/src/world/organisms/organism/score.rs`

**Current getter**:
```rust
pub fn score(&self) -> Option<f64> {
    *self.score.lock().unwrap()
}
```

**New getter** (lock-free):
```rust
/// Returns the organism's fitness score if it has been evaluated.
/// Uses atomic operations for lock-free concurrent access.
pub fn score(&self) -> Option<f64> {
    let bits = self.score.load(Ordering::Acquire);
    if bits == u64::MAX {
        None
    } else {
        Some(f64::from_bits(bits))
    }
}
```

**Current setter**:
```rust
pub fn set_score(&self, score: Option<f64>) {
    *self.score.lock().unwrap() = score;
}
```

**New setter** (lock-free):
```rust
/// Sets the organism's fitness score using atomic operations.
/// Thread-safe without locks, allowing concurrent access during parallel processing.
pub fn set_score(&self, score: Option<f64>) {
    let bits = score.map(|s| s.to_bits()).unwrap_or(u64::MAX);
    self.score.store(bits, Ordering::Release);
}
```

#### 4. Update `Clone` implementation (if exists in test-only code)

**Find** (likely in `organism/mod.rs` under `#[cfg(test)]`):
```rust
impl Clone for Organism {
    fn clone(&self) -> Self {
        Self {
            region_key: Mutex::new(self.region_key.lock().unwrap().clone()),
            score: Mutex::new(*self.score.lock().unwrap()),  // ← CHANGE THIS
            // ...
        }
    }
}
```

**Replace with**:
```rust
impl Clone for Organism {
    fn clone(&self) -> Self {
        Self {
            region_key: Mutex::new(self.region_key.lock().unwrap().clone()),
            score: AtomicU64::new(self.score.load(Ordering::Acquire)),  // ← NEW
            // ...
        }
    }
}
```

### Testing Solution 1

**Tests to Add** in `hill_descent_lib/src/world/organisms/organism/score.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;

    fn create_test_organism() -> Organism {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        Organism::new(phenotype, 0, (None, None))
    }

    #[test]
    fn given_new_organism_when_score_then_returns_none() {
        let org = create_test_organism();
        assert_eq!(org.score(), None);
    }

    #[test]
    fn given_score_set_when_get_score_then_returns_same_value() {
        let org = create_test_organism();
        org.set_score(Some(42.5));
        assert_eq!(org.score(), Some(42.5));
    }

    #[test]
    fn given_score_set_to_none_when_get_score_then_returns_none() {
        let org = create_test_organism();
        org.set_score(Some(10.0));
        org.set_score(None);
        assert_eq!(org.score(), None);
    }

    #[test]
    fn given_negative_score_when_set_then_preserved() {
        let org = create_test_organism();
        org.set_score(Some(-123.456));
        assert_eq!(org.score(), Some(-123.456));
    }

    #[test]
    fn given_zero_score_when_set_then_preserved() {
        let org = create_test_organism();
        org.set_score(Some(0.0));
        assert_eq!(org.score(), Some(0.0));
    }

    #[test]
    fn given_very_small_score_when_set_then_preserved() {
        let org = create_test_organism();
        let small_value = f64::EPSILON;
        org.set_score(Some(small_value));
        assert_eq!(org.score(), Some(small_value));
    }

    #[test]
    fn given_concurrent_reads_when_score_set_then_all_see_value() {
        use std::sync::Arc;
        use std::thread;

        let org = Arc::new(create_test_organism());
        org.set_score(Some(99.9));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let org_clone = Arc::clone(&org);
                thread::spawn(move || org_clone.score())
            })
            .collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), Some(99.9));
        }
    }
}
```

### Validation After Solution 1

Run these commands:
```powershell
cargo fmt
cargo test --workspace
cargo clippy --workspace  # Must be 0 warnings
```

**Expected test results**: All 430+ tests pass (existing + new score tests)

---

## Solution 2: Cache Scores Before Sorting

**Priority**: MEDIUM (do after Solution 1)  
**Effort**: ~1 hour  
**Expected Impact**: Further eliminate remaining score access overhead

### File to Modify

#### `hill_descent_lib/src/world/regions/region/process_region.rs`

**Current (lines 22-28)**:
```rust
// 2. Sort by fitness (best first) then age (older first)
self.organisms.sort_by(|a, b| {
    let score_cmp = a
        .score()
        .unwrap_or(f64::INFINITY)
        .partial_cmp(&b.score().unwrap_or(f64::INFINITY))
        .unwrap_or(std::cmp::Ordering::Equal);
    score_cmp.then_with(|| b.age().cmp(&a.age()))
});
```

**New (cache scores once)**:
```rust
// 2. Sort by fitness (best first) then age (older first)
// Cache scores once to avoid repeated atomic loads during sort
let mut organisms_with_scores: Vec<_> = self.organisms
    .iter()
    .map(|org| (Arc::clone(org), org.score().unwrap_or(f64::INFINITY), org.age()))
    .collect();

organisms_with_scores.sort_by(|a, b| {
    let score_cmp = a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal);
    score_cmp.then_with(|| b.2.cmp(&a.2))
});

self.organisms = organisms_with_scores.into_iter().map(|(org, _, _)| org).collect();
```

### Testing Solution 2

Add test in `process_region.rs`:
```rust
#[test]
fn given_unsorted_organisms_when_process_lifecycle_then_sorted_by_score_and_age() {
    let mut region = Region::new();
    region.set_carrying_capacity(Some(10));
    
    // Add organisms with known scores and ages
    let org1 = create_test_organism(5);  // age 5
    let org2 = create_test_organism(3);  // age 3
    let org3 = create_test_organism(7);  // age 7
    
    region.add_organism(org1);
    region.add_organism(org2);
    region.add_organism(org3);
    
    // Manually set scores: org1=3.0, org2=1.0 (best), org3=2.0
    region.organisms()[0].set_score(Some(3.0));
    region.organisms()[1].set_score(Some(1.0));
    region.organisms()[2].set_score(Some(2.0));
    
    region.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
    
    // Verify sorted: best score first, then by age
    let orgs = region.organisms();
    assert_eq!(orgs[0].score(), Some(1.0));  // Best score
    assert_eq!(orgs[1].score(), Some(2.0));  // Second best
    assert_eq!(orgs[2].score(), Some(3.0));  // Worst score
}
```

---

## Solution 3: Parallelize Collection Phase

**Priority**: HIGH (quick win)  
**Effort**: ~30 minutes  
**Expected Impact**: 2-3x additional speedup for small populations

### File to Modify

#### `hill_descent_lib/src/world/regions/parallel_process.rs`

**Current serial collection (lines 27-44)**:
```rust
// SERIAL: Collect all organisms
let mut all_organisms: Vec<Arc<Organism>> = Vec::new();
for (_key, region) in self.regions.iter() {
    for organism in region.organisms() {
        all_organisms.push(Arc::clone(organism));
    }
}

// Clear regions for redistribution
for (_key, region) in self.regions.iter_mut() {
    region.clear_organisms();
}

// Flatten and add offspring
all_organisms.extend(all_offspring.into_iter()
    .flat_map(|v| v.into_iter().map(Arc::new)));
```

**New parallel collection**:
```rust
// PARALLEL: Collect survivors from all regions
let survivors: Vec<Arc<Organism>> = self.regions
    .par_iter()
    .flat_map(|(_key, region)| {
        region.organisms().iter().map(Arc::clone)
    })
    .collect();

// Clear regions for redistribution (must be serial - uses iter_mut)
for (_key, region) in self.regions.iter_mut() {
    region.clear_organisms();
}

// PARALLEL: Flatten offspring
let offspring_flat: Vec<Arc<Organism>> = all_offspring
    .into_par_iter()
    .flat_map(|v| v.into_iter().map(Arc::new))
    .collect();

// Combine survivors and offspring
let mut all_organisms = survivors;
all_organisms.extend(offspring_flat);
```

### Testing Solution 3

Existing determinism tests should cover this - just verify they still pass.

---

## Solution 4: Optimize region_key Mutex

**Priority**: LOW (minor impact)  
**Effort**: ~30 minutes  
**Expected Impact**: Small additional improvement

### Analysis

The `region_key` field is:
- **Set once** during organism assignment (in `update` phase - serial)
- **Read occasionally** during organism management
- **Not accessed** during parallel processing phase

### Option A: Keep Mutex (simplest)

Leave as-is since it's not in the critical path.

### Option B: Convert to AtomicU64 Hash (if needed)

Similar to score, hash the region_key to a u64 and use AtomicU64. Only pursue if profiling shows it's still a bottleneck.

---

## Validation & Benchmarking

### After Completing All Solutions

1. **Format and lint**:
   ```powershell
   cargo fmt
   cargo clippy --workspace  # Must be 0 warnings
   ```

2. **Run all tests**:
   ```powershell
   cargo test --workspace  # Must be 430+ tests passing
   ```

3. **Run benchmarks**:
   ```powershell
   cd hill_descent_benchmarks
   cargo run --release
   ```

4. **Compare performance**:
   - Expected: **5-10x speedup** for Pop:10-100
   - Expected: **2-5x speedup** for Pop:500+
   - Baseline: Run stats in `run_stats/2025-10/04d8271b-08/` (master branch)
   - Current: Run stats in `run_stats/2025-10/ed9df9d7-15/` (before fix)
   - New: Will create new timestamped directory

5. **Use comparison script**:
   ```powershell
   cd run_stats/2025-10
   python benchmark_comparison.py
   ```

### Success Criteria

- ✅ All tests pass (430+)
- ✅ Zero clippy warnings
- ✅ Determinism tests pass (same seed = same results)
- ✅ **Pop:10 speedup**: 5-10x vs current (0.11x-0.2x) → target 0.8x-1.5x vs master
- ✅ **Pop:100 speedup**: 3-5x vs current (0.3x-0.4x) → target 2x-3x vs master
- ✅ **Pop:10000 speedup**: Maintain or improve current 1.2-1.6x vs master

---

## Commit Strategy

### Commit 1: Solution 1 (AtomicU64 for score)
```
perf: Replace Mutex<Option<f64>> with AtomicU64 for lock-free score access

Eliminates mutex lock contention in fitness evaluation and sorting operations.
Expected 5-10x speedup for small populations (Pop:10-100).

Changes:
- Convert Organism::score from Mutex<Option<f64>> to AtomicU64
- Use f64::to_bits()/from_bits() for atomic storage (u64::MAX = None)
- Update getter/setter to use Ordering::Acquire/Release
- Add comprehensive tests for atomic score operations including concurrent access

Performance Impact:
- Eliminates ~233 locks/iteration for Pop:10
- Eliminates ~66,400 locks/iteration for Pop:100
- Enables lock-free sorting: O(N log N) comparisons with zero locks

Tests: All 430+ tests pass, zero clippy warnings
```

### Commit 2: Solution 2 (Cache scores)
```
perf: Cache scores before sorting to eliminate repeated atomic loads

Pre-fetches all scores once before sorting to minimize atomic operations.

Changes:
- Build (organism, score, age) tuples before sorting
- Sort using cached values instead of repeated score() calls
- Add test for sort correctness with mixed scores and ages

Performance Impact: Reduces atomic operations during O(N log N) sort
Tests: All tests pass, zero clippy warnings
```

### Commit 3: Solution 3 (Parallel collection)
```
perf: Parallelize organism collection phase

Converts serial collection of survivors and offspring to parallel operations.

Changes:
- Use par_iter() for collecting survivors from regions
- Use into_par_iter() for flattening offspring
- Keep region.clear_organisms() serial (requires iter_mut)

Performance Impact: 2-3x additional speedup for small populations
Tests: Determinism tests pass, zero clippy warnings
```

### Commit 4: Benchmark results
```
chore: Add performance benchmark comparison after lock contention fixes

Documents performance improvements from atomic score implementation.

Results:
- Pop:10: [INSERT ACTUAL SPEEDUP]x vs baseline
- Pop:100: [INSERT ACTUAL SPEEDUP]x vs baseline
- Pop:10000: [INSERT ACTUAL SPEEDUP]x vs baseline

Benchmark files:
- run_stats/2025-10/[NEW_TIMESTAMP]/
- run_stats/2025-10/PERFORMANCE_COMPARISON.md
```

---

## Troubleshooting

### Problem: AtomicU64 ordering issues
**Symptom**: Non-deterministic test failures  
**Solution**: Ensure using `Ordering::Acquire` for loads, `Ordering::Release` for stores

### Problem: f64 precision loss
**Symptom**: Scores slightly different after round-trip  
**Solution**: `f64::to_bits()` and `from_bits()` are lossless - verify not comparing NaN

### Problem: Performance not improved
**Symptom**: Benchmarks still slow after changes  
**Solution**: 
1. Verify AtomicU64 actually being used (check with `grep -r "Mutex<Option<f64>>"`)
2. Ensure running `--release` mode benchmarks
3. Profile with flamegraph to find remaining bottlenecks

### Problem: Clippy warnings about atomic ordering
**Symptom**: Warnings about Ordering::Relaxed  
**Solution**: Use Acquire/Release for correctness - Relaxed insufficient for cross-thread synchronization

---

## References

- **Root cause analysis**: `LOCK_CONTENTION_ANALYSIS.md`
- **Benchmark comparison script**: `run_stats/2025-10/benchmark_comparison.py`
- **AGENTS.md**: Development standards (must have 0 clippy warnings)
- **Rust Atomics**: https://doc.rust-lang.org/std/sync/atomic/
- **f64::to_bits()**: https://doc.rust-lang.org/std/primitive.f64.html#method.to_bits

---

## Implementation Sequence

1. **Start Fresh Chat**: Provide this file + context about branch state
2. **Solution 1**: Implement AtomicU64 for score (~2 hours)
3. **Test & Validate**: Ensure all tests pass, 0 warnings
4. **Commit 1**: Atomic score implementation
5. **Solution 2**: Cache scores in sorting (~1 hour)
6. **Test & Validate**: All tests pass
7. **Commit 2**: Cached scoring
8. **Solution 3**: Parallel collection (~30 min)
9. **Test & Validate**: Determinism tests pass
10. **Commit 3**: Parallel collection
11. **Benchmark**: Run full benchmark suite
12. **Commit 4**: Document results
13. **Push**: Push all commits to remote

---

**End of Plan**
