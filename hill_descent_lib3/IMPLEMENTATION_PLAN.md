# Hill Descent Lib 3 Implementation Plan

## Goal
Create a performance baseline (named `hill_descent_lib3`) that isolates the impact of the message-passing architecture in `hill_descent_lib2`. This implementation will replicate the **exact logic flow** and **phase structure** of `lib2` but replace the `messaging_thread_pool` actor model with a shared-memory concurrency model using `Rayon` and `RwLock`.

## Core Philosophy
- **Logic**: Strict adherence to `lib2`'s algorithmic steps (combined `process_epoch`, specific reproduction logic).
- **Concurrency**: "Coarse-grained locking" on a single collection of organisms.
- **Parallelism**: `Rayon` parallel iterators instead of message passing.

## Architecture

### 1. Data Structures

#### `Organism`
Structure will mirror `lib2`'s `Organism` but explicitly hold its state (mutable) instead of being an actor.
- Will contain: `Phenotype`, `RegionKey`, `scores`, `age`, `dimensions_version`.
- Methods will correspond to `lib2`'s `_impl` functions:
    - `process_epoch(&mut self, world_ctx: &WorldContext) -> ProcessResult`
    - `reproduce(&self, partner: &Organism) -> (Phenotype, Phenotype)`

#### `World`
Manages the population and orchestration.
- **Storage**: `organisms: Vec<Arc<RwLock<Organism>>>`
    - `Arc`: Allows cheap cloning for "pair" vectors during reproduction (avoiding lookups).
    - `RwLock`: strict "lock as necessary" semantic.
        - `write` lock for internal updates (Phase 1).
        - `read` lock for parenting (Phase 2).
- **Global State**: `Dimensions`, `GlobalConstants`, `WorldFunction` (as standard, shared via `Arc` or reference).

### 2. Execution Flow (per Training Step)

The `training_run` loop will execute phases sequentially, parallelizing *within* each phase:

#### Phase 1: Process Epoch (Update)
- **Action**: Update region keys, evaluate fitness, increment age.
- **Concurrency**:
  ```rust
  self.organisms.par_iter().for_each(|org_lock| {
      let mut org = org_lock.write().unwrap();
      org.process_epoch(&ctx)
  });
  ```
- **Constraint Handling**: Collect "Out of Bounds" results. If any occur, expand dimensions and re-run (exactly as `lib2`).

#### Phase 2: Culling & Region Analysis
- **Action**: Remove dead organisms, group into regions to determine spatial density.
- **Implementation**:
    - Filter `self.organisms` (remove dead).
    - Re-build `Regions` map (needed for pairing strategy).

#### Phase 3: Reproduction
- **Selection**: Logic determines pairs of parents (based on regions/density).
- **Preparation**: Create a vector of `(Arc<RwLock<Organism>>, Arc<RwLock<Organism>>)` pairs.
- **Execution**:
  ```rust
  let offspring_phenotypes: Vec<(Phenotype, Phenotype)> = pairs.par_iter().map(|(p1, p2)| {
      let parent1 = p1.read().unwrap();
      let parent2 = p2.read().unwrap();
      Organism::reproduce(&*parent1, &*parent2)
  }).collect();
  ```
- **Birth**: Create new `Organism` instances from phenotypes and append to `self.organisms`.

## Module Structure
The directory structure will purposefully resemble `hill_descent_lib2` to make side-by-side comparison easy.

```
hill_descent_lib3/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── training_data.rs (shared/copied)
│   ├── tracing_init.rs (shared/copied)
│   ├── world/
│   │   ├── mod.rs (World struct & specialized impl)
│   │   ├── dimensions.rs
│   │   ├── regions.rs 
│   │   └── ...
│   ├── organism/
│   │   ├── mod.rs (Organism struct)
│   │   ├── process_epoch.rs (Logic ported from lib2)
│   │   ├── reproduce.rs (Logic ported from lib2)
│   │   └── ...
│   ├── phenotype/ (Likely shared or copied)
│   ├── locus/ (Likely shared or copied)
│   └── parameters/ (Shared)
```

## Implementation Steps

1.  **Scaffold**: Initialize `hill_descent_lib3` crate.
2.  **Dependencies**: Copy `lib2` dependencies (minus `messaging_thread_pool`), ensure `rayon` is present.
3.  **Core Domain**: Copy/Port pure domain logic (`locus`, `phenotype`, `parameters`, `gen_hybrid_range`) from `lib2`.
4.  **Organism**: Implement `Organism` struct wrapped around `lib2`'s `_impl` logic.
5.  **World**: Implement `World` struct with `par_iter` logic.
6.  **Integration**: Implement `training_run` loop.
7.  **Verification**: Add tests ensuring deterministic behavior matches `lib2` expectations.

## Key Considerations
- **Determinism**: Must ensure `StdRng` is seeded correctly in the parallel blocks (e.g. deriving seed from organism ID or index) to maintain reproducibility.
- **Memory**: `Arc<RwLock<>>` adds slight overhead; strict monitoring of contention (though `par_iter` partitions work well).
