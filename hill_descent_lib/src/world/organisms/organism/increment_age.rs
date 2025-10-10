use super::Organism;
use std::sync::atomic::Ordering;

impl Organism {
    /// Atomically increments the organism's age by 1.
    ///
    /// After incrementing, checks if the new age exceeds the organism's maximum age
    /// as defined in its phenotype's system parameters. If so, the organism is marked as dead.
    /// This operation is thread-safe and can be called from multiple threads.
    pub fn increment_age(&self) {
        // Atomically increment age and fetch the new value (previous + 1)
        let new_age = self.age.fetch_add(1, Ordering::Relaxed) + 1;
        let max_age = self.phenotype.system_parameters().max_age();

        // If the new age exceeds the phenotype's max_age, mark the organism as dead
        if (new_age as f64) > max_age {
            crate::debug!(
                "Organism {} dying: age {} exceeds max_age {:.3}",
                self.id(),
                new_age,
                max_age
            );
            self.mark_dead();
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{NUM_SYSTEM_PARAMETERS, phenotype::Phenotype};
    use std::rc::Rc;

    fn phenotype_with_max_age(max_age: f64) -> Rc<Phenotype> {
        let mut expressed = vec![1.0; NUM_SYSTEM_PARAMETERS];
        expressed[7] = max_age; // system parameter index for max_age (m1,m2,m3,m4,m5,m6,m6_sigma,max_age,crossover)
        Rc::new(Phenotype::new_for_test(expressed))
    }

    #[test]
    fn given_organism_when_increment_age_then_age_increments() {
        let phenotype = phenotype_with_max_age(100.0);
        let organism = super::Organism::new(Rc::clone(&phenotype), 0, (None, None));
        organism.increment_age();
        assert_eq!(organism.age(), 1);
        assert!(!organism.is_dead());
    }

    #[test]
    fn given_organism_when_age_exceeds_max_then_mark_dead() {
        let phenotype = phenotype_with_max_age(0.0);
        let organism = super::Organism::new(Rc::clone(&phenotype), 0, (None, None));
        organism.increment_age();
        assert_eq!(organism.age(), 1);
        assert!(organism.is_dead());
    }
}
