use std::rc::Rc;

use crate::phenotype::Phenotype;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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
    phenotype: Rc<Phenotype>,
    score: Mutex<Option<f64>>,
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
            phenotype: Rc::clone(&self.phenotype),
            score: Mutex::new(*self.score.lock().unwrap()),
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
        phenotype: Rc<Phenotype>,
        age: usize,
        parent_ids: (Option<usize>, Option<usize>),
    ) -> Self {
        Self {
            id: NEXT_ORGANISM_ID.fetch_add(1, Ordering::Relaxed),
            parent_ids,
            region_key: Mutex::new(None),
            score: Mutex::new(None),
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
    pub fn get_phenotype_rc(&self) -> Rc<Phenotype> {
        Rc::clone(&self.phenotype)
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
    pub fn score(&self) -> Option<f64> {
        *self.score.lock().unwrap()
    }

    /// Sets the score of the organism.
    pub fn set_score(&self, score: Option<f64>) {
        *self.score.lock().unwrap() = score;
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
    use std::rc::Rc;

    fn create_test_phenotype() -> Phenotype {
        let expressed_values = vec![1.0; NUM_SYSTEM_PARAMETERS];
        Phenotype::new_for_test(expressed_values)
    }

    #[test]
    fn given_new_organism_with_age_when_age_is_checked_then_it_is_correct() {
        let phenotype = Rc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 5, (None, None));
        assert_eq!(organism.age(), 5);
        assert!(!organism.is_dead());
    }

    #[test]
    fn given_organism_when_mark_dead_then_is_dead_returns_true() {
        let phenotype = Rc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));
        organism.mark_dead();
        assert!(organism.is_dead());
    }

    #[test]
    fn given_organism_created_with_new_when_parent_methods_called_then_returns_root_values() {
        let phenotype = Rc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0, (None, None));

        assert!(organism.is_root());
        assert_eq!(organism.parent_count(), 0);
        assert_eq!(organism.parent_ids(), (None, None));
    }

    #[test]
    fn given_organism_created_with_new_asexual_when_parent_methods_called_then_returns_correct_values()
     {
        let phenotype = Rc::new(create_test_phenotype());
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
        let phenotype = Rc::new(create_test_phenotype());
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
        let phenotype = Rc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 7, (None, None));

        assert!(organism.is_root());
        assert_eq!(organism.parent_count(), 0);
        assert_eq!(organism.parent_ids(), (None, None));
        assert_eq!(organism.age(), 7);
    }

    #[test]
    #[cfg(test)] // Only test Clone in test builds since it's test-only
    fn given_organism_when_cloned_then_parent_ids_are_copied() {
        let phenotype = Rc::new(create_test_phenotype());
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
}
