use rand::Rng;
use std::rc::Rc;

use crate::{
    phenotype::Phenotype,
    world::{organisms::organism::Organism, regions::region::Region},
};

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
    fn pair_organisms_for_reproduction(
        selected_organisms: &[Rc<Organism>],
    ) -> (Vec<(Rc<Organism>, Rc<Organism>)>, Option<Rc<Organism>>) {
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

    /// Performs sexual reproduction for all provided organism pairs.
    ///
    /// Each pair produces two offspring through sexual reproduction.
    ///
    /// * `organism_pairs` - Vector of organism pairs for sexual reproduction
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns a vector of offspring organisms produced from sexual reproduction
    fn perform_sexual_reproduction<R: Rng>(
        organism_pairs: &[(Rc<Organism>, Rc<Organism>)],
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

    /// Performs asexual reproduction for a single organism.
    ///
    /// * `organism` - The organism to reproduce asexually
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns the offspring organism produced from asexual reproduction
    fn perform_asexual_reproduction<R: Rng>(organism: &Rc<Organism>, rng: &mut R) -> Organism {
        let child_pheno = organism.phenotype().asexual_reproduction(rng);
        Organism::new(
            Rc::new(child_pheno),
            0,
            (Some(organism.id()), None), // Asexual: one parent
        )
    }

    /// Executes a single reproduction pass to generate offspring.
    ///
    /// This function handles the pairing logic and coordinates asexual and sexual reproduction
    /// for one pass, respecting the maximum offspring limit for the pass.
    ///
    /// * `selected_organisms` - The slice of organisms selected for reproduction
    /// * `max_offspring_this_pass` - Maximum number of offspring to produce in this pass
    /// * `rng` - Random number generator for reproduction operations
    ///
    /// Returns a vector of offspring produced in this reproduction pass
    fn execute_single_reproduction_pass<R: Rng>(
        selected_organisms: &[Rc<Organism>],
        max_offspring_this_pass: usize,
        rng: &mut R,
    ) -> Vec<Organism> {
        let mut offspring = Vec::new();

        // Pair organisms for reproduction
        let (organism_pairs, single_organism) =
            Self::pair_organisms_for_reproduction(selected_organisms);

        // Handle asexual reproduction if there's a single organism
        if let Some(single_org) = single_organism {
            if offspring.len() < max_offspring_this_pass {
                let asexual_child = Self::perform_asexual_reproduction(&single_org, rng);
                offspring.push(asexual_child);
            }
        }

        // Handle sexual reproduction for pairs
        if offspring.len() < max_offspring_this_pass {
            let sexual_offspring = Self::perform_sexual_reproduction(&organism_pairs, rng);
            offspring.extend(sexual_offspring);
        }

        // Limit offspring to what was requested for this pass
        offspring.truncate(max_offspring_this_pass);
        offspring
    }

    /// Executes multiple reproduction passes to generate offspring.
    ///
    /// This helper function handles the core reproduction logic, including asexual and sexual
    /// reproduction across multiple passes when needed for population growth.
    ///
    /// * `original_organisms` - The ranked parent organisms to use for reproduction
    /// * `parents_required` - Number of parents available for reproduction
    /// * `max_offspring_per_pass` - Maximum offspring that can be produced in a single pass
    /// * `number_to_reproduce` - Total number of offspring requested
    /// * `max_passes` - Maximum number of reproduction passes allowed
    /// * `rng` - Random number generator for reproduction operations
    pub(super) fn execute_reproduction_passes<R: Rng>(
        original_organisms: &[Rc<Organism>],
        parents_required: usize,
        max_offspring_per_pass: usize,
        number_to_reproduce: usize,
        max_passes: usize,
        rng: &mut R,
    ) -> Vec<Organism> {
        let mut all_offspring: Vec<Organism> = Vec::new();
        let mut remaining_to_reproduce = number_to_reproduce;

        // Reproduction passes
        for _pass in 0..max_passes {
            if remaining_to_reproduce == 0 {
                break;
            }

            let offspring_this_pass = remaining_to_reproduce.min(max_offspring_per_pass);
            let selected_slice = &original_organisms[..parents_required];

            // Execute single reproduction pass
            let offspring =
                Self::execute_single_reproduction_pass(selected_slice, offspring_this_pass, rng);

            remaining_to_reproduce -= offspring.len();
            all_offspring.extend(offspring);
        }

        all_offspring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{phenotype::Phenotype, world::organisms::organism::Organism};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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
    fn given_single_organism_single_pass_when_execute_then_one_offspring() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 1, // parents_required
            1, // max_offspring_per_pass
            1, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 1);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_two_organisms_single_pass_when_execute_then_two_offspring() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 2, // parents_required
            2, // max_offspring_per_pass
            2, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 2);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_single_organism_multiple_passes_when_execute_then_multiple_offspring() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 1, // parents_required
            1, // max_offspring_per_pass
            3, // number_to_reproduce
            3, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 3);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_two_organisms_multiple_passes_when_execute_then_multiple_offspring() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 2, // parents_required
            2, // max_offspring_per_pass
            6, // number_to_reproduce
            3, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 6);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_three_organisms_odd_number_when_execute_then_handles_asexual_correctly() {
        let organisms = vec![
            make_org(1.0, 5, 0),
            make_org(2.0, 3, 1),
            make_org(3.0, 2, 2),
        ];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 3, // parents_required
            3, // max_offspring_per_pass (3 organisms: 1 asexual + 2 sexual = 3)
            9, // number_to_reproduce
            3, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 9);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_zero_number_to_reproduce_when_execute_then_empty_result() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 1, // parents_required
            1, // max_offspring_per_pass
            0, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        assert!(offspring.is_empty());
    }

    #[test]
    fn given_max_passes_limit_when_execute_then_stops_at_limit() {
        let organisms = vec![make_org(1.0, 5, 0)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 1,  // parents_required
            1,  // max_offspring_per_pass
            10, // number_to_reproduce (way more than passes allow)
            3,  // max_passes
            &mut rng,
        );

        // Should only get 3 offspring due to max_passes limit
        assert_eq!(offspring.len(), 3);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_offspring_this_pass_limit_when_execute_then_respects_limit() {
        let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 2, // parents_required
            1, // max_offspring_per_pass (artificially low)
            3, // number_to_reproduce
            3, // max_passes
            &mut rng,
        );

        // Should get 3 offspring over 3 passes (1 per pass)
        assert_eq!(offspring.len(), 3);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_single_organism_when_execute_then_offspring_has_correct_asexual_parent_id() {
        let parent = make_org(1.0, 5, 0);
        let parent_id = parent.id();
        let organisms = vec![parent];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 1, // parents_required
            1, // max_offspring_per_pass
            1, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 1);
        let child = &offspring[0];
        assert_eq!(child.parent_count(), 1);
        assert_eq!(child.parent_ids(), (Some(parent_id), None));
        assert!(!child.is_root());
    }

    #[test]
    fn given_two_organisms_when_execute_then_offspring_have_correct_sexual_parent_ids() {
        let parent1 = make_org(1.0, 5, 0);
        let parent2 = make_org(2.0, 3, 1);
        let parent1_id = parent1.id();
        let parent2_id = parent2.id();
        let organisms = vec![parent1, parent2];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 2, // parents_required
            2, // max_offspring_per_pass
            2, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 2);

        // Both offspring should have both parents
        for child in &offspring {
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(parent1_id), Some(parent2_id)));
            assert!(!child.is_root());
        }
    }

    #[test]
    fn given_three_organisms_when_execute_then_mixed_reproduction_has_correct_parent_ids() {
        let parent1 = make_org(1.0, 5, 0);
        let parent2 = make_org(2.0, 3, 1);
        let parent3 = make_org(3.0, 2, 2);
        let parent1_id = parent1.id();
        let parent2_id = parent2.id();
        let parent3_id = parent3.id();
        let organisms = vec![parent1, parent2, parent3];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 3, // parents_required
            3, // max_offspring_per_pass (3 organisms: 1 asexual + 2 sexual = 3)
            3, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        assert_eq!(offspring.len(), 3);

        // First offspring should be from asexual reproduction (parent1)
        let asexual_child = &offspring[0];
        assert_eq!(asexual_child.parent_count(), 1);
        assert_eq!(asexual_child.parent_ids(), (Some(parent1_id), None));

        // Remaining offspring should be from sexual reproduction (parent2 & parent3)
        for child in &offspring[1..] {
            assert_eq!(child.parent_count(), 2);
            assert_eq!(child.parent_ids(), (Some(parent2_id), Some(parent3_id)));
        }
    }

    // Tests for helper functions
    mod helper_function_tests {
        use super::*;

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

        #[test]
        fn given_empty_pairs_when_perform_sexual_reproduction_then_returns_empty_offspring() {
            let pairs: Vec<(Rc<Organism>, Rc<Organism>)> = vec![];
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

        #[test]
        fn given_single_organism_when_execute_single_pass_then_returns_one_asexual_offspring() {
            let organisms = vec![make_org(1.0, 5, 0)];
            let parent_id = organisms[0].id();
            let mut rng = SmallRng::seed_from_u64(0);

            let offspring = Region::execute_single_reproduction_pass(&organisms, 5, &mut rng);

            assert_eq!(offspring.len(), 1);
            assert_eq!(offspring[0].age(), 0);
            assert_eq!(offspring[0].parent_count(), 1);
            assert_eq!(offspring[0].parent_ids(), (Some(parent_id), None));
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
        fn given_three_organisms_when_execute_single_pass_then_returns_mixed_reproduction() {
            let organisms = vec![
                make_org(1.0, 5, 0),
                make_org(2.0, 3, 1),
                make_org(3.0, 2, 2),
            ];
            let id1 = organisms[0].id();
            let id2 = organisms[1].id();
            let id3 = organisms[2].id();
            let mut rng = SmallRng::seed_from_u64(0);

            let offspring = Region::execute_single_reproduction_pass(&organisms, 5, &mut rng);

            assert_eq!(offspring.len(), 3);

            // First offspring should be asexual (from first organism)
            assert_eq!(offspring[0].parent_count(), 1);
            assert_eq!(offspring[0].parent_ids(), (Some(id1), None));

            // Remaining offspring should be sexual (from second and third organisms)
            for child in &offspring[1..] {
                assert_eq!(child.parent_count(), 2);
                assert_eq!(child.parent_ids(), (Some(id2), Some(id3)));
            }
        }

        #[test]
        fn given_max_offspring_limit_when_execute_single_pass_then_respects_limit() {
            let organisms = vec![make_org(1.0, 5, 0), make_org(2.0, 3, 1)];
            let mut rng = SmallRng::seed_from_u64(0);

            let offspring = Region::execute_single_reproduction_pass(&organisms, 1, &mut rng);

            assert_eq!(offspring.len(), 1); // Limited to 1 despite 2 organisms could produce 2
        }
    }
}
