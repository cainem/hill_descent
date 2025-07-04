use super::{Organism, Organisms};
use std::rc::Rc;

impl Organisms {
    /// Returns a reference to the organism with the **highest** fitness score.
    ///
    /// If no organism in the collection has a score (`score() == None`), the
    /// function returns `None`.
    pub fn best(&self) -> Option<Rc<Organism>> {
        self.organisms
            .iter()
            .filter_map(|o| o.score().map(|s| (o, s)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(o, _)| Rc::clone(o))
    }
}

#[cfg(test)]
mod tests {
    use super::Organisms;
    use crate::phenotype::Phenotype;
    use crate::world::organisms::organism::Organism;
    use std::rc::Rc;

    // Helper to create an organism with a given score
    fn make_scored_organism(score: f64) -> Organism {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0];
        let phenotype = Rc::new(Phenotype::new_for_test(expressed));
        let org = Organism::new(Rc::clone(&phenotype), 0);
        org.set_score(Some(score));
        org
    }

    #[test]
    fn given_multiple_scored_organisms_when_best_then_returns_highest() {
        let o1 = make_scored_organism(1.0);
        let o2 = make_scored_organism(10.0);
        let o3 = make_scored_organism(5.0);
        let orgs = Organisms::new_from_organisms(vec![o1.clone(), o2.clone(), o3.clone()]);

        let best_rc = orgs.best().unwrap();
        let best = best_rc.as_ref();
        assert_eq!(best.score(), Some(10.0));
    }

    #[test]
    fn given_no_scores_when_best_then_returns_none() {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0];
        let phenotype = Rc::new(Phenotype::new_for_test(expressed));
        let org1 = Organism::new(Rc::clone(&phenotype), 0);
        let org2 = org1.clone();
        let orgs = Organisms::new_from_organisms(vec![org1, org2]);

        assert!(orgs.best().is_none());
    }
}
