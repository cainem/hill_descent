use super::Organisms;

impl Organisms {
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self))
    )]
    pub fn increment_ages(&self) {
        for organism in self.organisms.iter() {
            organism.increment_age();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Organism, Organisms};
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::phenotype::Phenotype;
    use std::rc::Rc;

    fn create_test_organisms(count: usize, max_age: f64) -> Organisms {
        let mut expressed = vec![0.1; NUM_SYSTEM_PARAMETERS];
        expressed[7] = max_age; // system parameter index for max_age (m1,m2,m3,m4,m5,m6,m6_sigma,max_age,crossover)
        let phenotype = Rc::new(Phenotype::new_for_test(expressed));
        let organisms: Vec<Organism> = (0..count)
            .map(|_| Organism::new(Rc::clone(&phenotype), 0, (None, None)))
            .collect();
        Organisms::new_from_organisms(organisms)
    }

    #[test]
    fn given_multiple_organisms_when_increment_ages_then_all_ages_increment() {
        let orgs = create_test_organisms(3, 10.0);
        orgs.increment_ages();
        assert!(orgs.iter().all(|o| o.age() == 1));
        assert!(orgs.iter().all(|o| !o.is_dead()));
    }

    #[test]
    fn given_organism_exceeding_max_age_when_increment_ages_then_marked_dead() {
        let orgs = create_test_organisms(1, 0.0);
        orgs.increment_ages();
        let organism = orgs.iter().next().unwrap();
        assert_eq!(organism.age(), 1);
        assert!(organism.is_dead());
    }
}
