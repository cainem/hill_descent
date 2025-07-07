use crate::world::regions::Regions;

impl Regions {
    /// Updates the carrying capacity for all regions.
    ///
    /// Calculation is based on PDD section 4.2.4:
    /// P_i = P * (1/F_i) / sum_over_j(1/F_j)
    /// where P is total target population_size, F_i is min_score in region i.
    ///
    /// To prevent floating-point overflows when a `min_score` is extremely small,
    /// the inverse fitness is capped at a large but finite value if it would otherwise
    /// be infinite.
    ///
    /// Regions with no valid positive min_score, or if the sum of inverse fitnesses is not positive,
    /// will have their carrying capacity set to 0.
    pub(super) fn update_carrying_capacities(&mut self) {
        let mut sum_inverse_min_fitness = 0.0;

        // First pass to calculate the sum of inverse fitnesses.
        for (_, region) in self.regions.iter() {
            if let Some(min_score) = region.min_score() {
                if min_score > 0.0 {
                    let mut inverse_fitness = 1.0 / min_score;
                    if inverse_fitness.is_infinite() {
                        // If inverse fitness is infinite (due to a very small min_score),
                        // cap it to a large but finite number to avoid NaN calculations.
                        inverse_fitness = f64::MAX / 10.0;
                    }
                    sum_inverse_min_fitness += inverse_fitness;
                }
            }
        }

        let total_population_size = self.population_size;

        // Second pass to set the carrying capacity for each region.
        for (_, region) in self.regions_mut().iter_mut() {
            let mut capacity = 0;
            if sum_inverse_min_fitness > 0.0 {
                if let Some(min_score) = region.min_score() {
                    if min_score > 0.0 {
                        let mut inverse_fitness = 1.0 / min_score;
                        if inverse_fitness.is_infinite() {
                            inverse_fitness = f64::MAX / 10.0;
                        }
                        // The division should now be safe from producing NaN.
                        let capacity_float = total_population_size as f64
                            * (inverse_fitness / sum_inverse_min_fitness);
                        capacity = capacity_float.floor() as usize;
                    }
                }
            }
            region.set_carrying_capacity(Some(capacity));
        }
    }
}

#[cfg(test)]
mod test_update_carrying_capacities {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::dimensions::Dimensions; // Not directly used by update_carrying_capacities tests but by helpers
    use crate::world::organisms::Organisms; // Not directly used by update_carrying_capacities tests but by helpers
    use crate::world::regions::{Region, Regions};
    use std::collections::BTreeMap;
    use std::ops::RangeInclusive;

    // HELPER FUNCTIONS (copied from src/world/regions/update.rs test module)
    // These are general helpers that might be used by various tests related to regions.

    fn default_system_parameters() -> Vec<f64> {
        vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
    }

    #[allow(dead_code)] // This helper might not be used by all test files that copy it
    fn create_phenotype_with_problem_values(problem_values: &[f64]) -> Phenotype {
        let mut expressed_values = default_system_parameters();
        expressed_values.extend_from_slice(problem_values);
        Phenotype::new_for_test(expressed_values)
    }

    #[allow(dead_code)] // This helper might not be used by all test files that copy it
    fn create_test_organisms_from_problem_values(all_problem_values: Vec<Vec<f64>>) -> Organisms {
        let phenotypes: Vec<Phenotype> = all_problem_values
            .into_iter()
            .map(|pv| create_phenotype_with_problem_values(&pv))
            .collect();
        Organisms::new_from_phenotypes(phenotypes)
    }

    #[allow(dead_code)] // This helper might not be used by all test files that copy it
    fn create_test_organisms_single(p_values: &[f64]) -> Organisms {
        create_test_organisms_from_problem_values(vec![p_values.to_vec()])
    }

    fn create_test_regions_and_gc(
        max_regions: usize,
        population_size: usize,
    ) -> (Regions, GlobalConstants) {
        if population_size == 0 {
            let gc_temp = GlobalConstants::new(1, max_regions);
            let regions = Regions {
                regions: BTreeMap::new(),
                max_regions,
                population_size: 0,
            };
            return (regions, gc_temp);
        }

        let global_constants = GlobalConstants::new(population_size, max_regions);
        let regions = Regions::new(&global_constants);
        (regions, global_constants)
    }

    #[allow(dead_code)] // This helper might not be used by all test files that copy it
    fn create_test_dimensions(
        problem_bounds: Vec<RangeInclusive<f64>>,
        gc: &GlobalConstants,
    ) -> Dimensions {
        Dimensions::new(&problem_bounds, gc)
    }

    // Specific helper for update_carrying_capacities tests
    fn setup_region_with_min_score(min_score: Option<f64>) -> Region {
        let mut region = Region::new();
        region.set_min_score(min_score);
        region
    }

    #[test]
    fn given_no_regions_with_min_scores_when_update_capacities_then_all_capacities_zero() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct
            .regions_mut()
            .insert(key1.clone(), setup_region_with_min_score(None));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .regions
                .get(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_sum_inverse_fitness_zero_when_update_capacities_then_all_capacities_zero() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct
            .regions_mut()
            .insert(key1.clone(), setup_region_with_min_score(Some(0.0)));
        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .regions
                .get(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );

        regions_struct
            .regions_mut()
            .insert(key1.clone(), setup_region_with_min_score(Some(-5.0)));
        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .regions
                .get(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_one_region_with_positive_min_score_when_update_capacities_then_capacity_is_population_size()
     {
        let population_size = 10;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
        let key1 = vec![1];
        regions_struct
            .regions_mut()
            .insert(key1.clone(), setup_region_with_min_score(Some(5.0)));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .regions
                .get(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(population_size)
        );
    }

    #[test]
    fn given_multiple_regions_with_min_scores_when_update_capacities_then_capacities_proportional()
    {
        let population_size = 100;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
        let key1 = vec![1];
        let key2 = vec![2];

        // F1 = 10, F2 = 40. Sum(1/F) = 1/10 + 1/40 = 0.1 + 0.025 = 0.125
        // Cap1 = P * (1/F1) / Sum(1/F) = 100 * (0.1 / 0.125) = 100 * 0.8 = 80
        // Cap2 = P * (1/F2) / Sum(1/F) = 100 * (0.025 / 0.125) = 100 * 0.2 = 20
        regions_struct
            .regions_mut()
            .insert(key1.clone(), setup_region_with_min_score(Some(10.0)));
        regions_struct
            .regions_mut()
            .insert(key2.clone(), setup_region_with_min_score(Some(40.0)));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .regions
                .get(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(80)
        );
        assert_eq!(
            regions_struct
                .regions
                .get(&key2)
                .unwrap()
                .carrying_capacity(),
            Some(20)
        );
    }

    #[test]
    fn given_region_with_no_min_score_when_update_capacities_then_its_capacity_is_zero() {
        let population_size = 100;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
        let key_with_score = vec![1];
        let key_without_score = vec![2];

        regions_struct.regions_mut().insert(
            key_with_score.clone(),
            setup_region_with_min_score(Some(10.0)),
        );
        regions_struct
            .regions_mut()
            .insert(key_without_score.clone(), setup_region_with_min_score(None));

        regions_struct.update_carrying_capacities();

        // Region with score gets full capacity
        assert_eq!(
            regions_struct
                .regions
                .get(&key_with_score)
                .unwrap()
                .carrying_capacity(),
            Some(population_size)
        );
        // Region without score gets zero capacity
        assert_eq!(
            regions_struct
                .regions
                .get(&key_without_score)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_population_size_zero_when_update_capacities_then_all_capacities_zero() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 0); // Population size 0
        let key1 = vec![1];
        regions_struct
            .regions_mut()
            .insert(key1.clone(), setup_region_with_min_score(Some(10.0)));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .regions
                .get(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }
}
