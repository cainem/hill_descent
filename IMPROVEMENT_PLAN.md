# Hill Descent Library Improvements - Implementation Plan

**Created:** November 3, 2025  
**Branch:** improvements  
**Target Version:** 0.2.0 (breaking changes)

## Status Overview

- [ ] Phase 1: Core API Redesign (HIGH Priority)
- [ ] Phase 2: Update Tests & Examples  
- [ ] Phase 3: Documentation Improvements (MEDIUM Priority)
- [ ] Phase 4: Final Cleanup & Release

---

## Phase 1: Core API Redesign (HIGH Priority)

### Task 1.1: Create TrainingData Enum ✅ = DONE, 🚧 = IN PROGRESS, ⏸️ = BLOCKED, ❌ = FAILED

**Status:** ⬜ Not Started  
**Files to Create:**
- `hill_descent_lib/src/training_data.rs`

**Acceptance Criteria:**
- [ ] Enum with two variants: `None { floor_value: f64 }` and `Supervised { inputs: &'a [[f64]], outputs: &'a [[f64]] }`
- [ ] Comprehensive doc comments with examples for both variants
- [ ] Export from `lib.rs`
- [ ] Unit tests for enum creation and basic usage

**Commit Message:** `feat: add TrainingData enum for clearer API`

---

### Task 1.2: Refactor World::training_run()

**Status:** ⬜ Not Started  
**Depends On:** Task 1.1  
**Files to Modify:**
- `hill_descent_lib/src/world.rs` (or relevant world module file)

**Acceptance Criteria:**
- [ ] Change signature from `training_run(&mut self, inputs: &[[f64]], known_outputs: &[[f64]])` to `training_run(&mut self, data: TrainingData)`
- [ ] Update internal logic to handle both enum variants
- [ ] Remove old validation that caused confusion
- [ ] Ensure consistent behavior for both variants

**Commit Message:** `refactor: update World::training_run to use TrainingData enum`

---

### Task 1.3: Refactor World::get_best_organism()

**Status:** ⬜ Not Started  
**Depends On:** Task 1.1  
**Files to Modify:**
- `hill_descent_lib/src/world.rs` (or relevant world module file)

**Acceptance Criteria:**
- [ ] Change signature to accept `TrainingData` instead of separate parameters
- [ ] Make behavior consistent with `training_run` for `TrainingData::None`
- [ ] Remove panic on empty training data when using `None` variant
- [ ] Update internal implementation

**Commit Message:** `refactor: update World::get_best_organism to use TrainingData enum`

---

### Task 1.4: Add Convenience Methods to World

**Status:** ⬜ Not Started  
**Depends On:** Task 1.3  
**Files to Modify:**
- `hill_descent_lib/src/world.rs`

**Acceptance Criteria:**
- [ ] Add `get_best_params(&self) -> &[f64]` - non-mutating accessor
- [ ] Add `get_best_score(&self) -> f64` - if not already present as public method
- [ ] Add doc comments with usage examples
- [ ] Add unit tests for new methods

**Commit Message:** `feat: add convenience methods get_best_params and get_best_score`

---

## Phase 2: Update Tests & Examples

### Task 2.1: Update Integration Tests

**Status:** ⬜ Not Started  
**Depends On:** Tasks 1.2, 1.3, 1.4  
**Files to Modify:**
- `hill_descent_lib/tests/organism_persistence_test.rs`
- `hill_descent_lib/tests/parallel_determinism_test.rs`
- `hill_descent_lib/tests/simple_test.rs`
- `hill_descent_lib/tests/two_d_ackley_test.rs`
- `hill_descent_lib/tests/two_d_bukin_n6_test.rs`
- `hill_descent_lib/tests/two_d_himmelblau_test.rs`
- `hill_descent_lib/tests/two_d_levi_n13_test.rs`
- `hill_descent_lib/tests/two_d_rastrgin_test.rs`
- `hill_descent_lib/tests/two_d_schaffer_n2_test.rs`
- `hill_descent_lib/tests/two_d_styblinski_tang_test.rs`

**Acceptance Criteria:**
- [ ] All tests updated to use `TrainingData::None { floor_value: 0.0 }`
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --tests`

**Commit Message:** `refactor: update all integration tests to use TrainingData API`

---

### Task 2.2: Update Example Files

**Status:** ⬜ Not Started  
**Depends On:** Tasks 1.2, 1.3, 1.4  
**Files to Modify:**
- `hill_descent_lib/examples/custom_function.rs`
- `hill_descent_lib/examples/multi_dimensional.rs`
- `hill_descent_lib/examples/simple_optimization.rs`

**Acceptance Criteria:**
- [ ] All examples updated to use new TrainingData API
- [ ] Examples demonstrate best practices
- [ ] All examples compile and run successfully
- [ ] Doc comments in examples explain the API usage

**Commit Message:** `docs: update examples to use TrainingData API`

---

### Task 2.3: Update Server Code

**Status:** ⬜ Not Started  
**Depends On:** Tasks 1.2, 1.3, 1.4  
**Files to Modify:**
- `hill_descent_server/src/main.rs`

**Acceptance Criteria:**
- [ ] Server code updated to use TrainingData API
- [ ] Server compiles and runs
- [ ] Web visualization still works correctly

**Commit Message:** `refactor: update server to use TrainingData API`

---

### Task 2.4: Update Benchmark Code

**Status:** ⬜ Not Started  
**Depends On:** Tasks 1.2, 1.3, 1.4  
**Files to Check/Modify:**
- `hill_descent_benchmarks/src/runner.rs`
- `hill_descent_benchmarks/src/algorithms.rs`
- Any other files using the affected APIs

**Acceptance Criteria:**
- [ ] Benchmark code updated to use TrainingData API
- [ ] Benchmarks compile and run successfully
- [ ] Results are comparable to previous runs (algorithm unchanged)

**Commit Message:** `refactor: update benchmarks to use TrainingData API`

---

## Phase 3: Documentation Improvements (MEDIUM Priority)

### Task 3.1: Fix Initial Score Display

**Status:** ⬜ Not Started  
**Files to Modify:**
- World display/formatting logic

**Acceptance Criteria:**
- [ ] f64::MAX scores display as `<not yet evaluated>` or similar
- [ ] Add helper function for score formatting
- [ ] Update any logging or display code that shows scores
- [ ] Test output looks clean

**Commit Message:** `feat: improve display of initial/unset scores`

---

### Task 3.2: Add ML Use Case Documentation

**Status:** ⬜ Not Started  
**Files to Modify:**
- `hill_descent_lib/README.md`

**Acceptance Criteria:**
- [ ] Add comprehensive ML/neural network optimization example
- [ ] Show 10,000+ parameter use case
- [ ] Include complete working example with internal data management
- [ ] Demonstrate best practices for large-scale optimization

**Commit Message:** `docs: add machine learning use case example to README`

---

### Task 3.3: Add Scaling Guidelines

**Status:** ⬜ Not Started  
**Files to Modify:**
- `hill_descent_lib/README.md`

**Acceptance Criteria:**
- [ ] Add "Scaling Guidelines" section
- [ ] Include parameter count vs population size recommendations
- [ ] Document performance expectations
- [ ] Add "when to use" and "when not to use" guidance
- [ ] Include memory usage information

**Commit Message:** `docs: add scaling guidelines to README`

---

### Task 3.4: Update pdd.md

**Status:** ⬜ Not Started  
**Depends On:** Task 1.1  
**Files to Modify:**
- `hill_descent_lib/pdd.md`

**Acceptance Criteria:**
- [ ] Add TrainingData enum to domain definitions
- [ ] Update any API patterns documentation
- [ ] Ensure consistency with new API design
- [ ] Add notes about use case patterns

**Commit Message:** `docs: update pdd.md with TrainingData enum and new patterns`

---

## Phase 4: Final Cleanup & Release

### Task 4.1: Documentation Review

**Status:** ⬜ Not Started  
**Files to Review:**
- All modified files
- `hill_descent_lib/src/lib.rs` (module docs)

**Acceptance Criteria:**
- [ ] All doc comments updated for API changes
- [ ] Code examples in doc comments use new API
- [ ] Add migration notes for v0.1.0 users (if any)
- [ ] Run `cargo doc --open` and review generated docs

**Commit Message:** `docs: final documentation review and polish`

---

### Task 4.2: Full Test Suite Verification

**Status:** ⬜ Not Started  
**Depends On:** All previous tasks

**Acceptance Criteria:**
- [ ] `cargo fmt` - all code formatted
- [ ] `cargo test --workspace` - all tests pass
- [ ] `cargo clippy --workspace` - zero warnings
- [ ] `cargo clippy --tests` - zero warnings
- [ ] `cargo bench` - benchmarks run successfully
- [ ] Manual testing of examples
- [ ] Manual testing of server

**Commit Message:** `chore: verify all tests and quality checks pass`

---

### Task 4.3: Version Bump and Changelog

**Status:** ⬜ Not Started  
**Files to Modify:**
- `hill_descent_lib/Cargo.toml`
- `hill_descent_server/Cargo.toml`
- `hill_descent_benchmarks/Cargo.toml`
- `Cargo.toml` (workspace)
- Create or update `CHANGELOG.md`

**Acceptance Criteria:**
- [ ] Bump version to 0.2.0 in all Cargo.toml files
- [ ] Document all breaking changes in CHANGELOG
- [ ] Document all new features in CHANGELOG
- [ ] Update dependency versions if needed

**Commit Message:** `chore: bump version to 0.2.0 and update changelog`

---

## Progress Tracking

### Commits Made
1. (none yet)

### Next Action
**Start with Task 1.1:** Create TrainingData enum

### Notes & Decisions
- Using enum approach (Option A) for clearest API
- All tests and examples will be updated to new patterns
- Documentation will be added to README.md and pdd.md
- Breaking changes are acceptable (no known external users)

---

## How to Use This Plan

1. **Check the "Next Action" section** to see what to work on
2. **Update task status** as you progress (⬜ → 🚧 → ✅)
3. **Make commits frequently** - each task should be a separate commit
4. **Update "Commits Made" section** with commit hashes/messages
5. **Add notes** to the "Notes & Decisions" section as needed
6. **If blocked**, mark task as ⏸️ and document why in notes

This plan can be referenced across multiple chat contexts to maintain continuity.
