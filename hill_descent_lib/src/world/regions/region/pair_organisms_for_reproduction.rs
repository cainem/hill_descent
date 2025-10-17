use std::sync::Arc;

use crate::world::{organisms::organism::Organism, regions::region::Region};

type OrganismPairs = Vec<(Arc<Organism>, Arc<Organism>)>;

impl Region {
    /// Pairs organisms for reproduction using extreme pairing strategy.
    ///
    /// For even counts: Pairs first with last, second with second-to-last, etc.
    /// For odd counts: Duplicates the top performer, then applies extreme pairing.
    /// This eliminates asexual reproduction - even single organisms pair with themselves.
    ///
    /// * `selected_organisms` - The slice of organisms to pair for reproduction
    ///
    /// Returns a vector of organism pairs for sexual reproduction
    pub(super) fn pair_organisms_for_reproduction(
        selected_organisms: &[Arc<Organism>],
    ) -> OrganismPairs {
        let mut pairs = Vec::new();

        if selected_organisms.is_empty() {
            return pairs;
        }

        if selected_organisms.len() % 2 == 1 {
            // Odd count: duplicate the top performer and then use extreme pairing
            let mut working_list = Vec::with_capacity(selected_organisms.len() + 1);
            working_list.push(Arc::clone(&selected_organisms[0])); // First copy of top performer
            working_list.extend(selected_organisms.iter().cloned()); // Original list including top performer again

            // Pair using extreme strategy: first with last, second with second-to-last, etc.
            let len = working_list.len();
            for i in 0..(len / 2) {
                let first = Arc::clone(&working_list[i]);
                let last = Arc::clone(&working_list[len - 1 - i]);
                pairs.push((first, last));
            }
        } else {
            // Even count: directly pair from the slice without creating a vector
            let len = selected_organisms.len();
            for i in 0..(len / 2) {
                let first = Arc::clone(&selected_organisms[i]);
                let last = Arc::clone(&selected_organisms[len - 1 - i]);
                pairs.push((first, last));
            }
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use std::sync::Arc;

    /// Helper: create an Organism with given score and age.
    fn make_org(score: f64, age: usize, idx: usize) -> Arc<Organism> {
        // Expressed values: default 7 system parameters + one dummy problem param
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, idx as f64];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        let org = Organism::new(Arc::clone(&phenotype), age, (None, None));
        org.set_score(Some(score));
        Arc::new(org)
    }

    #[test]
    fn given_empty_organisms_when_pair_organisms_then_returns_empty_pairs() {
        let organisms: Vec<Arc<Organism>> = vec![];

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        assert!(pairs.is_empty());
    }

    #[test]
    fn given_single_organism_when_pair_organisms_then_pairs_with_itself() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let organism_id = organisms[0].id();

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.id(), organism_id);
        assert_eq!(pairs[0].1.id(), organism_id); // Same organism paired with itself
    }

    #[test]
    fn given_two_organisms_when_pair_organisms_then_pairs_first_with_last() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let id1 = organisms[0].id(); // Best performer (lowest score)
        let id2 = organisms[1].id(); // Worst performer

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.id(), id1); // First (best)
        assert_eq!(pairs[0].1.id(), id2); // Last (worst)
    }

    #[test]
    fn given_three_organisms_when_pair_organisms_then_duplicates_top_performer() {
        let organisms = vec![
            make_org(1.0, 5, 0), // Best performer
            make_org(2.0, 3, 1), // Middle
            make_org(3.0, 2, 2), // Worst performer
        ];
        let id1 = organisms[0].id(); // Best performer
        let id2 = organisms[1].id(); // Middle
        let id3 = organisms[2].id(); // Worst performer

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        // With 3 organisms, top performer gets duplicated creating 4 total:
        // [best, best, middle, worst] -> pairs: (best, worst), (best, middle)
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0.id(), id1); // First copy of best performer
        assert_eq!(pairs[0].1.id(), id3); // Worst performer
        assert_eq!(pairs[1].0.id(), id1); // Second copy of best performer  
        assert_eq!(pairs[1].1.id(), id2); // Middle performer
    }

    #[test]
    fn given_four_organisms_when_pair_organisms_then_pairs_extremes() {
        let organisms = vec![
            make_org(1.0, 5, 0), // Best performer
            make_org(2.0, 3, 1), // Second best
            make_org(3.0, 2, 2), // Second worst
            make_org(4.0, 1, 3), // Worst performer
        ];
        let id1 = organisms[0].id(); // Best
        let id2 = organisms[1].id(); // Second best
        let id3 = organisms[2].id(); // Second worst
        let id4 = organisms[3].id(); // Worst

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        // Even count: extreme pairing (first with last, second with second-to-last)
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0.id(), id1); // Best performer
        assert_eq!(pairs[0].1.id(), id4); // Worst performer
        assert_eq!(pairs[1].0.id(), id2); // Second best
        assert_eq!(pairs[1].1.id(), id3); // Second worst
    }

    #[test]
    fn given_five_organisms_when_pair_organisms_then_duplicates_top_and_pairs_extremes() {
        let organisms = vec![
            make_org(1.0, 5, 0), // Best
            make_org(2.0, 4, 1), // 2nd best
            make_org(3.0, 3, 2), // Middle
            make_org(4.0, 2, 3), // 2nd worst
            make_org(5.0, 1, 4), // Worst
        ];
        let id1 = organisms[0].id(); // Best
        let id2 = organisms[1].id(); // 2nd best
        let id3 = organisms[2].id(); // Middle
        let id4 = organisms[3].id(); // 2nd worst
        let id5 = organisms[4].id(); // Worst

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        // With 5 organisms, duplicated list becomes: [best, best, 2nd, middle, 2nd_worst, worst]
        // Extreme pairing: (best, worst), (best, 2nd_worst), (2nd, middle)
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].0.id(), id1); // First copy of best
        assert_eq!(pairs[0].1.id(), id5); // Worst
        assert_eq!(pairs[1].0.id(), id1); // Second copy of best
        assert_eq!(pairs[1].1.id(), id4); // 2nd worst
        assert_eq!(pairs[2].0.id(), id2); // 2nd best
        assert_eq!(pairs[2].1.id(), id3); // Middle
    }

    #[test]
    fn given_six_organisms_when_pair_organisms_then_pairs_all_extremes() {
        let organisms = vec![
            make_org(1.0, 6, 0), // Best
            make_org(2.0, 5, 1), // 2nd best
            make_org(3.0, 4, 2), // 3rd best
            make_org(4.0, 3, 3), // 3rd worst
            make_org(5.0, 2, 4), // 2nd worst
            make_org(6.0, 1, 5), // Worst
        ];
        let id1 = organisms[0].id();
        let id2 = organisms[1].id();
        let id3 = organisms[2].id();
        let id4 = organisms[3].id();
        let id5 = organisms[4].id();
        let id6 = organisms[5].id();

        let pairs = Region::pair_organisms_for_reproduction(&organisms);

        // Even count: (best, worst), (2nd_best, 2nd_worst), (3rd_best, 3rd_worst)
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].0.id(), id1); // Best
        assert_eq!(pairs[0].1.id(), id6); // Worst
        assert_eq!(pairs[1].0.id(), id2); // 2nd best
        assert_eq!(pairs[1].1.id(), id5); // 2nd worst
        assert_eq!(pairs[2].0.id(), id3); // 3rd best
        assert_eq!(pairs[2].1.id(), id4); // 3rd worst
    }
}
