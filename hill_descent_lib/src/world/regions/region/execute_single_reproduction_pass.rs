use rand::Rng;
use std::rc::Rc;

use crate::world::{organisms::organism::Organism, regions::region::Region};

impl Region {
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
    pub(super) fn execute_single_reproduction_pass<R: Rng>(
        selected_organisms: &[Rc<Organism>],
        max_offspring_this_pass: usize,
        rng: &mut R,
    ) -> Vec<Organism> {
        let mut offspring = Vec::new();

        // Pair organisms for reproduction
        let (organism_pairs, single_organism) =
            Self::pair_organisms_for_reproduction(selected_organisms);

        // Handle asexual reproduction if there's a single organism
        if let Some(single_org) = single_organism
            && offspring.len() < max_offspring_this_pass
        {
            let asexual_child = Self::perform_asexual_reproduction(&single_org, rng);
            offspring.push(asexual_child);
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
