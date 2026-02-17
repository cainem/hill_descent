# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.3.1] - 2026-02-17
### Changed
- Reduced allocation overhead in gamete reproduction by reusing pooled locus buffers.
- Reduced allocation overhead in phenotype expression by reusing pooled expressed-value buffers.

### Internal
- Added thread-local buffer pools with bounded retention to avoid unbounded memory growth.
- Refactored phenotype expression internals to support writing into caller-provided buffers.

## [0.3.0] - 2026-02-04
### Changed
- Significant performance improvements across genetic operations (~20-40% faster execution)
- Optimized mutation operations by pre-calculating Bernoulli distributions
- Improved LocusAdjustment to avoid redundant rebuilds when values are unchanged
- Enhanced Arc usage and reduced unnecessary clones in reproduction
- Better thread pool load balancing and synchronization
- Implemented incremental region key updates for more efficient spatial indexing

### Internal
- Added MutationDistributions struct for caching Bernoulli distributions (internal only)
- Optimized gamete reproduction, locus mutation, and phenotype expression
- Improved region processing and organism management performance

## [0.2.0] - 2025-11-21
### Added
- Introduced the `TrainingData` enum to unify how supervised and unsupervised runs are configured.
- Added `World::get_best_params` and expanded documentation covering neural-network sized problems, scaling guidance, and API usage patterns.
- Added the `format_score` helper for consistent presentation of unevaluated scores in logs and the UI.
- Created comprehensive release notes and improvement plan documentation for future contributors.

### Changed
- Refactored `World::training_run` and `World::get_best_organism` to accept `TrainingData`, removing the old multi-slice signature and simplifying validation.
- Updated all integration tests, examples, server endpoints, and benchmarks to the new API.
- Migrated the regions subsystem to the `RegionKey` type end-to-end, eliminating ad-hoc `Vec<usize>` keys and tightening determinism.
- Optimized `RegionKey` updates to use in-place modification, reducing memory allocations during organism movement.
- Improved carrying capacity calculations, refill logic, and min-score handling to match the refined region model.
- Expanded `pdd.md` with a new public API section and refreshed README content with detailed ML and scaling guidance.

### Fixed
- Corrected initial score display to show `<not yet evaluated>` instead of very large floating point values.
- Eliminated clippy warnings across the workspace and resolved assorted doc/test inconsistencies uncovered during the API redesign.

## [0.1.0] - Initial release
- First public release of the optimization library and visualization stack.
