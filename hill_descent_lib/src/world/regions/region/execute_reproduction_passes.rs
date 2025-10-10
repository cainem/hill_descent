use rand::Rng;
use std::rc::Rc;

use crate::world::{organisms::organism::Organism, regions::region::Region};

// The function `execute_single_reproduction_pass` from helper modules is imported in mod.rs
// to make it available on Region. This file depends on that function for reproduction logic.

impl Region {
    /// Executes multiple reproduction passes to generate offspring.
    ///
    /// This helper function handles the core reproduction logic using extreme pairing strategy
    /// across multiple passes when needed for population growth.
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
                Region::execute_single_reproduction_pass(selected_slice, offspring_this_pass, rng);

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
        let expressed = vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 0.0, 0.1, 100.0, 2.0, idx as f64,
        ];
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
    fn given_three_organisms_odd_number_when_execute_then_handles_extreme_pairing_correctly() {
        let organisms = vec![
            make_org(1.0, 5, 0),
            make_org(2.0, 3, 1),
            make_org(3.0, 2, 2),
        ];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 3, // parents_required
            4, // max_offspring_per_pass (3 organisms with duplication: 2 pairs = 4 offspring)
            9, // number_to_reproduce
            3, // max_passes
            &mut rng,
        );

        // With extreme pairing and duplication: 4 offspring per pass * 3 passes = 12, limited to 9
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
            2,  // max_offspring_per_pass (single organism produces 2 via self-fertilization)
            10, // number_to_reproduce (way more than passes allow)
            3,  // max_passes
            &mut rng,
        );

        // Should only get 6 offspring due to max_passes limit (3 passes * 2 offspring each)
        assert_eq!(offspring.len(), 6);
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
    fn given_single_organism_when_execute_then_offspring_has_correct_sexual_self_fertilization() {
        let parent = make_org(1.0, 5, 0);
        let parent_id = parent.id();
        let organisms = vec![parent];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 1, // parents_required
            2, // max_offspring_per_pass (single organism now produces 2 via self-fertilization)
            1, // number_to_reproduce
            1, // max_passes
            &mut rng,
        );

        // Single organism pairs with itself and produces 2 offspring, but limited to 1 by number_to_reproduce
        assert_eq!(offspring.len(), 1);
        let child = &offspring[0];
        assert_eq!(child.parent_count(), 2);
        assert_eq!(child.parent_ids(), (Some(parent_id), Some(parent_id))); // Self-fertilization
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
    fn given_three_organisms_when_execute_then_sexual_reproduction_with_top_performer_duplication()
    {
        let parent1 = make_org(1.0, 5, 0); // Best performer
        let parent2 = make_org(2.0, 3, 1); // Middle
        let parent3 = make_org(3.0, 2, 2); // Worst performer 
        let parent1_id = parent1.id(); // Best
        let parent2_id = parent2.id(); // Middle
        let parent3_id = parent3.id(); // Worst
        let organisms = vec![parent1, parent2, parent3];
        let mut rng = SmallRng::seed_from_u64(0);

        let offspring = Region::execute_reproduction_passes(
            &organisms, 3, // parents_required
            4, // max_offspring_per_pass (3 organisms become 4 with duplication: 2 pairs = 4 offspring)
            3, // number_to_reproduce (limit to 3)
            1, // max_passes
            &mut rng,
        );

        // With 3 organisms, top performer duplicated: [best, best, middle, worst]
        // Pairs: (best, worst), (best, middle) = 4 offspring, limited to 3
        assert_eq!(offspring.len(), 3);

        // All offspring should be sexual reproduction (2 parents each)
        for child in &offspring {
            assert_eq!(child.parent_count(), 2);

            // Each offspring should have best performer as one parent
            let (p1, p2) = child.parent_ids();
            assert!(p1 == Some(parent1_id) || p2 == Some(parent1_id)); // Best performer is always a parent

            // The other parent should be either middle or worst performer
            assert!(
                (p1 == Some(parent1_id) && (p2 == Some(parent2_id) || p2 == Some(parent3_id)))
                    || (p2 == Some(parent1_id)
                        && (p1 == Some(parent2_id) || p1 == Some(parent3_id)))
            );
        }
    }
}
