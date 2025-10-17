use std::sync::Arc;

use crate::phenotype::Phenotype;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

pub mod increment_age;
pub mod run;
pub mod update_region_key;

#[cfg(test)]
mod parent_tracking_tests;

// Global counter for unique organism IDs
static NEXT_ORGANISM_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
// Simulation entity that holds phenotype, spatial key, fitness score, age, and alive/dead status.
pub struct Organism {
    /// Unique identifier for this organism, assigned at creation
    id: usize,
    /// Parent IDs for pedigree tracking: (parent1_id, parent2_id)
    /// - Asexual reproduction: (Some(parent_id), None)
    /// - Sexual reproduction: (Some(parent1_id), Some(parent2_id))
    /// - Root/initial organisms: (None, None)
    parent_ids: (Option<usize>, Option<usize>),
    region_key: Mutex<Option<Vec<usize>>>,
    phenotype: Arc<Phenotype>,
    /// Fitness score stored as f64 bit representation (u64::MAX = None).
    /// Uses atomic operations for lock-free concurrent access during parallel processing.
    score: AtomicU64,
    /// The age of the organism, in ticks (atomic for thread-safe increments).
    age: AtomicUsize,
    /// Thread-safe flag indicating whether the organism has been marked as dead.
    is_dead: AtomicBool,
}

// Clone implementation is only available in test builds to avoid production bugs
// while allowing existing tests to continue working during the transition period.
// Production code should use organism references instead of cloning.
#[cfg(test)]
impl Clone for Organism {
    fn clone(&self) -> Self {
        // This should only be used in tests - create a new organism with new ID
        Self {
            id: NEXT_ORGANISM_ID.fetch_add(1, Ordering::Relaxed),
            parent_ids: self.parent_ids,
            region_key: Mutex::new(self.region_key.lock().unwrap().clone()),
            phenotype: Arc::clone(&self.phenotype),
            score: AtomicU64::new(self.score.load(Ordering::Acquire)),
            age: AtomicUsize::new(self.age.load(Ordering::Relaxed)),
            is_dead: AtomicBool::new(self.is_dead.load(Ordering::Relaxed)),
        }
    }
}

impl Organism {
    /// Creates a new `Organism` with parent tracking for pedigree.
    ///
    /// # Arguments
    ///
    /// * `phenotype` - The phenotype of the organism, wrapped in an `Rc` for shared ownership.
    /// * `age` - The initial age of the organism.
    /// * `parent_ids` - Parent IDs for pedigree tracking:
    ///   - Asexual reproduction: `(Some(parent_id), None)`
    ///   - Sexual reproduction: `(Some(parent1_id), Some(parent2_id))`
    ///   - Root/initial organisms: `(None, None)`
    pub fn new(
        phenotype: Arc<Phenotype>,
        age: usize,
        parent_ids: (Option<usize>, Option<usize>),
    ) -> Self {
        Self {
            id: NEXT_ORGANISM_ID.fetch_add(1, Ordering::Relaxed),
            parent_ids,
            region_key: Mutex::new(None),
            score: AtomicU64::new(u64::MAX), // u64::MAX represents None
            phenotype,
            age: AtomicUsize::new(age),
            is_dead: AtomicBool::new(false),
        }
    }

    /// Returns the unique ID of this organism.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns a reference to the organism's phenotype.
    pub fn phenotype(&self) -> &Phenotype {
        &self.phenotype
    }

    /// Returns a clone of the organism's phenotype Rc.
    pub fn get_phenotype_rc(&self) -> Arc<Phenotype> {
        Arc::clone(&self.phenotype)
    }

    /// Returns the region key of the organism, if set.
    pub fn region_key(&self) -> Option<Vec<usize>> {
        // TODO - this clone needs to ber gotten rid of
        self.region_key.lock().unwrap().clone()
    }

    /// Sets the region key of the organism.
    pub fn set_region_key(&self, region_key: Option<Vec<usize>>) {
        *self.region_key.lock().unwrap() = region_key;
    }

    /// Returns the score of the organism, if set.
    ///
    /// Uses atomic operations for lock-free concurrent access during parallel processing.
    /// This allows multiple threads to read scores simultaneously without contention.
    pub fn score(&self) -> Option<f64> {
        let bits = self.score.load(Ordering::Acquire);
        if bits == u64::MAX {
            None
        } else {
            Some(f64::from_bits(bits))
        }
    }

    /// Sets the score of the organism.
    ///
    /// Uses atomic operations for lock-free concurrent access. Thread-safe without locks,
    /// allowing parallel fitness evaluation across multiple threads.
    pub fn set_score(&self, score: Option<f64>) {
        let bits = score.map(|s| s.to_bits()).unwrap_or(u64::MAX);
        self.score.store(bits, Ordering::Release);
    }

    /// Returns the current age of the organism in ticks.
    ///
    /// This is a thread-safe operation that can be called from multiple threads.
    /// The age is the number of time units the organism has been alive.
    pub fn age(&self) -> usize {
        self.age.load(Ordering::Relaxed)
    }

    /// Marks the organism as dead. Thread-safe.
    pub fn mark_dead(&self) {
        self.is_dead.store(true, Ordering::Relaxed);
    }

    /// Returns `true` if the organism has been marked as dead.
    pub fn is_dead(&self) -> bool {
        self.is_dead.load(Ordering::Relaxed)
    }

    /// Returns the parent IDs for pedigree tracking.
    ///
    /// Returns a tuple `(parent1_id, parent2_id)` where:
    /// - Asexual reproduction: `(Some(parent_id), None)`
    /// - Sexual reproduction: `(Some(parent1_id), Some(parent2_id))`
    /// - Root/initial organisms: `(None, None)`
    pub fn parent_ids(&self) -> (Option<usize>, Option<usize>) {
        self.parent_ids
    }

    /// Returns `true` if this organism is a root organism (has no parents).
    pub fn is_root(&self) -> bool {
        matches!(self.parent_ids, (None, None))
    }

    /// Returns the number of parents this organism has (0, 1, or 2).
    pub fn parent_count(&self) -> usize {
        match self.parent_ids {
            (None, None) => 0,
            (Some(_), None) => 1,
            (Some(_), Some(_)) => 2,
            (None, Some(_)) => {
                unreachable!("Invalid parent configuration: second parent without first")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NUM_SYSTEM_PARAMETERS, phenotype::Phenotype};
    use std::sync::Arc;

    fn create_test_phenotype() -> Phenotype {
        let expressed_values = vec![1.0; NUM_SYSTEM_PARAMETERS];
        Phenotype::new_for_test(expressed_values)
    }

    #[test]
    fn given_new_organism_with_age_when_age_is_checked_then_it_is_correct() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 5, (None, None));
        assert_eq!(organism.age(), 5);
        assert!(!organism.is_dead());
    }

    #[test]
    fn given_organism_when_mark_dead_then_is_dead_returns_true() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.mark_dead();
        assert!(organism.is_dead());
    }

    #[test]
    fn given_organism_created_with_new_when_parent_methods_called_then_returns_root_values() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));

        assert!(organism.is_root());
        assert_eq!(organism.parent_count(), 0);
        assert_eq!(organism.parent_ids(), (None, None));
    }

    #[test]
    fn given_organism_created_with_new_asexual_when_parent_methods_called_then_returns_correct_values()
     {
        let phenotype = Arc::new(create_test_phenotype());
        let parent_id = 123;
        let organism = Organism::new(phenotype, 5, (Some(parent_id), None));

        assert!(!organism.is_root());
        assert_eq!(organism.parent_count(), 1);
        assert_eq!(organism.parent_ids(), (Some(parent_id), None));
        assert_eq!(organism.age(), 5);
    }

    #[test]
    fn given_organism_created_with_new_sexual_when_parent_methods_called_then_returns_correct_values()
     {
        let phenotype = Arc::new(create_test_phenotype());
        let parent1_id = 123;
        let parent2_id = 456;
        let organism = Organism::new(phenotype, 10, (Some(parent1_id), Some(parent2_id)));

        assert!(!organism.is_root());
        assert_eq!(organism.parent_count(), 2);
        assert_eq!(organism.parent_ids(), (Some(parent1_id), Some(parent2_id)));
        assert_eq!(organism.age(), 10);
    }

    #[test]
    fn given_organism_created_with_new_root_when_parent_methods_called_then_returns_root_values() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 7, (None, None));

        assert!(organism.is_root());
        assert_eq!(organism.parent_count(), 0);
        assert_eq!(organism.parent_ids(), (None, None));
        assert_eq!(organism.age(), 7);
    }

    #[test]
    #[cfg(test)] // Only test Clone in test builds since it's test-only
    fn given_organism_when_cloned_then_parent_ids_are_copied() {
        let phenotype = Arc::new(create_test_phenotype());
        let parent1_id = 789;
        let parent2_id = 101112;
        let original = Organism::new(phenotype, 3, (Some(parent1_id), Some(parent2_id)));

        let cloned = original.clone();

        // Clone should have same parent IDs but different organism ID
        assert_eq!(cloned.parent_ids(), original.parent_ids());
        assert_eq!(cloned.parent_count(), original.parent_count());
        assert_eq!(cloned.is_root(), original.is_root());
        assert_ne!(cloned.id(), original.id()); // Different ID due to cloning
    }

    // Tests for atomic score implementation (Solution 1 - lock-free score access)

    #[test]
    fn given_new_organism_when_score_checked_then_returns_none() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        assert_eq!(organism.score(), None);
    }

    #[test]
    fn given_score_set_when_get_score_then_returns_same_value() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(Some(42.5));
        assert_eq!(organism.score(), Some(42.5));
    }

    #[test]
    fn given_score_set_to_none_when_get_score_then_returns_none() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(Some(10.0));
        organism.set_score(None);
        assert_eq!(organism.score(), None);
    }

    #[test]
    fn given_negative_score_when_set_then_preserved() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(Some(-123.456));
        assert_eq!(organism.score(), Some(-123.456));
    }

    #[test]
    fn given_zero_score_when_set_then_preserved() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(Some(0.0));
        assert_eq!(organism.score(), Some(0.0));
    }

    #[test]
    fn given_very_small_score_when_set_then_preserved() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        let small_value = f64::EPSILON;
        organism.set_score(Some(small_value));
        assert_eq!(organism.score(), Some(small_value));
    }

    #[test]
    fn given_very_large_score_when_set_then_preserved() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        let large_value = f64::MAX / 2.0; // Use half of MAX to avoid overflow issues
        organism.set_score(Some(large_value));
        assert_eq!(organism.score(), Some(large_value));
    }

    #[test]
    fn given_concurrent_reads_when_score_set_then_all_see_value() {
        use std::thread;

        let phenotype = Arc::new(create_test_phenotype());
        let organism = Arc::new(Organism::new(phenotype, 0, (None, None)));
        organism.set_score(Some(99.9));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let org_clone = Arc::clone(&organism);
                thread::spawn(move || org_clone.score())
            })
            .collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), Some(99.9));
        }
    }

    #[test]
    fn given_concurrent_writes_when_scores_set_then_final_value_is_one_of_them() {
        use std::thread;

        let phenotype = Arc::new(create_test_phenotype());
        let organism = Arc::new(Organism::new(phenotype, 0, (None, None)));

        let test_values: Vec<f64> = (0..10).map(|i| i as f64 * 10.0).collect();

        let handles: Vec<_> = test_values
            .iter()
            .map(|&val| {
                let org_clone = Arc::clone(&organism);
                thread::spawn(move || org_clone.set_score(Some(val)))
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Final value should be one of the values we set
        let final_score = organism.score().expect("Score should be set");
        assert!(
            test_values.contains(&final_score),
            "Final score {} should be one of the set values",
            final_score
        );
    }

    #[test]
    fn given_infinity_score_when_set_then_preserved() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(Some(f64::INFINITY));
        assert_eq!(organism.score(), Some(f64::INFINITY));
    }

    #[test]
    fn given_negative_infinity_score_when_set_then_preserved() {
        let phenotype = Arc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.set_score(Some(f64::NEG_INFINITY));
        assert_eq!(organism.score(), Some(f64::NEG_INFINITY));
    }
}
