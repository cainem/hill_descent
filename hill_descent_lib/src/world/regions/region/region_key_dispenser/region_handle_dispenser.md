# The RegionHandleDispenser

The `RegionHandleDispenser` manages compact handles for regions in an n‑dimensional grid. It maps a potentially large `full_region_key: Vec<usize>` (one index per dimension) to a small, stable handle `RegionHandle { id, adjacent_zone_id }` that is cheaper to store and compare.

## Overview

- The full key (`full_region_key`) can be large (one element per dimension) and is unsuitable for widespread use as a map key in many places.
- The dispenser keeps an internal mapping `BTreeMap<Vec<usize>, RegionHandle>` from the full key to a compact handle:
  - `id: usize` — unique, monotonic identifier for the region instance
  - `adjacent_zone_id: usize` — identifier for the connected component (zone) of spatially adjacent regions
- Determinism is important; we deliberately use `BTreeMap` for deterministic iteration and ordering.

## Adjacency definition

Two regions are adjacent if their `full_region_key`s differ by exactly one coordinate by ±1, and all other coordinates are identical.

Example: `a = [4,5,6,7,8]` is adjacent to `b = [4,5,6,7,9]` (only the last coordinate differs by +1).

## Lifecycle

### Reset
When regions are recalculated, existing keys become invalid. Calling `reset()`:
- Clears the internal map
- Resets `next_id` and `next_adjacent_zone_id` to 0

### Insertion via get_or_insert
`get_or_insert(full_region_key: &[usize]) -> RegionHandle` is the single entry point:
1. If `full_region_key` already exists, return its `RegionHandle`.
2. Otherwise, generate all 2*d neighbor keys by applying ±1 to each coordinate in turn and probe the map for neighbors.
3. Determine `adjacent_zone_id`:
   - No neighbors found: allocate a new `adjacent_zone_id` using `next_adjacent_zone_id`, then increment it.
   - One neighbor zone: reuse that zone id.
   - Multiple neighbor zones: merge them into a single canonical zone id — specifically the minimum zone id (deterministic policy). All affected entries are relabeled to the canonical id.
4. Allocate a new `id` using `next_id`, increment it, insert the mapping, and return the new `RegionHandle`.

## Adjacent zone merging
If a new region is adjacent to multiple existing regions with different `adjacent_zone_id`s, those zones must be merged. The dispenser:
- Selects the minimum `adjacent_zone_id` among the neighbors as the canonical id (deterministic choice)
- Relabels all entries that belonged to other involved zone ids to the canonical id
- Note: zone ids are not reused after merges; gaps are acceptable and intentional for simplicity

## Motivation for adjacent zones
Future carrying capacity allocation can first share capacity among zones (e.g., proportionally to zone size squared), then distribute within zones based on region scores. This delays prematurely excluding alternatives by encouraging diversity across connected regions.

Example zone weighting by size squared: if zones A, B, C have sizes 1, 2, 3 then total weight is `1^2 + 2^2 + 3^2 = 14`, so shares are 1/14, 4/14, and 9/14 respectively.

## API summary (scaffold)
- `RegionHandle` (private fields, with getters): `{ id, adjacent_zone_id }`
- `RegionHandleDispenser` state: `BTreeMap<Vec<usize>, RegionHandle>`, `next_id`, `next_adjacent_zone_id`
- Methods (to be implemented):
  - `reset(&mut self)`
  - `get_or_insert(&mut self, full_region_key: &[usize]) -> RegionHandle`
  - `get_adjacent_full_keys(&self, full_region_key: &[usize]) -> Vec<Vec<usize>>`
  - `merge_zones(&mut self, zones_to_merge: Vec<usize>)`
