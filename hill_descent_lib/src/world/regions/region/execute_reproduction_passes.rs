use rand::Rng;
use std::rc::Rc;

use crate::{
    phenotype::Phenotype,
    world::{organisms::organism::Organism, regions::region::Region},
};

impl Region {
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
            let mut selected: Vec<Rc<Organism>> = original_organisms[..parents_required].to_vec();

            // ------------- 3 & 4. Produce offspring -----------------------------
            let mut offspring: Vec<Organism> = Vec::new();

            // If odd, #0 reproduces asexually first.
            if selected.len() % 2 == 1 && offspring.len() < offspring_this_pass {
                let parent = &selected[0];
                let child_pheno = parent.phenotype().asexual_reproduction(rng);
                offspring.push(Organism::new(Rc::new(child_pheno), 0));
                // Remove the first so that the remainder is even
                selected.remove(0);
            }

            // Pair sequentially for sexual reproduction.
            for chunk in selected.chunks(2) {
                if offspring.len() >= offspring_this_pass {
                    break;
                }
                if let [p1, p2] = chunk {
                    let (c1, c2) =
                        Phenotype::sexual_reproduction(p1.phenotype(), p2.phenotype(), rng);
                    offspring.push(Organism::new(Rc::new(c1), 0));
                    if offspring.len() < offspring_this_pass {
                        offspring.push(Organism::new(Rc::new(c2), 0));
                    }
                }
            }

            // Limit offspring to what was requested for this pass
            offspring.truncate(offspring_this_pass);
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
        let org = Organism::new(Rc::clone(&phenotype), age);
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
}
