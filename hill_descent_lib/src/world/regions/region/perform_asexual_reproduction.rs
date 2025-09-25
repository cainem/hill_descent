use rand::Rng;
use std::rc::Rc;

use crate::world::{organisms::organism::Organism, regions::region::Region};

impl Region {
    /// Performs asexual reproduction for a single organism.
    ///
    /// * `organism` - The organism to reproduce asexually
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns the offspring organism produced from asexual reproduction
    pub(super) fn perform_asexual_reproduction<R: Rng>(
        organism: &Rc<Organism>,
        rng: &mut R,
    ) -> Organism {
        let child_pheno = organism.phenotype().asexual_reproduction(rng);
        Organism::new(
            Rc::new(child_pheno),
            0,
            (Some(organism.id()), None), // Asexual: one parent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::rc::Rc;

    /// Helper: create an Organism with given score and age.
    fn make_org(score: f64, age: usize, idx: usize) -> Rc<Organism> {
        // Expressed values: default 7 system parameters + one dummy problem param
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, idx as f64];
        let phenotype = Rc::new(Phenotype::new_for_test(expressed));
        let org = Organism::new(Rc::clone(&phenotype), age, (None, None));
        org.set_score(Some(score));
        Rc::new(org)
    }

    #[test]
    fn given_organism_when_perform_asexual_reproduction_then_returns_one_offspring() {
        let parent = make_org(1.0, 5, 0);
        let parent_id = parent.id();
        let mut rng = SmallRng::seed_from_u64(0);

        let child = Region::perform_asexual_reproduction(&parent, &mut rng);

        assert_eq!(child.age(), 0);
        assert_eq!(child.parent_count(), 1);
        assert_eq!(child.parent_ids(), (Some(parent_id), None));
        assert!(!child.is_root());
    }
}