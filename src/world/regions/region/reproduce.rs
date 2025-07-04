use rand::Rng;
use std::rc::Rc;

use crate::{
    phenotype::Phenotype,
    world::{organisms::organism::Organism, regions::region::Region},
};

impl Region {
    /// Reproduces new organisms for this region based on the ranking rules in the PDD (§5.2.3).
    ///
    /// * `number_to_reproduce` – the number of offspring **required** for this region.
    /// * `rng` – RNG used for crossover & mutation in the underlying phenotype reproduction.
    ///
    /// The algorithm follows the PDD exactly:
    /// 1. Rank organisms by (a) fitness score ascending (lower is better),
    ///    (b) age descending (older first). Any further tie is arbitrary.
    /// 2. Select the top `number_to_reproduce` parents (or all organisms if fewer).
    /// 3. If the selected count is odd, the top-ranked organism reproduces asexually to yield one
    ///    offspring, then the remainder (now even) are paired sequentially for sexual reproduction.
    /// 4. Each pair produces two offspring.
    ///
    /// The resulting offspring are returned as a `Vec<Organism>` with age 0 and no score.
    pub fn reproduce<R: Rng>(&mut self, number_to_reproduce: usize, rng: &mut R) -> Vec<Organism> {
        if number_to_reproduce == 0 || self.organisms.is_empty() {
            return Vec::new();
        }

        // ------------- 1 & 2. Rank and select parents -----------------------
        // Sort organisms in-place by score (asc) then age (desc). No further
        // ordering is required once these two keys are equal.
        let slice = &mut self.organisms;
        slice.sort_by(|a, b| {
            let score_cmp = a
                .score()
                .unwrap_or(f64::INFINITY)
                .partial_cmp(&b.score().unwrap_or(f64::INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal);
            score_cmp.then_with(|| b.age().cmp(&a.age()))
        });

        let parents_required = number_to_reproduce.min(slice.len());
        let mut selected: Vec<Rc<Organism>> = slice[..parents_required].to_vec();

        // ------------- 3 & 4. Produce offspring -----------------------------
        let mut offspring: Vec<Organism> = Vec::new();

        // If odd, #0 reproduces asexually first.
        if selected.len() % 2 == 1 {
            let parent = &selected[0];
            let child_pheno = parent.phenotype().asexual_reproduction(rng);
            offspring.push(Organism::new(Rc::new(child_pheno), 0));
            // Remove the first so that the remainder is even
            selected.remove(0);
        }

        // Pair sequentially for sexual reproduction.
        for chunk in selected.chunks(2) {
            if let [p1, p2] = chunk {
                let (c1, c2) = Phenotype::sexual_reproduction(p1.phenotype(), p2.phenotype(), rng);
                offspring.push(Organism::new(Rc::new(c1), 0));
                offspring.push(Organism::new(Rc::new(c2), 0));
            }
        }

        offspring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{phenotype::Phenotype, world::organisms::organism::Organism};
    use rand::rngs::mock::StepRng;

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
    fn given_even_r_when_reproduce_then_returns_r_offspring() {
        let mut region = Region::new();
        for i in 0..4 {
            region.add_phenotype(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = StepRng::new(0, 1);
        let offspring = region.reproduce(4, &mut rng);
        assert_eq!(offspring.len(), 4);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_odd_r_when_reproduce_then_returns_r_offspring() {
        let mut region = Region::new();
        for i in 0..5 {
            region.add_phenotype(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = StepRng::new(0, 1);
        let offspring = region.reproduce(3, &mut rng);
        assert_eq!(offspring.len(), 3);
    }

    #[test]
    fn given_zero_r_when_reproduce_then_returns_empty_vec() {
        let mut region = Region::new();
        region.add_phenotype(make_org(1.0, 0, 0));
        let mut rng = StepRng::new(0, 1);
        let offspring = region.reproduce(0, &mut rng);
        assert!(offspring.is_empty());
    }

    #[test]
    fn given_empty_region_when_reproduce_then_returns_empty_vec() {
        let mut region = Region::new();
        let mut rng = StepRng::new(0, 1);
        let offspring = region.reproduce(3, &mut rng);
        assert!(offspring.is_empty());
    }

    #[test]
    fn given_one_parent_when_reproduce_then_one_offspring_asexual() {
        let mut region = Region::new();
        region.add_phenotype(make_org(2.0, 5, 0));
        let mut rng = StepRng::new(0, 1);
        let offspring = region.reproduce(1, &mut rng);
        assert_eq!(offspring.len(), 1);
    }

    #[test]
    fn given_r_exceeds_parents_when_reproduce_then_all_parents_used() {
        let mut region = Region::new();
        region.add_phenotype(make_org(1.0, 1, 0));
        region.add_phenotype(make_org(2.0, 2, 1));
        let mut rng = StepRng::new(0, 1);
        // Request more than available (5 > 2)
        let offspring = region.reproduce(5, &mut rng);
        // Two parents => even => 2 offspring via sexual reproduction
        assert_eq!(offspring.len(), 2);
    }

    #[test]
    fn given_equal_score_different_age_then_older_ranks_higher() {
        let mut region = Region::new();
        // Same score, different ages
        region.add_phenotype(make_org(1.0, 10, 0)); // older
        region.add_phenotype(make_org(1.0, 5, 1)); // younger
        let mut rng = StepRng::new(0, 1);
        let _ = region.reproduce(1, &mut rng);
        // After reproduction, organisms slice is sorted; index 0 should be older
        let first_age = region.get_organisms()[0].age();
        assert_eq!(first_age, 10);
    }
}
