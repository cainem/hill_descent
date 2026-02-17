pub mod new_random_gamete;
pub mod reproduce;

use crate::locus::Locus;
use std::cell::RefCell;

/// Minimum gamete capacity (in loci) for pool eligibility.
/// Gametes smaller than this threshold are cheaply allocated and freed by the
/// system allocator, so pooling overhead is not justified.
const MIN_POOL_CAPACITY: usize = 1_000;

/// Maximum number of buffers retained per thread in the pool.
/// Prevents unbounded memory growth from cross-thread drops (e.g. when gametes
/// created on Rayon workers are dropped on the main thread).
const MAX_POOL_ENTRIES: usize = 50;

thread_local! {
    /// Thread-local pool of `Vec<Locus>` buffers for reuse by `Gamete::reproduce`.
    ///
    /// When a large `Gamete` is dropped, its backing `Vec<Locus>` is returned to
    /// this pool (subject to capacity and size thresholds). When `reproduce` needs
    /// a new buffer, it takes one from the pool instead of allocating, avoiding
    /// heap allocation in the steady state.
    static LOCI_POOL: RefCell<Vec<Vec<Locus>>> = const { RefCell::new(Vec::new()) };
}

/// A gamete is a string of loci contributed by a parent organism.
#[derive(Debug, Clone, PartialEq)]
pub struct Gamete {
    /// The list of loci for this gamete, one per genetic dimension.
    loci: Vec<Locus>,
}

impl Gamete {
    /// Creates a new Gamete from a vector of loci.
    pub fn new(loci: Vec<Locus>) -> Self {
        Self { loci }
    }

    /// Takes a pre-allocated loci buffer from the thread-local pool,
    /// or creates a new one with the given capacity if the pool is empty.
    ///
    /// Only uses the pool for capacities at or above `MIN_POOL_CAPACITY`.
    /// The returned buffer has length 0 but may have capacity greater than
    /// or equal to `capacity` (from a previous gamete of equal or larger size).
    pub(crate) fn take_buffer(capacity: usize) -> Vec<Locus> {
        if capacity >= MIN_POOL_CAPACITY {
            LOCI_POOL.with(|pool| {
                let mut buffer = pool
                    .borrow_mut()
                    .pop()
                    .unwrap_or_else(|| Vec::with_capacity(capacity));
                if buffer.capacity() < capacity {
                    buffer.reserve(capacity - buffer.capacity());
                }
                buffer
            })
        } else {
            Vec::with_capacity(capacity)
        }
    }

    /// Returns a slice of loci.
    pub fn loci(&self) -> &[Locus] {
        &self.loci
    }

    /// Consumes the gamete and returns the underlying loci.
    ///
    /// The returned `Vec<Locus>` is NOT returned to the thread-local pool;
    /// the caller takes full ownership.
    pub fn into_loci(mut self) -> Vec<Locus> {
        std::mem::take(&mut self.loci)
    }

    /// Returns the number of loci in this gamete.
    pub fn len(&self) -> usize {
        self.loci.len()
    }

    /// Returns true if this gamete contains no loci.
    pub fn is_empty(&self) -> bool {
        self.loci.is_empty()
    }
}

impl Drop for Gamete {
    fn drop(&mut self) {
        // Only pool large buffers where the allocation cost justifies pool overhead.
        if self.loci.capacity() >= MIN_POOL_CAPACITY {
            let mut buf = std::mem::take(&mut self.loci);
            buf.clear();
            // Return buffer to thread-local pool for reuse, subject to pool cap.
            // try_with handles the case where the thread-local is being destroyed
            // during thread shutdown.
            let _ = LOCI_POOL.try_with(|pool| {
                let mut pool = pool.borrow_mut();
                if pool.len() < MAX_POOL_ENTRIES {
                    pool.push(buf);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameters::parameter::Parameter;

    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    #[test]
    fn given_empty_loci_when_new_then_len_is_zero_and_is_empty() {
        let loci = vec![];
        let gamete = Gamete::new(loci);
        assert_eq!(gamete.len(), 0);
        assert!(gamete.is_empty());
        assert!(gamete.loci().is_empty());
    }

    #[test]
    fn given_non_empty_loci_when_new_then_len_and_accessors_work() {
        let loci = vec![create_test_locus(1.0), create_test_locus(2.0)];
        let gamete = Gamete::new(loci.clone());
        assert_eq!(gamete.len(), 2);
        assert!(!gamete.is_empty());
        assert_eq!(gamete.loci(), loci.as_slice());
        assert_eq!(gamete.into_loci(), loci);
    }

    #[test]
    fn given_small_gamete_when_dropped_then_buffer_not_pooled() {
        // Gametes below MIN_POOL_CAPACITY should not be pooled.
        let initial_pool_len = LOCI_POOL.with(|pool| pool.borrow().len());

        {
            let loci: Vec<Locus> = (0..10).map(|i| create_test_locus(i as f64)).collect();
            let _gamete = Gamete::new(loci);
            // _gamete dropped here
        }

        let after_pool_len = LOCI_POOL.with(|pool| pool.borrow().len());
        assert_eq!(
            after_pool_len, initial_pool_len,
            "Small gamete buffer should not be added to pool"
        );
    }

    #[test]
    fn given_large_gamete_when_dropped_then_buffer_is_pooled() {
        // Gametes at or above MIN_POOL_CAPACITY should be pooled.
        let initial_pool_len = LOCI_POOL.with(|pool| pool.borrow().len());

        {
            let loci: Vec<Locus> = (0..MIN_POOL_CAPACITY)
                .map(|i| create_test_locus(i as f64))
                .collect();
            let _gamete = Gamete::new(loci);
            // _gamete dropped here
        }

        let after_pool_len = LOCI_POOL.with(|pool| pool.borrow().len());
        assert_eq!(
            after_pool_len,
            initial_pool_len + 1,
            "Large gamete buffer should be returned to pool"
        );
    }

    #[test]
    fn given_take_buffer_when_pool_has_entry_then_reuses_buffer() {
        // Seed the pool with a known buffer.
        let capacity = MIN_POOL_CAPACITY + 100;
        let buf = Vec::with_capacity(capacity);
        LOCI_POOL.with(|pool| pool.borrow_mut().push(buf));

        let reused = Gamete::take_buffer(MIN_POOL_CAPACITY);
        assert!(
            reused.capacity() >= capacity,
            "take_buffer should return a pooled buffer with retained capacity"
        );
        assert_eq!(reused.len(), 0, "Pooled buffer should be empty");
    }

    #[test]
    fn given_take_buffer_when_pool_empty_then_allocates_new() {
        // Drain the pool first.
        LOCI_POOL.with(|pool| pool.borrow_mut().clear());

        let buf = Gamete::take_buffer(MIN_POOL_CAPACITY);
        assert!(
            buf.capacity() >= MIN_POOL_CAPACITY,
            "New buffer should have at least the requested capacity"
        );
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn given_pool_at_max_capacity_when_gamete_dropped_then_buffer_freed() {
        // Fill pool to MAX_POOL_ENTRIES.
        LOCI_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            pool.clear();
            for _ in 0..MAX_POOL_ENTRIES {
                pool.push(Vec::with_capacity(MIN_POOL_CAPACITY));
            }
        });

        {
            let loci: Vec<Locus> = (0..MIN_POOL_CAPACITY)
                .map(|i| create_test_locus(i as f64))
                .collect();
            let _gamete = Gamete::new(loci);
            // _gamete dropped here — pool is full, so buffer is freed normally
        }

        let pool_len = LOCI_POOL.with(|pool| pool.borrow().len());
        assert_eq!(
            pool_len, MAX_POOL_ENTRIES,
            "Pool should not exceed MAX_POOL_ENTRIES"
        );

        // Clean up to avoid affecting other tests.
        LOCI_POOL.with(|pool| pool.borrow_mut().clear());
    }

    #[test]
    fn given_into_loci_when_called_then_buffer_not_pooled() {
        // into_loci should give ownership to the caller, not the pool.
        let initial_pool_len = LOCI_POOL.with(|pool| pool.borrow().len());

        let loci: Vec<Locus> = (0..MIN_POOL_CAPACITY)
            .map(|i| create_test_locus(i as f64))
            .collect();
        let gamete = Gamete::new(loci);

        let returned_loci = gamete.into_loci();
        assert_eq!(returned_loci.len(), MIN_POOL_CAPACITY);

        let after_pool_len = LOCI_POOL.with(|pool| pool.borrow().len());
        assert_eq!(
            after_pool_len, initial_pool_len,
            "into_loci should not add buffer to pool"
        );
    }

    #[test]
    fn given_take_buffer_when_below_min_capacity_then_skips_pool() {
        // Seed pool with a buffer.
        LOCI_POOL.with(|pool| {
            pool.borrow_mut()
                .push(Vec::with_capacity(MIN_POOL_CAPACITY));
        });
        let pool_before = LOCI_POOL.with(|pool| pool.borrow().len());

        // Request a buffer below the threshold — should NOT take from pool.
        let buf = Gamete::take_buffer(10);
        assert_eq!(buf.capacity(), 10);

        let pool_after = LOCI_POOL.with(|pool| pool.borrow().len());
        assert_eq!(
            pool_after, pool_before,
            "take_buffer below MIN_POOL_CAPACITY should not drain the pool"
        );

        // Clean up.
        LOCI_POOL.with(|pool| pool.borrow_mut().clear());
    }
}
