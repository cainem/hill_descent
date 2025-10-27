# Stage 1: API Audit - Public vs Internal Items

**Date:** October 27, 2025  
**Goal:** Identify what must be public vs implementation details for publishing to crates.io

## Summary of Findings

Based on analysis of:
- `hill_descent_server` (web visualization server)
- `hill_descent_benchmarks` (performance testing tool)
- Integration tests in `hill_descent_lib/tests/`

## Current Public API Analysis

### ✅ DEFINITELY PUBLIC - Core API (Currently Used)

These items are actively used by consumers and MUST remain public:

#### Top-Level Function
- **`setup_world()`** - Primary initialization function
  - Used by: server, benchmarks, all integration tests
  - Signature: `fn setup_world(params: &[RangeInclusive<f64>], GlobalConstants, Box<dyn WorldFunction>) -> World`

#### Types
- **`GlobalConstants`** (struct)
  - Used by: server, benchmarks, all integration tests
  - Constructor: `GlobalConstants::new(population_size, regions)`
  - Methods used: `world_seed()`, `population_size()`, `target_regions()`

- **`World`** (struct)
  - Primary container for optimization state
  - Methods actively used:
    - `training_run(&[], &[floor_value]) -> bool` - Run one optimization step
    - `get_best_score() -> f64` - Get current best fitness
    - `get_state_for_web() -> String` - Get JSON state (server only)
    - `get_state() -> String` - Get JSON state (tests only)
    - `run_epoch(&[&[f64]], &[&[f64]])` - Batch training (not currently used but documented)
    - `get_best_organism()` - Access best solution (not seen in usage but likely needed)

#### Traits
- **`SingleValuedFunction`** (trait)
  - Used by: server, benchmarks, all integration tests
  - Required method: `single_run(&self, phenotype_expressed_values: &[f64]) -> f64`
  - Optional method: `function_floor(&self) -> f64` (defaults to 0.0)

- **`WorldFunction`** (trait)
  - Used by: server, benchmarks
  - Automatically implemented for all `SingleValuedFunction` types
  - Required for `setup_world()` parameter
  - Methods: `run()`, `function_floor()`

### ❓ POSSIBLY INTERNAL - Not Used Externally

These are currently public but NOT used by any external consumers:

- **`Phenotype`** - Only used internally by World
- **`Gamete`** - Only used internally by World/Organisms
- **`Locus`** - Only used internally by Gamete

**Recommendation:** Make these `pub(crate)` unless there's a future use case for advanced users to access organism internals.

### 🔒 CLEARLY INTERNAL - Already Restricted or Should Be

- **`organisms` module** - Currently `pub mod` but only used internally
- **`regions` module** - Implementation detail of World
- **`dimensions` module** - Implementation detail of World
- **`test_utils` module** - Already marked `#[cfg(test)]` ✅
- **`tracing_init` module** - Already feature-gated ✅

### 📦 RE-EXPORTS NEEDED (Currently Deep Paths)

Currently users must write:
```rust
use hill_descent_lib::world::single_valued_function::SingleValuedFunction;
use hill_descent_lib::world::world_function::WorldFunction;
use hill_descent_lib::parameters::GlobalConstants;
```

Should be flattened to:
```rust
use hill_descent_lib::{SingleValuedFunction, WorldFunction, GlobalConstants, World, setup_world};
```

## Usage Pattern Examples

### Server Usage (hill_descent_server/src/main.rs)
```rust
use hill_descent_lib::{
    GlobalConstants, setup_world, 
    world::single_valued_function::SingleValuedFunction,
    world::world_function::WorldFunction,
};

// Implement a custom function
struct BukinN6;
impl SingleValuedFunction for BukinN6 {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // ... implementation
    }
}

// Create world
let param_range: Vec<RangeInclusive<f64>> = vec![...];
let global_constants = GlobalConstants::new(100, 10);
let world = setup_world(&param_range, global_constants, Box::new(BukinN6));

// Use world
world.get_best_score();
world.get_state_for_web();
world.training_run(&[], &[floor]);
```

### Benchmarks Usage (hill_descent_benchmarks/src/)
```rust
use hill_descent_lib::{setup_world, GlobalConstants};
use hill_descent_lib::world::single_valued_function::SingleValuedFunction;
use hill_descent_lib::WorldFunction;

let mut world = setup_world(&param_ranges, global_constants, function);
world.training_run(&[], &[floor]);
world.get_best_score();
```

### Integration Tests Usage (hill_descent_lib/tests/)
```rust
use hill_descent_lib::{
    GlobalConstants, setup_world, 
    world::single_valued_function::SingleValuedFunction,
};

// Same pattern as server
```

## Proposed Changes for Stage 1

### 1. Keep Public (Already Good)
- ✅ `setup_world()` function
- ✅ `GlobalConstants` struct and its public methods
- ✅ `World` struct and its public methods
- ✅ `SingleValuedFunction` trait
- ✅ `WorldFunction` trait
- ✅ Constants: `NUM_SYSTEM_PARAMETERS`, `E0`

### 2. Make Internal (pub(crate) or private)
- ❌ `Phenotype` → `pub(crate)`
- ❌ `Gamete` → `pub(crate)`
- ❌ `Locus` → `pub(crate)`
- ❌ `world::organisms` module → `pub(crate)`
- ❌ `world::regions` module → already internal ✅
- ❌ `world::dimensions` module → already internal ✅

### 3. Remove from lib.rs Re-exports
Current lib.rs re-exports these (should be removed):
```rust
pub use gamete::Gamete;        // ❌ Not used externally
pub use locus::Locus;          // ❌ Not used externally
pub use phenotype::Phenotype;  // ❌ Not used externally
```

Keep these re-exports:
```rust
pub use parameters::GlobalConstants;                    // ✅ Keep
pub use world::World;                                   // ✅ Keep
pub use world::world_function::WorldFunction;           // ✅ Keep
```

### 4. Add Missing Re-exports to lib.rs
```rust
pub use world::single_valued_function::SingleValuedFunction;  // 🆕 Add
```

### 5. Module Visibility Changes

**File: `src/lib.rs`**
```rust
// Internal modules (not pub)
mod gamete;           // Change from: pub mod gamete;
mod locus;            // Change from: pub mod locus;
mod phenotype;        // Change from: pub mod phenotype;

// Public module (for trait access)
pub mod world;        // Keep as pub (contains public traits)
pub mod parameters;   // Keep as pub (contains GlobalConstants)

// Other internal modules
mod gen_hybrid_range;  // Already not pub ✅
```

## Minimal Public API Summary

After Stage 1 changes, the ONLY public API will be:

```rust
// Top-level initialization
pub fn setup_world(
    params: &[RangeInclusive<f64>],
    global_constants: GlobalConstants,
    function: Box<dyn WorldFunction>
) -> World

// Core types
pub struct GlobalConstants { ... }
pub struct World { ... }

// Traits users must implement
pub trait SingleValuedFunction { ... }
pub trait WorldFunction { ... }  // Auto-implemented for SingleValuedFunction

// Constants
pub const NUM_SYSTEM_PARAMETERS: usize;
pub const E0: f64;

// Logging macros (when enable-tracing feature enabled)
#[cfg(feature = "enable-tracing")]
pub fn init_tracing() { ... }
```

## Testing After Changes

Must verify:
1. ✅ `cargo build --package hill_descent_lib`
2. ✅ `cargo build --package hill_descent_server`
3. ✅ `cargo build --package hill_descent_benchmarks`
4. ✅ `cargo test --workspace`
5. ✅ `cargo clippy --workspace`

## Questions for User

1. **`get_state()` vs `get_state_for_web()`**: 
   - Server uses `get_state_for_web()`
   - Tests use `get_state()`
   - Should both remain public or just one?

2. **`run_epoch()` method**:
   - Not currently used but documented
   - Keep public for batch training use case?

3. **`get_best_organism()` method**:
   - Not seen in current usage but likely needed to extract final solution
   - Keep public?

4. **Constants `NUM_SYSTEM_PARAMETERS` and `E0`**:
   - Currently public but not used externally
   - Make internal or keep for transparency?

## Next Steps

Once approved:
1. Implement visibility changes
2. Update re-exports in lib.rs
3. Test all dependent crates compile
4. Run full test suite
5. Verify no public API breakage
6. Proceed to Stage 2 (Cargo.toml metadata)
