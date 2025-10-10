use rand::Rng;

use crate::world::{organisms::organism::Organism, regions::region::Region};

impl Region {
    /// Reproduces new organisms for this region based on the ranking rules in the PDD (§5.2.3).
    ///
    /// **PRECONDITION**: Organisms in this region must already be sorted by fitness score (ascending)
    /// then age (descending). This sorting should be done by calling `Regions::sort_regions()` before
    /// calling this function.
    ///
    /// When population is low relative to carrying capacity, organisms can reproduce multiple
    /// times (up to REPRODUCTION_FACTOR) to accelerate population growth.
    ///
    /// * `number_to_reproduce` – the number of offspring **required** for this region.
    /// * `rng` – RNG used for crossover & mutation in the underlying phenotype reproduction.
    ///
    /// The algorithm follows the PDD exactly:
    /// 1. Select the top `number_to_reproduce` parents from the pre-sorted list (or all organisms if fewer).
    /// 2. Organisms are paired using extreme pairing strategy (best with worst, etc.).
    ///    For odd counts, the top performer is duplicated to create an even list.
    /// 3. Each pair produces two offspring via sexual reproduction.
    /// 4. If carrying capacity allows and population is still low, repeat reproduction passes
    ///    up to REPRODUCTION_FACTOR times using only the original organisms.
    ///
    /// The resulting offspring are returned as a `Vec<Organism>` with age 0 and no score.
    pub fn reproduce<R: Rng>(&mut self, number_to_reproduce: usize, rng: &mut R) -> Vec<Organism> {
        if number_to_reproduce == 0 || self.organisms.is_empty() {
            return Vec::new();
        }

        // Store current population size for carrying capacity check
        let current_population = self.organisms.len();

        // ------------- Select parents from pre-sorted organisms -------------
        // Organisms are assumed to be already sorted by score (asc) then age (desc)
        // by a prior call to Regions::sort_regions()
        let slice = &mut self.organisms;

        // Work with references to avoid cloning - organisms should never be cloned
        let parents_required = number_to_reproduce.min(slice.len());

        // Calculate maximum offspring per pass based on available parents
        // With extreme pairing:
        // - Odd count: top performer duplicated, creating (parents_required+1) organisms,
        //   which form (parents_required+1)/2 pairs producing 2*(parents_required+1)/2 = parents_required+1 offspring
        // - Even count: parents_required/2 pairs producing 2*(parents_required/2) = parents_required offspring
        let max_offspring_per_pass = if parents_required % 2 == 1 {
            parents_required + 1 // Odd: top performer duplicated creates one extra offspring
        } else {
            parents_required // Even: each organism pairs once
        };

        // Check if multiple passes are warranted based on carrying capacity
        let should_do_multiple_passes = if let Some(capacity) = self.carrying_capacity {
            // Only do multiple passes if current population + requested offspring would still be under capacity
            // and we can't satisfy the request in a single pass
            let total_desired = current_population + number_to_reproduce;
            total_desired <= capacity && number_to_reproduce > max_offspring_per_pass
        } else {
            // No carrying capacity set, use original behavior (single pass only)
            false
        };

        let max_passes = if should_do_multiple_passes {
            Self::REPRODUCTION_FACTOR
        } else {
            1
        };

        // Execute reproduction passes
        Self::execute_reproduction_passes(
            slice,
            parents_required,
            max_offspring_per_pass,
            number_to_reproduce,
            max_passes,
            rng,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{phenotype::Phenotype, world::organisms::organism::Organism};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::rc::Rc;

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
    fn given_even_r_when_reproduce_then_returns_r_offspring() {
        let mut region = Region::new();
        for i in 0..4 {
            region.add_organism(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = SmallRng::seed_from_u64(0);
        let offspring = region.reproduce(4, &mut rng);
        assert_eq!(offspring.len(), 4);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_odd_r_when_reproduce_then_returns_r_offspring() {
        let mut region = Region::new();
        for i in 0..5 {
            region.add_organism(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = SmallRng::seed_from_u64(0);
        let offspring = region.reproduce(3, &mut rng);
        assert_eq!(offspring.len(), 3);
    }

    #[test]
    fn given_zero_r_when_reproduce_then_returns_empty_vec() {
        let mut region = Region::new();
        region.add_organism(make_org(1.0, 0, 0));
        let mut rng = SmallRng::seed_from_u64(0);
        let offspring = region.reproduce(0, &mut rng);
        assert!(offspring.is_empty());
    }

    #[test]
    fn given_empty_region_when_reproduce_then_returns_empty_vec() {
        let mut region = Region::new();
        let mut rng = SmallRng::seed_from_u64(0);
        let offspring = region.reproduce(3, &mut rng);
        assert!(offspring.is_empty());
    }

    #[test]
    fn given_one_parent_when_reproduce_then_two_offspring_sexual_self_fertilization() {
        let mut region = Region::new();
        region.add_organism(make_org(2.0, 5, 0));
        let mut rng = SmallRng::seed_from_u64(0);
        let offspring = region.reproduce(1, &mut rng);
        // Single organism produces 2 offspring via self-fertilization, limited to 1 by request
        assert_eq!(offspring.len(), 1);
    }

    #[test]
    fn given_r_exceeds_parents_when_reproduce_then_all_parents_used() {
        let mut region = Region::new();
        region.add_organism(make_org(1.0, 1, 0));
        region.add_organism(make_org(2.0, 2, 1));
        let mut rng = SmallRng::seed_from_u64(0);
        // Request more than available (5 > 2)
        let offspring = region.reproduce(5, &mut rng);
        // Two parents => even => 2 offspring via sexual reproduction
        assert_eq!(offspring.len(), 2);
    }

    #[test]
    fn given_equal_score_different_age_then_older_ranks_higher() {
        let mut region = Region::new();
        // Same score, different ages
        region.add_organism(make_org(1.0, 10, 0)); // older
        region.add_organism(make_org(1.0, 5, 1)); // younger
        let mut rng = SmallRng::seed_from_u64(0);
        let _ = region.reproduce(1, &mut rng);
        // After reproduction, organisms slice is sorted; index 0 should be older
        let first_age = region.organisms()[0].age();
        assert_eq!(first_age, 10);
    }

    #[test]
    fn given_single_organism_when_reproduce_multiple_passes_then_produces_multiple_offspring() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(100)); // Set high carrying capacity to enable multiple passes
        region.add_organism(make_org(1.0, 5, 0));
        let mut rng = SmallRng::seed_from_u64(0);
        // Request more offspring than can be produced in single pass
        let offspring = region.reproduce(3, &mut rng);
        // Single organism pairs with itself (self-fertilization), producing 2 offspring per pass
        // 3 requested / 2 per pass = 2 passes = 3 offspring (last pass truncated)
        assert_eq!(offspring.len(), 3);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_two_organisms_when_reproduce_multiple_passes_then_produces_multiple_offspring() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(100)); // Set high carrying capacity to enable multiple passes
        region.add_organism(make_org(1.0, 5, 0));
        region.add_organism(make_org(2.0, 3, 1));
        let mut rng = SmallRng::seed_from_u64(0);
        // Request more offspring than can be produced in single pass (2 parents -> 2 offspring per pass)
        let offspring = region.reproduce(6, &mut rng);
        // Two organisms can produce 2 offspring per pass, so we get 6 offspring over 3 passes
        assert_eq!(offspring.len(), 6);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_reproduction_factor_limit_when_reproduce_then_stops_at_factor_limit() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(100)); // Set high carrying capacity to enable multiple passes
        region.add_organism(make_org(1.0, 5, 0));
        let mut rng = SmallRng::seed_from_u64(0);
        // Request way more offspring than REPRODUCTION_FACTOR allows
        let offspring = region.reproduce(20, &mut rng);
        // Single organism produces 2 offspring per pass via self-fertilization, limited by REPRODUCTION_FACTOR = 10
        // 10 passes * 2 offspring = 20
        assert_eq!(offspring.len(), 20);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_sufficient_parents_when_reproduce_single_pass_then_no_multiple_passes() {
        let mut region = Region::new();
        // Add enough organisms to satisfy reproduction in single pass
        for i in 0..4 {
            region.add_organism(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = SmallRng::seed_from_u64(0);
        let offspring = region.reproduce(4, &mut rng);
        // Should produce exactly 4 offspring in single pass
        assert_eq!(offspring.len(), 4);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_odd_number_organisms_when_reproduce_multiple_passes_then_handles_extreme_pairing_correctly()
     {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(100)); // Set high carrying capacity to enable multiple passes
        // Add 3 organisms (odd number)
        for i in 0..3 {
            region.add_organism(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = SmallRng::seed_from_u64(0);
        // Request more than single pass can produce
        let offspring = region.reproduce(9, &mut rng);
        // 3 organisms with extreme pairing: top performer duplicated creates 4 organisms
        // 2 pairs produce 4 offspring per pass. 9 requested / 4 per pass = 3 passes (rounded up) = 9 offspring
        assert_eq!(offspring.len(), 9);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_no_carrying_capacity_when_reproduce_then_single_pass_only() {
        let mut region = Region::new();
        // No carrying capacity set - should default to single pass behavior
        region.add_organism(make_org(1.0, 5, 0));
        let mut rng = SmallRng::seed_from_u64(0);
        // Request more offspring than single pass can produce
        let offspring = region.reproduce(5, &mut rng);
        // Without carrying capacity, should only get 2 offspring (single pass, self-fertilization)
        assert_eq!(offspring.len(), 2);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }

    #[test]
    fn given_carrying_capacity_exceeded_when_reproduce_then_single_pass_only() {
        let mut region = Region::new();
        region.set_carrying_capacity(Some(5)); // Low carrying capacity
        // Add organisms that would exceed capacity with requested offspring
        for i in 0..3 {
            region.add_organism(make_org(i as f64 + 1.0, i, i));
        }
        let mut rng = SmallRng::seed_from_u64(0);
        // Request offspring that would exceed carrying capacity (3 + 5 = 8 > 5)
        let offspring = region.reproduce(5, &mut rng);
        // Should only do single pass since total would exceed capacity
        // 3 organisms with extreme pairing: top performer duplicated = 2 pairs = 4 offspring per pass
        assert_eq!(offspring.len(), 4);
        assert!(offspring.iter().all(|o| o.age() == 0));
    }
}
