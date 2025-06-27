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
        // If the new age exceeds the phenotype's max_age, mark the organism as dead
        if (new_age as f64) > self.phenotype.system_parameters().max_age() {
            self.mark_dead();
        }
    }
}
