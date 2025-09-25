use std::rc::Rc;

use crate::world::{organisms::organism::Organism, regions::region::Region};

type OrganismPairs = Vec<(Rc<Organism>, Rc<Organism>)>;
type PairingResult = (OrganismPairs, Option<Rc<Organism>>);

impl Region {
    /// Pairs organisms for reproduction based on their count.
    ///
    /// If the number of organisms is odd, the first organism is set aside for asexual reproduction.
    /// The remaining organisms are paired sequentially for sexual reproduction.
    ///
    /// * `selected_organisms` - The slice of organisms to pair for reproduction
    ///
    /// Returns a tuple containing:
    /// * A vector of organism pairs for sexual reproduction
    /// * An optional single organism for asexual reproduction
    pub(super) fn pair_organisms_for_reproduction(
        selected_organisms: &[Rc<Organism>],
    ) -> PairingResult {
        let mut pairs = Vec::new();
        let single_organism = if selected_organisms.len() % 2 == 1 {
            Some(Rc::clone(&selected_organisms[0]))
        } else {
            None
        };

        // Determine starting index for pairing (skip first if it's used for asexual reproduction)
        let sexual_start = if single_organism.is_some() { 1 } else { 0 };

        // Pair organisms sequentially for sexual reproduction
        for chunk in selected_organisms[sexual_start..].chunks(2) {
            if let [p1, p2] = chunk {
                pairs.push((Rc::clone(p1), Rc::clone(p2)));
            }
        }

        (pairs, single_organism)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
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
    fn given_empty_organisms_when_pair_organisms_then_returns_empty_pairs_and_no_single() {
        let organisms: Vec<Rc<Organism>> = vec![];

        let (pairs, single) = Region::pair_organisms_for_reproduction(&organisms);

        assert!(pairs.is_empty());
        assert!(single.is_none());
    }

    #[test]
    fn given_single_organism_when_pair_organisms_then_returns_empty_pairs_and_single() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let organism_id = organisms[0].id();

        let (pairs, single) = Region::pair_organisms_for_reproduction(&organisms);

        assert!(pairs.is_empty());
        assert!(single.is_some());
        assert_eq!(single.unwrap().id(), organism_id);
    }

    #[test]
    fn given_two_organisms_when_pair_organisms_then_returns_one_pair_and_no_single() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let id1 = organisms[0].id();
        let id2 = organisms[1].id();

        let (pairs, single) = Region::pair_organisms_for_reproduction(&organisms);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.id(), id1);
        assert_eq!(pairs[0].1.id(), id2);
        assert!(single.is_none());
    }

    #[test]
    fn given_three_organisms_when_pair_organisms_then_returns_one_pair_and_first_single() {
        let organisms = vec![
            make_org(1.0, 5, 0),
            make_org(2.0, 3, 1),
            make_org(3.0, 2, 2),
        ];
        let id1 = organisms[0].id();
        let id2 = organisms[1].id();
        let id3 = organisms[2].id();

        let (pairs, single) = Region::pair_organisms_for_reproduction(&organisms);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.id(), id2); // Second organism becomes first in pair
        assert_eq!(pairs[0].1.id(), id3); // Third organism becomes second in pair
        assert!(single.is_some());
        assert_eq!(single.unwrap().id(), id1); // First organism is single
    }

    #[test]
    fn given_four_organisms_when_pair_organisms_then_returns_two_pairs_and_no_single() {
        let organisms = vec![
            make_org(1.0, 5, 0),
            make_org(2.0, 3, 1),
            make_org(3.0, 2, 2),
            make_org(4.0, 1, 3),
        ];
        let id1 = organisms[0].id();
        let id2 = organisms[1].id();
        let id3 = organisms[2].id();
        let id4 = organisms[3].id();

        let (pairs, single) = Region::pair_organisms_for_reproduction(&organisms);

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0.id(), id1);
        assert_eq!(pairs[0].1.id(), id2);
        assert_eq!(pairs[1].0.id(), id3);
        assert_eq!(pairs[1].1.id(), id4);
        assert!(single.is_none());
    }
}
