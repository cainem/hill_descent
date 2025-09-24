use crate::world::regions::Regions;

impl Regions {
    /// Updates the carrying capacity for all regions.
    ///
    /// Calculation is based on PDD section 4.2.4:
    /// P_i = P * (1/F_i) / sum_over_j(1/F_j)
    /// where P is total target population_size, F_i is min_score in region i.
    ///
    /// Special handling for infinite inverse fitness:
    /// - If regions have infinite inverse fitness (min_score ≈ 0), they get priority
    /// - If multiple regions have infinite inverse fitness, capacity is divided equally among them
    /// - Regions with finite inverse fitness use proportional allocation only if no infinite regions exist
    ///
    /// This approach prevents floating-point overflows that can occur when many regions
    /// have very large inverse fitness values.
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self))
    )]
    #[allow(dead_code)]
    pub(super) fn update_carrying_capacities(&mut self) {
        let total_population_size = self.population_size;

        // First, identify regions with infinite and finite inverse fitness
        let mut infinite_fitness_regions = Vec::new();
        let mut finite_fitness_data = Vec::new(); // (key, region_ref, inverse_fitness)

        for (key, region) in self.iter_regions() {
            if let Some(min_score) = region.min_score()
                && min_score > 0.0
            {
                let inverse_fitness = 1.0 / min_score;
                if inverse_fitness.is_infinite() {
                    infinite_fitness_regions.push(key.clone());
                } else {
                    finite_fitness_data.push((key.clone(), inverse_fitness));
                }
            }
        }

        // First, set all regions to 0 capacity
        for (_, region) in self.iter_regions_mut() {
            region.set_carrying_capacity(Some(0));
        }

        // If there are regions with infinite inverse fitness, they get all the capacity
        if !infinite_fitness_regions.is_empty() {
            let capacity_per_infinite_region =
                total_population_size / infinite_fitness_regions.len();
            let remainder = total_population_size % infinite_fitness_regions.len();

            // Set capacity for infinite fitness regions
            for (i, key) in infinite_fitness_regions.iter().enumerate() {
                if let Some(region) = self.get_region_mut(key) {
                    let mut capacity = capacity_per_infinite_region;
                    // Distribute remainder among first few regions
                    if i < remainder {
                        capacity += 1;
                    }
                    region.set_carrying_capacity(Some(capacity));
                }
            }
        } else if !finite_fitness_data.is_empty() {
            // Handle regions with finite inverse fitness using proportional allocation
            let sum_inverse_fitness: f64 =
                finite_fitness_data.iter().map(|(_, inv_fit)| inv_fit).sum();

            // Calculate capacities and track allocated total
            let mut allocated_so_far = 0;
            let mut finite_regions: Vec<_> = finite_fitness_data.iter().collect();

            // Sort by key for deterministic remainder allocation
            finite_regions.sort_by(|a, b| a.0.cmp(&b.0));

            for (i, (key, inverse_fitness)) in finite_regions.iter().enumerate() {
                if let Some(region) = self.get_region_mut(key) {
                    let capacity = if i == finite_regions.len() - 1 {
                        // Last region gets remaining capacity to ensure exact total
                        total_population_size - allocated_so_far
                    } else {
                        let capacity_float =
                            total_population_size as f64 * (*inverse_fitness / sum_inverse_fitness);
                        let capacity = capacity_float.floor() as usize;
                        allocated_so_far += capacity;
                        capacity
                    };
                    region.set_carrying_capacity(Some(capacity));
                }
            }
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
        target_regions: usize,
        population_size: usize,
    ) -> (Regions, GlobalConstants) {
        let global_constants = GlobalConstants::new(population_size, target_regions);
        let regions = Regions::new(&global_constants);
        (regions, global_constants)
    }

    #[allow(dead_code)] // This helper might not be used by all test files that copy it
    fn create_test_dimensions(problem_bounds: Vec<RangeInclusive<f64>>) -> Dimensions {
        Dimensions::new(&problem_bounds)
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
        regions_struct.insert_region(key1.clone(), setup_region_with_min_score(None));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .get_region(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_sum_inverse_fitness_zero_when_update_capacities_then_all_capacities_zero() {
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, 10);
        let key1 = vec![1];
        regions_struct.insert_region(key1.clone(), setup_region_with_min_score(Some(0.0)));
        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .get_region(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );

        // Test with negative score as well
        regions_struct.insert_region(key1.clone(), setup_region_with_min_score(Some(-5.0)));
        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .get_region(&key1)
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
        regions_struct.insert_region(key1.clone(), setup_region_with_min_score(Some(5.0)));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .get_region(&key1)
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
        regions_struct.insert_region(key1.clone(), setup_region_with_min_score(Some(10.0)));
        regions_struct.insert_region(key2.clone(), setup_region_with_min_score(Some(40.0)));

        regions_struct.update_carrying_capacities();
        assert_eq!(
            regions_struct
                .get_region(&key1)
                .unwrap()
                .carrying_capacity(),
            Some(80)
        );
        assert_eq!(
            regions_struct
                .get_region(&key2)
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

        regions_struct.insert_region(
            key_with_score.clone(),
            setup_region_with_min_score(Some(10.0)),
        );
        regions_struct.insert_region(key_without_score.clone(), setup_region_with_min_score(None));

        regions_struct.update_carrying_capacities();

        // Region with score gets full capacity
        assert_eq!(
            regions_struct
                .get_region(&key_with_score)
                .unwrap()
                .carrying_capacity(),
            Some(population_size)
        );
        // Region without score gets zero capacity
        assert_eq!(
            regions_struct
                .get_region(&key_without_score)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_single_region_with_infinite_inverse_fitness_when_update_capacities_then_gets_all_capacity()
     {
        let population_size = 100;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
        let key_infinite = vec![1];
        let key_finite = vec![2];

        // Very small min_score leads to infinite inverse fitness
        let very_small = 1e-320; // This will cause 1.0/very_small to be infinite
        regions_struct.insert_region(
            key_infinite.clone(),
            setup_region_with_min_score(Some(very_small)),
        );
        regions_struct.insert_region(key_finite.clone(), setup_region_with_min_score(Some(10.0)));

        regions_struct.update_carrying_capacities();

        // Region with infinite inverse fitness gets all capacity
        assert_eq!(
            regions_struct
                .get_region(&key_infinite)
                .unwrap()
                .carrying_capacity(),
            Some(population_size)
        );
        // Region with finite inverse fitness gets zero capacity
        assert_eq!(
            regions_struct
                .get_region(&key_finite)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_multiple_regions_with_infinite_inverse_fitness_when_update_capacities_then_capacity_divided_equally()
     {
        let population_size = 100;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
        let key1 = vec![1];
        let key2 = vec![2];
        let key3 = vec![3];
        let key_finite = vec![4];

        // Three regions with infinite inverse fitness (using very small values that cause overflow to infinity)
        let very_small = 1e-320; // This will cause 1.0/very_small to be infinite
        regions_struct.insert_region(key1.clone(), setup_region_with_min_score(Some(very_small)));
        regions_struct.insert_region(key2.clone(), setup_region_with_min_score(Some(very_small)));
        regions_struct.insert_region(key3.clone(), setup_region_with_min_score(Some(very_small)));
        // One region with finite inverse fitness
        regions_struct.insert_region(key_finite.clone(), setup_region_with_min_score(Some(10.0)));

        regions_struct.update_carrying_capacities();

        // Each infinite region gets 33 capacity (100/3 = 33.33, floored)
        // One region gets the remainder (100 % 3 = 1)
        let capacities = [
            regions_struct
                .get_region(&key1)
                .unwrap()
                .carrying_capacity()
                .unwrap(),
            regions_struct
                .get_region(&key2)
                .unwrap()
                .carrying_capacity()
                .unwrap(),
            regions_struct
                .get_region(&key3)
                .unwrap()
                .carrying_capacity()
                .unwrap(),
        ];

        // Two regions should get 33, one should get 34 (33 + remainder of 1)
        assert_eq!(capacities.iter().sum::<usize>(), population_size);
        assert!(capacities.iter().all(|&c| c == 33 || c == 34));
        assert_eq!(capacities.iter().filter(|&&c| c == 34).count(), 1);
        assert_eq!(capacities.iter().filter(|&&c| c == 33).count(), 2);

        // Finite region gets zero capacity
        assert_eq!(
            regions_struct
                .get_region(&key_finite)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }

    #[test]
    fn given_many_regions_with_very_small_scores_when_update_capacities_then_no_overflow() {
        let population_size = 1000;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(20, population_size);

        // Create 15 regions with very small min_scores that would cause overflow in the old implementation
        let very_small = 1e-320; // This will cause 1.0/very_small to be infinite
        for i in 1..=15 {
            let key = vec![i];
            // This would create infinite inverse fitness, causing overflow in the old implementation
            // With 15 such regions, the sum would overflow
            regions_struct.insert_region(key, setup_region_with_min_score(Some(very_small)));
        }

        // This should not panic or produce NaN values
        regions_struct.update_carrying_capacities();

        // Verify all infinite regions get equal share of capacity
        let mut total_allocated = 0;
        for i in 1..=15 {
            let key = vec![i];
            let capacity = regions_struct
                .get_region(&key)
                .unwrap()
                .carrying_capacity()
                .unwrap();
            total_allocated += capacity;
            // Each region should get approximately population_size / 15
            assert!((66..=67).contains(&capacity)); // 1000/15 = 66.67
        }
        assert_eq!(total_allocated, population_size);
    }

    #[test]
    fn given_mix_of_zero_and_positive_min_scores_when_update_capacities_then_only_positive_get_capacity()
     {
        let population_size = 100;
        let (mut regions_struct, _gc) = create_test_regions_and_gc(4, population_size);
        let key_zero = vec![1];
        let key_negative = vec![2];
        let key_positive = vec![3];
        let key_none = vec![4];

        regions_struct.insert_region(key_zero.clone(), setup_region_with_min_score(Some(0.0)));
        regions_struct.insert_region(
            key_negative.clone(),
            setup_region_with_min_score(Some(-5.0)),
        );
        regions_struct.insert_region(
            key_positive.clone(),
            setup_region_with_min_score(Some(10.0)),
        );
        regions_struct.insert_region(key_none.clone(), setup_region_with_min_score(None));

        regions_struct.update_carrying_capacities();

        // Only the positive score region gets capacity
        assert_eq!(
            regions_struct
                .get_region(&key_positive)
                .unwrap()
                .carrying_capacity(),
            Some(population_size)
        );
        // All others get zero
        assert_eq!(
            regions_struct
                .get_region(&key_zero)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
        assert_eq!(
            regions_struct
                .get_region(&key_negative)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
        assert_eq!(
            regions_struct
                .get_region(&key_none)
                .unwrap()
                .carrying_capacity(),
            Some(0)
        );
    }
}
