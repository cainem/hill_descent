use rand::Rng;
use std::sync::Arc;

use crate::world::{organisms::organism::Organism, regions::region::Region};

impl Region {
    /// Executes a single reproduction pass to generate offspring.
    ///
    /// This function uses extreme pairing strategy where organisms are paired first-with-last,
    /// second-with-second-to-last, etc. For odd counts, the top performer is duplicated.
    /// All reproduction is now sexual (no asexual reproduction).
    ///
    /// * `selected_organisms` - The slice of organisms selected for reproduction
    /// * `max_offspring_this_pass` - Maximum number of offspring to produce in this pass
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns a vector of offspring produced in this reproduction pass
    pub(super) fn execute_single_reproduction_pass<R: Rng>(
        selected_organisms: &[Arc<Organism>],
        max_offspring_this_pass: usize,
        rng: &mut R,
    ) -> Vec<Organism> {
        let mut offspring = Vec::new();

        // Pair organisms for reproduction using extreme pairing strategy
        let organism_pairs = Self::pair_organisms_for_reproduction(selected_organisms);

        // Perform sexual reproduction for all pairs (no more asexual reproduction)
        let sexual_offspring = Self::perform_sexual_reproduction(&organism_pairs, rng);
        offspring.extend(sexual_offspring);

        // Limit offspring to what was requested for this pass
        offspring.truncate(max_offspring_this_pass);
        offspring
    }

    /// Executes a single reproduction pass to generate offspring, returning an iterator.
    ///
    /// This function uses extreme pairing strategy where organisms are paired first-with-last,
    /// second-with-second-to-last, etc. For odd counts, the top performer is duplicated.
    /// All reproduction is now sexual (no asexual reproduction).
    /// This iterator version returns offspring as an owned iterator (via Vec).
    ///
    /// Note: Due to Rust lifetime constraints with mutable RNG in closures, we collect offspring
    /// into a Vec and return it as an iterator. The real allocation savings come from upper layers
    /// (execute_reproduction_passes and parallel_process_regions) not allocating intermediate Vecs.
    ///
    /// * `selected_organisms` - The slice of organisms selected for reproduction
    /// * `max_offspring_this_pass` - Maximum number of offspring to produce in this pass
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns an iterator that yields offspring organisms
    pub(super) fn execute_single_reproduction_pass_iter<R: Rng>(
        selected_organisms: &[Arc<Organism>],
        max_offspring_this_pass: usize,
        rng: &mut R,
    ) -> impl Iterator<Item = Organism> {
        // Collect pairs (small allocation - typically 5-10 pairs per region)
        let pairs: Vec<_> = Self::pair_organisms_for_reproduction_iter(selected_organisms)
            .map(|(p1, p2)| (Arc::clone(p1), Arc::clone(p2)))
            .collect();

        // Produce offspring from pairs
        // We need to collect here due to mutable RNG lifetime constraints
        let offspring: Vec<_> = pairs
            .into_iter()
            .flat_map(|(parent1, parent2)| {
                Self::perform_sexual_reproduction_iter(&parent1, &parent2, rng).collect::<Vec<_>>()
            })
            .take(max_offspring_this_pass)
            .collect();

        // Return as iterator for compatibility with upper layers
        offspring.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use rand::{SeedableRng, rngs::SmallRng};
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
    fn given_single_organism_when_execute_single_pass_then_returns_two_sexual_offspring() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let parent_id = organisms[0].id();
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_single_reproduction_pass(&organisms, 5, &mut rng);

        // Single organism pairs with itself, producing 2 offspring
        assert_eq!(offspring.len(), 2);
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(parent_id), Some(parent_id))); // Self-fertilization
        }
    }

    #[test]
    fn given_two_organisms_when_execute_single_pass_then_returns_two_sexual_offspring() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let id1 = organisms[0].id();
        let id2 = organisms[1].id();
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_single_reproduction_pass(&organisms, 5, &mut rng);

        assert_eq!(offspring.len(), 2);
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(id1), Some(id2)));
        }
    }

    #[test]
    fn given_three_organisms_when_execute_single_pass_then_returns_sexual_reproduction_with_duplication()
     {
        let organisms = vec![
            make_org(1.0, 5, 0), // Best performer (will be duplicated)
            make_org(2.0, 3, 1), // Middle
            make_org(3.0, 2, 2), // Worst performer
        ];
        let id1 = organisms[0].id(); // Best
        let id2 = organisms[1].id(); // Middle
        let id3 = organisms[2].id(); // Worst
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_single_reproduction_pass(&organisms, 5, &mut rng);

        // With 3 organisms, top performer duplicated: [best, best, middle, worst]
        // Pairs: (best, worst), (best, middle) = 4 offspring total
        assert_eq!(offspring.len(), 4);

        // All offspring should be sexual reproduction (2 parents each)
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);

            // Each offspring should have best performer as one parent
            let (p1, p2) = child.parent_ids();
            assert!(p1 == Some(id1) || p2 == Some(id1)); // Best performer is always a parent

            // The other parent should be either middle or worst performer
            assert!(
                (p1 == Some(id1) && (p2 == Some(id2) || p2 == Some(id3)))
                    || (p2 == Some(id1) && (p1 == Some(id2) || p1 == Some(id3)))
            );
        }
    }

    #[test]
    fn given_max_offspring_limit_when_execute_single_pass_then_respects_limit() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_single_reproduction_pass(&organisms, 1, &mut rng);

        assert_eq!(offspring.len(), 1); // Limited to 1 despite 2 organisms could produce 2
    }

    // Tests for iterator version
    #[test]
    fn given_single_organism_when_execute_single_pass_iter_then_returns_two_sexual_offspring() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let parent_id = organisms[0].id();
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring: Vec<_> = Region::execute_single_reproduction_pass_iter(&organisms, 5, &mut rng).collect();

        // Single organism pairs with itself, producing 2 offspring
        assert_eq!(offspring.len(), 2);
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(parent_id), Some(parent_id))); // Self-fertilization
        }
    }

    #[test]
    fn given_two_organisms_when_execute_single_pass_iter_then_returns_two_sexual_offspring() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let id1 = organisms[0].id();
        let id2 = organisms[1].id();
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring: Vec<_> = Region::execute_single_reproduction_pass_iter(&organisms, 5, &mut rng).collect();

        assert_eq!(offspring.len(), 2);
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(id1), Some(id2)));
        }
    }

    #[test]
    fn given_three_organisms_when_execute_single_pass_iter_then_returns_sexual_reproduction_with_duplication()
     {
        let organisms = vec![
            make_org(1.0, 5, 0), // Best performer (will be duplicated)
            make_org(2.0, 3, 1), // Middle
            make_org(3.0, 2, 2), // Worst performer
        ];
        let id1 = organisms[0].id(); // Best
        let id2 = organisms[1].id(); // Middle
        let id3 = organisms[2].id(); // Worst
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring: Vec<_> = Region::execute_single_reproduction_pass_iter(&organisms, 5, &mut rng).collect();

        // With 3 organisms, top performer duplicated: [best, best, middle, worst]
        // Pairs: (best, worst), (best, middle) = 4 offspring total
        assert_eq!(offspring.len(), 4);

        // All offspring should be sexual reproduction (2 parents each)
        for child in &offspring {
            assert_eq!(child.age(), 0);
            assert_eq!(child.parent_count(), 2);

            // Each offspring should have best performer as one parent
            let (p1, p2) = child.parent_ids();
            assert!(p1 == Some(id1) || p2 == Some(id1)); // Best performer is always a parent

            // The other parent should be either middle or worst performer
            assert!(
                (p1 == Some(id1) && (p2 == Some(id2) || p2 == Some(id3)))
                    || (p2 == Some(id1) && (p1 == Some(id2) || p1 == Some(id3)))
            );
        }
    }

    #[test]
    fn given_max_offspring_limit_when_execute_single_pass_iter_then_respects_limit() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring: Vec<_> = Region::execute_single_reproduction_pass_iter(&organisms, 1, &mut rng).collect();

        assert_eq!(offspring.len(), 1); // Limited to 1 despite 2 organisms could produce 2
    }
}
