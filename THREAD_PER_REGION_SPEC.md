# Thread-Per-Region Parallelization Implementation Specification

**Target Branch**: master | **Effort**: 3-5 days | **Expected**: 2-5x speedup | **Risk**: Medium

## Executive Summary

Convert from fine-grained (per-organism) to coarse-grained (per-region) parallelism. Each region executes its complete lifecycle on a dedicated thread, with synchronization only for global operations.

**Benefits**: Natural parallelization boundary, reduced coordination overhead, better cache locality, per-region RNG for determinism, scales with region count.

## Success Criteria

1. All existing tests pass | 2. Deterministic (same seed = same results) | 3. 2-5x speedup for Pop 500+ | 4. No API breaks | 5. Zero clippy warnings | 6. cargo fmt compliant

## Architecture

**Current**: Serial processing, `Vec<Rc<Organism>>`, 8 separate steps  
**New**: Parallel per-region processing, `Vec<Arc<Organism>>`, single parallel phase + sync phase

```
PARALLEL: Fitness → Sort → Truncate → Cull → Reproduce → Age → Cull
SYNC: Collect organisms → Update dimensions → Redistribute → Update capacities
```

---

## Implementation Tasks

### Task 1: Add Dependencies

**File**: `hill_descent_lib/Cargo.toml`

```toml
rayon = "1.10"
indexmap = { version = "2.11", features = ["serde", "rayon"] }
```

### Task 2: Convert Rc → Arc

Replace in all files (32 total):
- `std::rc::Rc` → `std::sync::Arc`
- `Rc::new` → `Arc::new`  
- `Rc::clone` → `Arc::clone`
- `Rc::ptr_eq` → `Arc::ptr_eq`

Find files: `Get-ChildItem -Path "hill_descent_lib/src" -Recurse -Filter "*.rs" | Select-String -Pattern "use std::rc::Rc"`

### Task 3: Add Sync Trait Bounds

**Files**: `world_function.rs`, `single_valued_function.rs`

```rust
pub trait WorldFunction: Debug + Sync { ... }
pub trait SingleValuedFunction: Debug + Sync { ... }
```


### Task 4: Per-Region RNG Derivation

**New File**: `hill_descent_lib/src/world/regions/derive_region_seed.rs`

```rust
use xxhash_rust::xxh3::xxh3_64;

/// Derives deterministic seed for region from world seed + region key.
/// Same world seed + region key = same RNG, different regions = independent streams.
pub fn derive_region_seed(world_seed: u64, region_key: &[usize]) -> u64 {
    let mut hasher_input = world_seed.to_le_bytes().to_vec();
    for &idx in region_key {
        hasher_input.extend_from_slice(&(idx as u64).to_le_bytes());
    }
    xxh3_64(&hasher_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_same_world_seed_and_key_when_derive_then_same_result() {
        assert_eq!(derive_region_seed(12345, &[0, 1, 2]), derive_region_seed(12345, &[0, 1, 2]));
    }

    #[test]
    fn given_different_world_seeds_when_derive_then_different_results() {
        assert_ne!(derive_region_seed(12345, &[0, 1, 2]), derive_region_seed(67890, &[0, 1, 2]));
    }

    #[test]
    fn given_different_region_keys_when_derive_then_different_results() {
        assert_ne!(derive_region_seed(12345, &[0, 1, 2]), derive_region_seed(12345, &[0, 1, 3]));
    }

    #[test]
    fn given_empty_region_key_when_derive_then_returns_valid_seed() {
        assert_ne!(derive_region_seed(12345, &[]), 0);
    }

    #[test]
    fn given_large_region_key_when_derive_then_returns_valid_seed() {
        let large_key: Vec<usize> = (0..100).collect();
        assert_ne!(derive_region_seed(12345, &large_key), 0);
    }
}
```

**Update module**: `hill_descent_lib/src/world/regions/mod.rs` add:
```rust
mod derive_region_seed;
pub use derive_region_seed::derive_region_seed;
```


### Task 5: Region Lifecycle Processing

**New File**: `hill_descent_lib/src/world/regions/region/process_region.rs`

```rust
use super::Region;
use crate::world::organisms::organism::Organism;
use crate::world::world_function::WorldFunction;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;

impl Region {
    /// Processes region's complete lifecycle independently (designed for parallel execution).
    /// Operations: Fitness → Sort → Truncate → Cull → Reproduce → Age → Cull
    pub fn process_region_lifecycle(
        &mut self,
        world_function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: Option<&[f64]>,
        region_seed: u64,
    ) -> Vec<Organism> {
        // 1. Fitness evaluation
        for organism in self.organisms.iter() {
            organism.run(world_function, inputs, known_outputs);
        }

        // 2. Sort by fitness (best first) then age (older first)
        self.organisms.sort_by(|a, b| {
            let score_cmp = a.score().unwrap_or(f64::INFINITY)
                .partial_cmp(&b.score().unwrap_or(f64::INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal);
            score_cmp.then_with(|| b.age().cmp(&a.age()))
        });

        // 3. Truncate to capacity
        if let Some(capacity) = self.carrying_capacity {
            if self.organism_count() > capacity {
                for organism in self.organisms.iter().skip(capacity) {
                    organism.mark_dead();
                }
            }
        }

        // 4. Remove dead
        self.organisms.retain(|org| !org.is_dead());

        // 5. Reproduce offspring
        let offspring = if let Some(capacity) = self.carrying_capacity {
            let current = self.organism_count();
            if current < capacity {
                let mut region_rng = StdRng::seed_from_u64(region_seed);
                self.reproduce(capacity - current, &mut region_rng)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // 6. Age organisms
        for organism in self.organisms.iter() {
            organism.increment_age();
        }

        // 7. Remove aged-out
        self.organisms.retain(|org| !org.is_dead());

        offspring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;

    #[derive(Debug)]
    struct MockFunction;
    impl WorldFunction for MockFunction {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> { vec![1.0] }
    }

    fn create_test_organism(age: usize) -> Arc<Organism> {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        Arc::new(Organism::new(phenotype, age, (None, None)))
    }

    #[test]
    fn given_region_with_organisms_when_process_lifecycle_then_fitness_evaluated() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(10));
        for i in 0..5 {
            region.add_organism(create_test_organism(i));
        }

        let offspring = region.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);

        for org in region.organisms() {
            assert!(org.score().is_some());
        }
        assert_eq!(offspring.len(), 5);
    }

    #[test]
    fn given_region_over_capacity_when_process_lifecycle_then_truncated() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(3));
        for i in 0..5 {
            region.add_organism(create_test_organism(i));
        }

        region.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(region.organism_count(), 3);
    }

    #[test]
    fn given_same_seed_when_process_lifecycle_then_deterministic_offspring() {
        let mut region1 = Region::new();
        let mut region2 = Region::new();
        region1.set_carrying_capacity(Some(10));
        region2.set_carrying_capacity(Some(10));
        
        for i in 0..5 {
            region1.add_organism(create_test_organism(i));
            region2.add_organism(create_test_organism(i));
        }

        let offspring1 = region1.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
        let offspring2 = region2.process_region_lifecycle(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(offspring1.len(), offspring2.len());
    }
}
```

**Update module**: `hill_descent_lib/src/world/regions/region/mod.rs` add `mod process_region;`


### Task 6: Parallel Region Processing

**New File**: `hill_descent_lib/src/world/regions/parallel_process.rs`

```rust
use super::Regions;
use crate::world::organisms::{organism::Organism, Organisms};
use crate::world::regions::derive_region_seed;
use crate::world::world_function::WorldFunction;
use rayon::prelude::*;
use std::sync::Arc;

impl Regions {
    /// Processes all regions in parallel (core parallelization point).
    /// Each region gets dedicated thread with deterministic RNG.
    pub fn parallel_process_regions(
        &mut self,
        world_function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: Option<&[f64]>,
        world_seed: u64,
    ) -> Organisms {
        let all_offspring: Vec<Vec<Organism>> = self
            .regions
            .par_iter_mut()
            .map(|(region_key, region)| {
                let region_seed = derive_region_seed(world_seed, region_key);
                region.process_region_lifecycle(world_function, inputs, known_outputs, region_seed)
            })
            .collect();

        let all_offspring_flat: Vec<Arc<Organism>> = all_offspring
            .into_iter()
            .flat_map(|v| v.into_iter().map(Arc::new))
            .collect();

        Organisms::new_from_arc_vec(all_offspring_flat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use crate::world::regions::region::Region;

    #[derive(Debug)]
    struct MockFunction;
    impl WorldFunction for MockFunction {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> { vec![1.0] }
    }

    fn create_test_organism() -> Arc<Organism> {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        Arc::new(Organism::new(phenotype, 0, (None, None)))
    }

    #[test]
    fn given_multiple_regions_when_parallel_process_then_all_processed() {
        let mut regions = Regions::new();
        for i in 0..3 {
            let mut region = Region::new();
            region.set_carrying_capacity(Some(10));
            for _ in 0..5 { region.add_organism(create_test_organism()); }
            regions.insert_region(vec![i], region);
        }

        let offspring = regions.parallel_process_regions(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(offspring.len(), 15); // 3 regions * 5 offspring each
    }

    #[test]
    fn given_same_seed_when_parallel_process_then_deterministic_results() {
        let mut regions1 = Regions::new();
        let mut regions2 = Regions::new();
        
        for i in 0..5 {
            let mut r1 = Region::new();
            let mut r2 = Region::new();
            r1.set_carrying_capacity(Some(10));
            r2.set_carrying_capacity(Some(10));
            for _ in 0..5 {
                r1.add_organism(create_test_organism());
                r2.add_organism(create_test_organism());
            }
            regions1.insert_region(vec![i], r1);
            regions2.insert_region(vec![i], r2);
        }

        let offspring1 = regions1.parallel_process_regions(&MockFunction, &[], Some(&[1.0]), 12345);
        let offspring2 = regions2.parallel_process_regions(&MockFunction, &[], Some(&[1.0]), 12345);
        assert_eq!(offspring1.len(), offspring2.len());
    }
}
```

**Update module**: `hill_descent_lib/src/world/regions/mod.rs` add `mod parallel_process;`

### Task 7: Add Helper to Organisms

**File**: `hill_descent_lib/src/world/organisms/mod.rs`

```rust
impl Organisms {
    pub fn new_from_arc_vec(organisms: Vec<Arc<Organism>>) -> Self {
        Self { organisms }
    }
    
    pub fn new_empty() -> Self {
        Self { organisms: Vec::new() }
    }
}
```


### Task 8: Refactor training_run

**File**: `hill_descent_lib/src/world/training_run.rs`

Replace the main training loop with:

```rust
pub fn training_run(&mut self, inputs: &[f64], known_outputs: Option<&[f64]>) -> bool {
    // Validate inputs
    if let Some(expected_outputs) = known_outputs {
        assert!(!expected_outputs.is_empty(), "known_outputs must not be empty");
        assert!(expected_outputs.iter().all(|&x| x.is_finite()), "known_outputs must be finite");
    }

    // PARALLEL PHASE: Process all regions independently
    let world_seed = self.global_constants.world_seed();
    let offspring = self.regions.parallel_process_regions(
        self.world_function.as_ref(),
        inputs,
        known_outputs,
        world_seed,
    );

    // SYNC PHASE: Global coordination
    self.organisms.extend(offspring.into_inner());
    
    let resolution_limit_reached = self
        .regions
        .update(&mut self.organisms, &mut self.dimensions);

    resolution_limit_reached
}
```

### Task 9: Update World for world_seed Access

**File**: `hill_descent_lib/src/world/mod.rs`

If `GlobalConstants::world_seed()` doesn't exist, add to World struct:
```rust
pub struct World {
    // existing fields...
    world_seed: u64,
}
```

And store in `World::new()`:
```rust
let world_seed = global_constants.world_seed();
Self { /* fields */, world_seed }
```

### Task 10: Clean Up Old Code

Remove parallelization from (should be serial now):
- `hill_descent_lib/src/world/organisms/run_all.rs` - remove `par_iter()`
- `hill_descent_lib/src/world/regions/sort_regions.rs` - remove `par_iter_mut()`  
- `hill_descent_lib/src/world/organisms/increment_ages.rs` - remove `par_iter()`

These are now handled inside `Region::process_region_lifecycle`.

---

## Testing

**New Test File**: `hill_descent_lib/tests/parallel_determinism_test.rs`

```rust
use hill_descent_lib::*;
use std::ops::RangeInclusive;

#[derive(Debug)]
struct SimpleTestFunction;
impl world::world_function::WorldFunction for SimpleTestFunction {
    fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> { vec![1.0] }
}

#[test]
fn given_same_seed_when_multiple_runs_then_identical_results() {
    let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, 0.0..=1.0];
    let seed = 42;
    
    let constants1 = parameters::global_constants::GlobalConstants::new_with_seed(100, 10, seed);
    let mut world1 = setup_world(&bounds, constants1, Box::new(SimpleTestFunction));
    
    for _ in 0..10 { world1.training_run(&[0.5], Some(&[1.0])); }
    
    let constants2 = parameters::global_constants::GlobalConstants::new_with_seed(100, 10, seed);
    let mut world2 = setup_world(&bounds, constants2, Box::new(SimpleTestFunction));
    
    for _ in 0..10 { world2.training_run(&[0.5], Some(&[1.0])); }
    
    assert_eq!(world1.organisms().len(), world2.organisms().len());
    assert_eq!(world1.get_best_score(), world2.get_best_score());
}

#[test]
fn given_different_seeds_when_run_then_different_results() {
    let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, 0.0..=1.0];
    
    let constants1 = parameters::global_constants::GlobalConstants::new_with_seed(100, 10, 42);
    let mut world1 = setup_world(&bounds, constants1, Box::new(SimpleTestFunction));
    for _ in 0..10 { world1.training_run(&[0.5], Some(&[1.0])); }
    
    let constants2 = parameters::global_constants::GlobalConstants::new_with_seed(100, 10, 123);
    let mut world2 = setup_world(&bounds, constants2, Box::new(SimpleTestFunction));
    for _ in 0..10 { world2.training_run(&[0.5], Some(&[1.0])); }
    
    assert_ne!(world1.get_best_score(), world2.get_best_score());
}

#[test]
fn given_parallel_execution_when_same_seed_then_deterministic() {
    let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0; 2];
    let seed = 12345;
    let mut results = Vec::new();
    
    for _ in 0..5 {
        let constants = parameters::global_constants::GlobalConstants::new_with_seed(500, 20, seed);
        let mut world = setup_world(&bounds, constants, Box::new(SimpleTestFunction));
        for _ in 0..20 { world.training_run(&[0.0], Some(&[1.0])); }
        results.push((world.organisms().len(), world.get_best_score()));
    }
    
    let first = results[0];
    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(first, *result, "Run {} differs: {:?} vs {:?}", i, first, result);
    }
}
```


---

## Validation Checklist

- [ ] `cargo build` - compiles without errors
- [ ] `cargo fmt` - all files formatted
- [ ] `cargo clippy` - zero warnings
- [ ] `cargo clippy --tests` - zero warnings
- [ ] `cargo test --workspace` - all tests pass
- [ ] New determinism tests pass
- [ ] Benchmarks run successfully
- [ ] Same seed = identical results (validate with 5+ runs)
- [ ] Different seeds = different results
- [ ] Performance improvements for Pop >= 100

---

## Expected Performance

| Config           | Baseline | Expected | Improvement        |
| ---------------- | -------- | -------- | ------------------ |
| Pop 10, R 2      | 10ms     | 15ms     | 0.67x (acceptable) |
| Pop 100, R 10    | 103ms    | 50ms     | 2.1x               |
| Pop 500, R 20    | 419ms    | 150ms    | 2.8x               |
| Pop 10000, R 100 | 10,919ms | 3,000ms  | 3.6x               |

**Why this works**: Coarser granularity (10-100 organisms/thread vs 1), better cache locality, less coordination overhead, scales with region count.

---

## Troubleshooting

**Problem**: `Arc<T>: Send` trait errors  
**Solution**: Add `+ Sync` to WorldFunction and SingleValuedFunction traits

**Problem**: Non-deterministic results  
**Solution**: Verify `derive_region_seed()` called correctly, check `StdRng::seed_from_u64()` not `thread_rng()`

**Problem**: Performance worse than baseline  
**Solution**: Check regions >= 4, verify Arc conversion complete, ensure `parallel_process_regions` actually parallel

**Problem**: `world_seed() method not found`  
**Solution**: Add `world_seed()` to GlobalConstants or store in World struct

---

## Post-Implementation

1. Update PARALLELIZATION_ANALYSIS.md with results
2. Update README with performance characteristics  
3. Consider Phase 2 optimizations
4. Benchmark on different hardware (4/8/16/32-core systems)

---

## References

- **AGENTS.md** - Development guidelines
- **hill_descent_lib/pdd.md** - Algorithm definitions
- **Rayon docs** - https://docs.rs/rayon/

---

**End of Specification**

