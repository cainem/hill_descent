use rand::Rng;
use std::rc::Rc;

use crate::{
    phenotype::Phenotype,
    world::{organisms::organism::Organism, regions::region::Region},
};

type OrganismPairs = Vec<(Rc<Organism>, Rc<Organism>)>;

impl Region {
    /// Performs sexual reproduction for all provided organism pairs.
    ///
    /// Each pair produces two offspring through sexual reproduction.
    ///
    /// * `organism_pairs` - Vector of organism pairs for sexual reproduction
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns a vector of offspring organisms produced from sexual reproduction
    pub(super) fn perform_sexual_reproduction<R: Rng>(
        organism_pairs: &OrganismPairs,
        rng: &mut R,
    ) -> Vec<Organism> {
        let mut offspring = Vec::new();

        for (p1, p2) in organism_pairs {
            let (c1, c2) = Phenotype::sexual_reproduction(p1.phenotype(), p2.phenotype(), rng);
            offspring.push(Organism::new(
                Rc::new(c1),
                0,
                (Some(p1.id()), Some(p2.id())), // Sexual: two parents
            ));
            offspring.push(Organism::new(
                Rc::new(c2),
                0,
                (Some(p1.id()), Some(p2.id())), // Sexual: two parents
            ));
        }

        offspring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use rand::{SeedableRng, rngs::SmallRng};
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
    fn given_empty_pairs_when_perform_sexual_reproduction_then_returns_empty_offspring() {
        let pairs: OrganismPairs = vec![];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::perform_sexual_reproduction(&pairs, &mut rng);

        assert!(offspring.is_empty());
    }

    #[test]
    fn given_one_pair_when_perform_sexual_reproduction_then_returns_two_offspring() {
        let org1 = make_org(1.0, 5, 0);
        let org2 = make_org(2.0, 3, 1);
        let id1 = org1.id();
        let id2 = org2.id();
        let pairs = vec![(org1, org2)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::perform_sexual_reproduction(&pairs, &mut rng);

        assert_eq!(offspring.len(), 2);
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(id1), Some(id2)));
            assert!(!child.is_root());
        }
    }

    #[test]
    fn given_two_pairs_when_perform_sexual_reproduction_then_returns_four_offspring() {
        let org1 = make_org(1.0, 5, 0);
        let org2 = make_org(2.0, 3, 1);
        let org3 = make_org(3.0, 2, 2);
        let org4 = make_org(4.0, 1, 3);
        let pairs = vec![(org1, org2), (org3, org4)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::perform_sexual_reproduction(&pairs, &mut rng);

        assert_eq!(offspring.len(), 4);
        assert!(offspring.iter().all(|o| o.age() == 0));
        assert!(offspring.iter().all(|o| o.parent_count() == 2));
        assert!(offspring.iter().all(|o| !o.is_root()));
    }
}
