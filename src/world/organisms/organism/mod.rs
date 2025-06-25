use std::rc::Rc;

use crate::phenotype::Phenotype;

pub mod run;
pub mod update_region_key;

#[derive(Debug, Clone)]
pub struct Organism {
    region_key: Option<Vec<usize>>,
    phenotype: Rc<Phenotype>,
    score: Option<f64>,
    /// The age of the organism, in ticks.
    age: usize,
}

impl Organism {
    /// Creates a new `Organism`.
    ///
    /// # Arguments
    ///
    /// * `phenotype` - The phenotype of the organism, wrapped in an `Rc` for shared ownership.
    /// * `age` - The initial age of the organism.
    pub fn new(phenotype: Rc<Phenotype>, age: usize) -> Self {
        Self {
            region_key: None,
            score: None,
            phenotype,
            age,
        }
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
    pub fn region_key(&self) -> Option<&Vec<usize>> {
        self.region_key.as_ref()
    }

    /// Sets the region key of the organism.
    pub fn set_region_key(&mut self, region_key: Option<Vec<usize>>) {
        self.region_key = region_key;
    }

    /// Returns the score of the organism, if set.
    pub fn score(&self) -> Option<f64> {
        self.score
    }

    /// Sets the score of the organism.
    pub fn set_score(&mut self, score: Option<f64>) {
        self.score = score;
    }

    /// Returns the age of the organism.
    pub fn age(&self) -> usize {
        self.age
    }

    /// Increments the age of the organism by one.
    pub fn increment_age(&mut self) {
        self.age += 1;
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
        let organism = Organism::new(phenotype, 5);
        assert_eq!(organism.age(), 5);
    }

    #[test]
    fn given_organism_when_increment_age_is_called_then_age_is_incremented() {
        let phenotype = Rc::new(create_test_phenotype());
        let mut organism = Organism::new(phenotype, 10);
        organism.increment_age();
        assert_eq!(organism.age(), 11);
        organism.increment_age();
        assert_eq!(organism.age(), 12);
    }
}
