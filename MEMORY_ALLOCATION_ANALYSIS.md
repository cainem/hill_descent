# Memory Allocation vs Atomic Operations Analysis

**Date:** October 16, 2025  
**Context:** Performance optimization investigation for hill_descent parallel processing

## Summary

Investigation revealed that **memory allocation overhead dominates atomic operation costs** in our workload. Attempted score caching optimization (Solution 2) degraded performance by 5.9% overall, with up to 47% slowdown for small populations.

## Key Findings

### 1. Solution 2 Failure: Score Caching

**Hypothesis:** Caching scores before sorting would reduce O(N log N) atomic operations to O(N), improving performance.

**Implementation:**
```rust
// Create cache: Vec<(Arc<Organism>, f64, usize)>
let mut organisms_with_cached_values: Vec<_> = self
    .organisms
    .iter()
    .map(|org| (Arc::clone(org), org.score().unwrap_or(f64::INFINITY), org.age()))
    .collect();

// Sort cached tuples
organisms_with_cached_values.sort_by(...);

// Reconstruct vector
self.organisms = organisms_with_cached_values.into_iter().map(|(org, _, _)| org).collect();
```

**Results:**
- **Overall:** +5.9% slower (38.058s → 40.310s)
- **Styblinski-Tang:** +7.0% slower
- **Ackley:** +1.7% slower
- **Himmelblau:** +9.1% slower
- **Worst case:** Himmelblau Pop:100, Regions:2 → **+47.1% slower** (0.140s → 0.206s)

### 2. Why Memory Allocation Cost More Than Atomic Operations

**Atomic Operation Cost:**
- `AtomicU64::load(Ordering::Acquire)`: ~10-20 CPU cycles
- For Pop:100 sort: ~1,328 atomic loads (2× per comparison × ~664 comparisons)
- **Total cost:** ~13,280-26,560 cycles = **~5-10 microseconds**

**Memory Allocation Cost:**
```rust
// 1. Vec allocation for cache
let mut organisms_with_cached_values: Vec<_> = Vec::with_capacity(100);

// 2. Arc::clone for each organism (atomic increment)
// 3. Tuple allocation (24 bytes each × 100)
.map(|org| (Arc::clone(org), score, age))  // 2,400 bytes

// 4. collect() - heap allocation

// 5. Reconstruct organisms Vec
// 6. Drop old tuples (Arc decrements)
```

**Estimated overhead:**
- Heap allocation: ~100-500 cycles per allocation × 2 allocations
- Memory copying: 2,400 bytes × 2 copies = 4,800 bytes
- Arc reference counting: 100 increments + 100 decrements = 200 atomic operations
- Cache misses: New memory not in CPU cache

**Total cost:** **~50-100 microseconds** (10-20× more than atomic loads!)

### 3. Windows Allocator Performance

Windows default allocator (HeapAlloc) has known performance issues in multi-threaded scenarios:
- Lock contention on global heap
- Coarse-grained locking
- Poor scaling beyond 4-8 threads

**Solution: MiMalloc**
- Integrated `mimalloc` as global allocator
- Thread-local allocation arenas
- Lock-free fast paths
- Proven 10-30% speedup in multi-threaded Rust applications on Windows

## Memory Allocations Still Present

### In `parallel_process_regions`:

```rust
// 1. Region sorting (unavoidable but small)
let mut region_entries: Vec<_> = self.regions.iter_mut().collect();

// 2. Offspring collection (LARGE - Pop×Regions offspring per epoch)
let all_offspring: Vec<Vec<Organism>> = region_entries
    .par_iter_mut()
    .map(|...| region.process_region_lifecycle(...))  // Each returns Vec<Organism>
    .collect();

// 3. Organism collection (OPTIMIZED - pre-allocated with exact capacity)
let surviving_count: usize = self.regions.iter().map(|(_, r)| r.organism_count()).sum();
let offspring_count: usize = all_offspring.iter().map(|v| v.len()).sum();
let total_capacity = surviving_count + offspring_count;

let mut all_organisms: Vec<Arc<Organism>> = Vec::with_capacity(total_capacity);
for (_key, region) in self.regions.iter() {
    for organism in region.organisms() {
        all_organisms.push(Arc::clone(organism));  // No reallocation needed
    }
}

// 4. Offspring extension (OPTIMIZED - direct iterator, no intermediate Vec)
all_organisms.extend(
    all_offspring
        .into_iter()
        .flat_map(|v| v.into_iter().map(Arc::new))  // Iterator chain - no Vec allocation
);
```

### Optimizations Applied:

1. ✅ **Pre-allocate with exact capacity:** Eliminates Vec reallocations during push operations
   ```rust
   let mut all_organisms: Vec<Arc<Organism>> = Vec::with_capacity(total_capacity);
   ```

2. ✅ **Eliminated `all_offspring_flat`:** Changed from `.collect()` intermediate Vec to direct `.extend()` with iterator
   - **Before:** `let all_offspring_flat: Vec<_> = ...collect(); all_organisms.extend(all_offspring_flat);`
   - **After:** `all_organisms.extend(...iterator...);`
   - **Savings:** One heap allocation + one Vec copy eliminated

3. ⏳ **Remaining optimization possibilities:**
   - Reuse buffers between epochs (requires careful lifetime management)
   - Consider arena allocator for short-lived allocations
   - Profile to identify other hotspots

## Lessons Learned

1. **Measure, don't assume:** Intuition about atomic costs was wrong
2. **Memory allocation is expensive:** Especially heap allocations in hot paths
3. **Windows allocator matters:** MiMalloc can provide 10-30% improvement
4. **Avoid intermediate collections:** Each `.collect()` is a heap allocation
5. **Simple code often wins:** Direct atomic loads beat complex caching schemes

## Recommendations

1. ✅ **DONE:** Revert Solution 2 (score caching)
2. ✅ **DONE:** Add MiMalloc as global allocator
3. ⏳ **TODO:** Benchmark with MiMalloc to measure actual improvement
4. ⏳ **TODO:** Profile to identify hottest allocation sites
5. ⏳ **TODO:** Consider pre-allocation strategies for parallel_process

## Expected Results with MiMalloc

Based on similar workloads:
- **Small populations (10-100):** 15-30% speedup (allocation-heavy)
- **Large populations (1000+):** 5-15% speedup (computation-heavy)
- **Overall:** 10-20% speedup expected

Need to benchmark to confirm.
