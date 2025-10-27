# Crates.io Publication Plan for hill_descent_lib

**Created:** October 27, 2025  
**Status:** Stage 1 Complete, Ready for Stage 2  
**Goal:** Prepare `hill_descent_lib` for public consumption on crates.io

---

## Overview

This document outlines the complete 7-stage plan to prepare `hill_descent_lib` for publication to crates.io. The library is an n-dimensional genetic algorithm optimization system designed for general-purpose optimization problems.

---

## Stage 1: API Audit & Minimal Public Surface ✅ COMPLETE

**Goal:** Identify what MUST be public vs implementation details

### Completed Changes:

1. **Made Internal (Hidden from Public API):**
   - `Gamete`, `Locus`, `Phenotype` structs → implementation details
   - `NUM_SYSTEM_PARAMETERS`, `E0` constants → `pub(crate)`
   - `World::run_epoch()` method → `pub(crate)` (only used internally)
   - `gamete`, `locus`, `phenotype` modules → private modules

2. **Flat Re-exports Added:**
   ```rust
   // Users can now import from crate root:
   use hill_descent_lib::{
       World, 
       GlobalConstants, 
       SingleValuedFunction, 
       WorldFunction, 
       setup_world
   };
   ```

3. **Final Public API:**
   - `setup_world()` - Primary initialization function
   - `GlobalConstants` - Configuration struct
   - `World` - Main optimization container
   - `SingleValuedFunction` - Trait for implementing custom fitness functions
   - `WorldFunction` - Auto-implemented wrapper trait
   - `World` methods: `training_run()`, `get_best_score()`, `get_best_organism()`, `get_state()`, `get_state_for_web()`

### Verification:
- ✅ All workspace crates compile
- ✅ All 432 tests pass
- ✅ Zero clippy warnings
- ✅ No breaking changes for server or benchmarks

### Documentation:
- `STAGE_1_API_AUDIT.md` - Detailed analysis
- `STAGE_1_COMPLETE.md` - Summary of changes

---

## Stage 2: Cargo.toml Metadata (NEXT)

**Goal:** Add all required and recommended metadata for crates.io publication

### Required Fields:

1. **`license`** - Choose license type
   - Option 1: `"MIT"` (permissive, most common)
   - Option 2: `"Apache-2.0"` (permissive, patent protection)
   - Option 3: `"MIT OR Apache-2.0"` (dual license, common in Rust ecosystem)
   - **Decision needed:** Which license?

2. **`description`** - One-line summary (max ~60 chars optimal)
   - Proposed: `"N-dimensional genetic algorithm optimization library"`
   - Alternative: `"Genetic algorithm library for n-dimensional optimization problems"`

3. **`repository`** - GitHub repository URL
   - Value: `"https://github.com/cainem/hill_descent"`

4. **`readme`** - Path to README file
   - Value: `"README.md"` (to be created in Stage 4)

### Recommended Fields:

5. **`keywords`** - Max 5 keywords for searchability
   - Proposed: `["optimization", "genetic-algorithm", "hill-descent", "evolution", "fitness"]`
   - Alternative: `["optimization", "genetic-algorithm", "neural-network", "evolution", "machine-learning"]`

6. **`categories`** - From crates.io category list
   - Proposed: `["algorithms", "science"]`
   - Available categories: https://crates.io/category_slugs

7. **`homepage`** - Project homepage (optional)
   - Could be same as repository or a dedicated docs site

8. **`documentation`** - Custom docs URL (optional)
   - Defaults to docs.rs, usually not needed

9. **`authors`** - List of authors (optional but recommended)
   - Format: `["Name <email@example.com>"]`

10. **`edition`** - Rust edition
    - Current: `"2024"` ✅ Already set

11. **`version`** - Version number
    - Current: `"0.1.0"` 
    - Consider: `"0.0.1"` for very first release (signals early/experimental)
    - Or keep `"0.1.0"` (signals somewhat stable API)

### Example Result:
```toml
[package]
name = "hill_descent_lib"
version = "0.1.0"  # or "0.0.1"
edition = "2024"
license = "MIT OR Apache-2.0"
description = "N-dimensional genetic algorithm optimization library"
repository = "https://github.com/cainem/hill_descent"
readme = "README.md"
keywords = ["optimization", "genetic-algorithm", "hill-descent", "evolution", "fitness"]
categories = ["algorithms", "science"]
authors = ["Your Name <your.email@example.com>"]

# ... rest of Cargo.toml
```

### Verification:
- Run `cargo package --list` to see what files will be included
- Check package size (must be < 10MB)
- Verify all metadata is correct

---

## Stage 3: README.md Creation

**Goal:** Create a compelling README that explains the library in 30 seconds

### Required Sections:

1. **Title & One-Line Description**
   ```markdown
   # hill_descent_lib
   
   A Rust genetic algorithm library for n-dimensional optimization problems.
   ```

2. **Quick Example** (2-10 lines of code)
   ```rust
   use hill_descent_lib::{GlobalConstants, SingleValuedFunction, setup_world};
   use std::ops::RangeInclusive;
   
   // Define your fitness function
   #[derive(Debug)]
   struct MyFunction;
   impl SingleValuedFunction for MyFunction {
       fn single_run(&self, params: &[f64]) -> f64 {
           // Return fitness score (lower is better)
           params[0].powi(2) + params[1].powi(2)
       }
   }
   
   // Set up optimization
   let bounds = vec![-10.0..=10.0, -10.0..=10.0];
   let constants = GlobalConstants::new(100, 10); // pop_size, regions
   let mut world = setup_world(&bounds, constants, Box::new(MyFunction));
   
   // Run optimization
   for _ in 0..100 {
       world.training_run(&[], &[0.0]); // Empty inputs, target floor of 0
   }
   println!("Best score: {}", world.get_best_score());
   ```

3. **Features List**
   - N-dimensional optimization
   - Genetic algorithm with spatial regions
   - Adaptive mutation and reproduction
   - Deterministic (seeded RNG)
   - Optional tracing/logging
   - Zero-copy parallelism via rayon

4. **Installation**
   ```toml
   [dependencies]
   hill_descent_lib = "0.1.0"
   ```

5. **Documentation Link**
   - Link to docs.rs (generated automatically)

6. **License**
   - State the license clearly

7. **Optional Sections:**
   - Use cases (optimization, neural networks, parameter tuning)
   - Performance characteristics
   - Comparison to other libraries
   - Contributing guidelines
   - Badges (build status, docs, crates.io version)

### Best Practices:
- Keep it under 500 lines (preferably under 200)
- Use clear, simple language
- Show working code, not pseudocode
- Explain *what* it does before *how* it works
- Link to comprehensive docs for details

---

## Stage 4: Module & Item Documentation

**Goal:** Add comprehensive doc comments to all public items

### Documentation Requirements:

1. **Module-Level Docs** (`//!` at top of files)
   - `lib.rs` - Crate overview with examples
   - `world/mod.rs` - Explain the World concept
   - `parameters/mod.rs` - Explain configuration
   - `world/single_valued_function.rs` - Trait documentation
   - `world/world_function.rs` - Trait documentation

2. **Struct Documentation** (`///` before structs)
   - `World` - Main optimization container
   - `GlobalConstants` - Configuration parameters
   
3. **Method Documentation** (`///` before methods)
   - All public methods on `World`
   - All public methods on `GlobalConstants`
   - Include:
     - Brief description
     - Parameters with types and meaning
     - Return value description
     - Examples (runnable with `cargo test`)
     - Panics section if applicable
     - Safety section if unsafe

4. **Trait Documentation**
   - `SingleValuedFunction` - Full explanation with examples
   - `WorldFunction` - Explain auto-implementation

5. **Function Documentation**
   - `setup_world()` - Comprehensive guide

### Documentation Standards:

```rust
/// Brief one-line summary.
///
/// More detailed explanation if needed. Can span
/// multiple paragraphs.
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Panics
///
/// Conditions under which this panics
///
/// # Examples
///
/// ```
/// use hill_descent_lib::{GlobalConstants, setup_world};
/// let constants = GlobalConstants::new(100, 10);
/// // ... example code
/// ```
pub fn my_function(param1: Type1, param2: Type2) -> ReturnType {
    // implementation
}
```

### Testing:
- Run `cargo doc --open` to view generated docs
- Verify all doc tests compile: `cargo test --doc`
- Check for broken links
- Ensure examples actually work

---

## Stage 5: Examples Directory (Optional but Recommended)

**Goal:** Provide working examples users can run and learn from

### Proposed Examples:

1. **`examples/simple_optimization.rs`**
   - Basic 2D optimization
   - Quadratic function
   - Shows complete workflow
   - ~50 lines

2. **`examples/custom_function.rs`**
   - Implementing `SingleValuedFunction`
   - Multiple test functions (Himmelblau, Rastrigin)
   - Demonstrates trait usage
   - ~100 lines

3. **`examples/multi_dimensional.rs`**
   - 10+ dimension optimization
   - Shows scalability
   - ~75 lines

4. **`examples/with_logging.rs`**
   - Using the `enable-tracing` feature
   - Shows debugging workflow
   - ~60 lines

### Running Examples:
```bash
cargo run --example simple_optimization
cargo run --example custom_function
cargo run --example multi_dimensional
cargo run --example with_logging --features enable-tracing
```

### Benefits:
- Lowers barrier to entry
- Provides copy-paste starting points
- Demonstrates best practices
- Increases adoption

---

## Stage 6: Pre-Publish Verification

**Goal:** Catch all issues before publishing

### Checklist:

1. **Package Contents**
   ```bash
   cargo package --list
   ```
   - Verify only necessary files included
   - Check for accidentally included large files
   - Ensure no secrets in code/comments

2. **Package Size**
   - Check `.crate` file size in `target/package/`
   - Must be < 10MB
   - Typical library should be < 1MB

3. **Dry Run**
   ```bash
   cargo publish --dry-run
   ```
   - Performs all checks without publishing
   - Builds documentation
   - Runs tests
   - Validates metadata

4. **Documentation Build**
   ```bash
   cargo doc --no-deps --open
   ```
   - Verify docs render correctly
   - Check all links work
   - Ensure examples display properly
   - Simulate docs.rs experience

5. **Test All Features**
   ```bash
   cargo test
   cargo test --features enable-tracing
   cargo test --all-features
   cargo test --no-default-features
   ```

6. **Clippy (Strict)**
   ```bash
   cargo clippy --all-features -- -D warnings
   cargo clippy --tests -- -D warnings
   ```

7. **Format Check**
   ```bash
   cargo fmt -- --check
   ```

8. **Dependency Audit**
   ```bash
   cargo tree
   ```
   - Verify no unexpected dependencies
   - Check for duplicate versions

9. **Cross-Platform Check** (if possible)
   - Test on Windows ✅ (primary platform)
   - Test on Linux (via CI or VM)
   - Test on macOS (via CI or VM)

### Exclusions (Optional):

Add to `Cargo.toml` if needed:
```toml
[package]
exclude = [
    "target/",
    ".git/",
    ".github/",
    "run_stats/",
    "*.md",  # Exclude non-essential docs
]
```

Or use `include` for explicit control:
```toml
[package]
include = [
    "src/**/*.rs",
    "Cargo.toml",
    "README.md",
    "LICENSE*",
]
```

---

## Stage 7: Publication

**Goal:** Publish to crates.io successfully

### Prerequisites:

1. **Crates.io Account**
   - Sign up at https://crates.io/
   - Log in via GitHub
   - Verify email address at https://crates.io/settings/profile

2. **API Token**
   - Create at https://crates.io/settings/tokens
   - Copy token (will only be shown once!)
   - Login via cargo:
     ```bash
     cargo login
     # Paste token when prompted
     ```
   - Token stored in `~/.cargo/credentials.toml`

### Publishing Steps:

1. **Final Version Check**
   - Ensure `Cargo.toml` version is correct
   - Version must follow SemVer
   - Cannot republish same version (must increment)

2. **Git Tag** (Recommended)
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

3. **Publish**
   ```bash
   cargo publish
   ```
   - Uploads `.crate` file to crates.io
   - Triggers docs.rs build
   - Makes crate publicly available

4. **Verification**
   - Visit https://crates.io/crates/hill_descent_lib
   - Check crate page looks correct
   - Wait ~5 minutes for docs.rs build
   - Visit https://docs.rs/hill_descent_lib
   - Verify docs built successfully

5. **Test Installation**
   ```bash
   # In a temporary directory
   cargo new test_install
   cd test_install
   cargo add hill_descent_lib
   # Add example code to src/main.rs
   cargo run
   ```

### Post-Publication:

1. **Announce**
   - Add badge to README:
     ```markdown
     [![Crates.io](https://img.shields.io/crates/v/hill_descent_lib.svg)](https://crates.io/crates/hill_descent_lib)
     [![Documentation](https://docs.rs/hill_descent_lib/badge.svg)](https://docs.rs/hill_descent_lib)
     ```
   - Update repository README
   - Share on Reddit r/rust (optional)
   - Share on social media (optional)

2. **Monitor**
   - Watch for bug reports
   - Check docs.rs build status
   - Monitor download statistics

3. **Future Updates**
   - Increment version number for changes
   - Follow SemVer strictly:
     - Patch (0.1.X): Bug fixes, no API changes
     - Minor (0.X.0): New features, backward compatible
     - Major (X.0.0): Breaking changes
   - Tag each release in git

### Emergency: Yanking a Version

If you publish a broken version:
```bash
cargo yank --version 0.1.0
# Or undo yank:
cargo yank --version 0.1.0 --undo
```

**Note:** Yanking doesn't delete code, just prevents new projects from using it. Existing `Cargo.lock` files still work.

---

## References

- **Cargo Publishing Guide:** https://doc.rust-lang.org/cargo/reference/publishing.html
- **Rust API Guidelines:** https://rust-lang.github.io/api-guidelines/
- **SemVer Specification:** https://semver.org/
- **Crates.io Category List:** https://crates.io/category_slugs
- **Docs.rs:** https://docs.rs/

---

## Decision Points Summary

### Decisions Needed Before Proceeding:

1. **License Choice** (Stage 2)
   - [ ] MIT
   - [ ] Apache-2.0
   - [ ] MIT OR Apache-2.0 (recommended)
   - [ ] Other: __________

2. **Version Number** (Stage 2)
   - [ ] 0.0.1 (very experimental)
   - [ ] 0.1.0 (somewhat stable)

3. **Keywords** (Stage 2)
   - Proposed: `["optimization", "genetic-algorithm", "hill-descent", "evolution", "fitness"]`
   - Alternative suggestions: __________

4. **Categories** (Stage 2)
   - Proposed: `["algorithms", "science"]`
   - Alternative suggestions: __________

5. **Author Information** (Stage 2)
   - Name and email for author field

6. **Documentation Focus** (Stage 4)
   - [ ] General n-dimensional optimization
   - [ ] Neural network emphasis
   - [ ] Both equally

---

## Progress Tracking

- [x] Stage 1: API Audit & Minimal Public Surface
- [ ] Stage 2: Cargo.toml Metadata
- [ ] Stage 3: README.md Creation
- [ ] Stage 4: Module & Item Documentation
- [ ] Stage 5: Examples Directory
- [ ] Stage 6: Pre-Publish Verification
- [ ] Stage 7: Publication

---

## Notes

- Keep this document updated as stages are completed
- Document any deviations from the plan
- Record lessons learned for future crate publications
- This plan is version controlled and part of the repository history
