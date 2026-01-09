//! Carrying capacity calculation for regions.

use super::Regions;

impl Regions {
    /// Updates carrying capacities for all regions based on relative fitness.
    ///
    /// Uses the inverse fitness formula from the PDD:
    /// capacity_i = P * (1/score_i) / sum(1/score_j for all j)
    ///
    /// Where P is the total population size.
    ///
    /// # Special Cases
    ///
    /// - Regions with min_score ≈ 0 (infinite inverse fitness) get priority allocation
    /// - Multiple infinite fitness regions share capacity equally
    /// - Regions without scores get 0 capacity
    pub fn update_carrying_capacities(&mut self) {
        let total_population_size = self.population_size;

        // Categorize regions by fitness type
        let mut infinite_fitness_keys = Vec::new();
        let mut finite_fitness_data: Vec<(super::RegionKey, f64)> = Vec::new();

        for (key, region) in self.iter() {
            if let Some(min_score) = region.min_score() {
                let inverse_fitness = 1.0 / min_score;
                if inverse_fitness.is_infinite() {
                    infinite_fitness_keys.push(key.clone());
                } else {
                    finite_fitness_data.push((key.clone(), inverse_fitness));
                }
            }
        }

        // Reset all regions to 0 capacity first
        for (_, region) in self.iter_mut() {
            region.set_carrying_capacity(0);
        }

        // Allocate capacity based on fitness
        if !infinite_fitness_keys.is_empty() {
            // Infinite fitness regions get all capacity, divided equally
            let capacity_per_region = total_population_size / infinite_fitness_keys.len();
            let remainder = total_population_size % infinite_fitness_keys.len();

            for (i, key) in infinite_fitness_keys.iter().enumerate() {
                if let Some(region) = self.get_region_mut(key) {
                    let mut capacity = capacity_per_region;
                    if i < remainder {
                        capacity += 1;
                    }
                    region.set_carrying_capacity(capacity);
                }
            }
        } else if !finite_fitness_data.is_empty() {
            // Proportional allocation based on inverse fitness
            let sum_inverse_fitness: f64 = finite_fitness_data.iter().map(|(_, inv)| inv).sum();

            // Sort for deterministic remainder allocation
            finite_fitness_data.sort_by(|a, b| a.0.cmp(&b.0));

            let mut allocated_so_far = 0;
            for (i, (key, inverse_fitness)) in finite_fitness_data.iter().enumerate() {
                if let Some(region) = self.get_region_mut(key) {
                    let capacity = if i == finite_fitness_data.len() - 1 {
                        // Last region gets remaining to ensure exact total
                        total_population_size - allocated_so_far
                    } else {
                        let proportion = inverse_fitness / sum_inverse_fitness;
                        let cap = ((total_population_size as f64) * proportion).floor() as usize;
                        allocated_so_far += cap;
                        cap
                    };
                    region.set_carrying_capacity(capacity);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::GlobalConstants;
    use crate::world::regions::{OrganismEntry, RegionKey};

    fn make_regions(pop: usize) -> Regions {
        Regions::new(&GlobalConstants::new(pop, 10))
    }

    #[test]
    fn given_regions_with_scores_when_update_capacities_then_capacities_calculated() {
        let mut regions = make_regions(100);

        // Create two regions with different scores
        let key1 = RegionKey::new(vec![0]);
        let key2 = RegionKey::new(vec![1]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        }
        {
            let region2 = regions.get_or_insert(key2.clone());
            region2.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        }

        regions.update_carrying_capacities();

        // Both regions should have capacity set
        assert!(
            regions
                .get_region(&key1)
                .unwrap()
                .carrying_capacity()
                .is_some()
        );
        assert!(
            regions
                .get_region(&key2)
                .unwrap()
                .carrying_capacity()
                .is_some()
        );
    }

    #[test]
    fn given_region_with_lower_score_when_update_capacities_then_higher_capacity() {
        let mut regions = make_regions(100);

        // Region 1: score 1.0 (better) -> inverse fitness 1.0
        // Region 2: score 2.0 (worse) -> inverse fitness 0.5
        // Total inverse: 1.5
        // Region 1 should get 1.0/1.5 = 66.67% ≈ 66 or 67
        // Region 2 should get 0.5/1.5 = 33.33% ≈ 33 or 34

        let key1 = RegionKey::new(vec![0]);
        let key2 = RegionKey::new(vec![1]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        }
        {
            let region2 = regions.get_or_insert(key2.clone());
            region2.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        }

        regions.update_carrying_capacities();

        let cap1 = regions
            .get_region(&key1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2 = regions
            .get_region(&key2)
            .unwrap()
            .carrying_capacity()
            .unwrap();

        assert!(
            cap1 > cap2,
            "Region with lower score should have higher capacity: {} vs {}",
            cap1,
            cap2
        );
    }

    #[test]
    fn given_all_capacities_when_summed_then_equals_population_size() {
        let mut regions = make_regions(100);

        let key1 = RegionKey::new(vec![0]);
        let key2 = RegionKey::new(vec![1]);
        let key3 = RegionKey::new(vec![2]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        }
        {
            let region2 = regions.get_or_insert(key2.clone());
            region2.add_organism(OrganismEntry::new(2, 0, Some(3.0)));
        }
        {
            let region3 = regions.get_or_insert(key3.clone());
            region3.add_organism(OrganismEntry::new(3, 0, Some(5.0)));
        }

        regions.update_carrying_capacities();

        let total: usize = regions
            .iter()
            .map(|(_, r)| r.carrying_capacity().unwrap_or(0))
            .sum();

        assert_eq!(total, 100, "Sum of capacities should equal population size");
    }

    #[test]
    fn given_region_with_zero_score_when_update_capacities_then_gets_all_capacity() {
        let mut regions = make_regions(100);

        let key1 = RegionKey::new(vec![0]);
        let key2 = RegionKey::new(vec![1]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.add_organism(OrganismEntry::new(1, 0, Some(0.0))); // Perfect score
        }
        {
            let region2 = regions.get_or_insert(key2.clone());
            region2.add_organism(OrganismEntry::new(2, 0, Some(1.0)));
        }

        regions.update_carrying_capacities();

        let cap1 = regions
            .get_region(&key1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2 = regions
            .get_region(&key2)
            .unwrap()
            .carrying_capacity()
            .unwrap();

        assert_eq!(cap1, 100, "Region with zero score should get all capacity");
        assert_eq!(
            cap2, 0,
            "Region with non-zero score should get 0 when infinite exists"
        );
    }

    #[test]
    fn given_multiple_zero_score_regions_when_update_capacities_then_shared_equally() {
        let mut regions = make_regions(100);

        let key1 = RegionKey::new(vec![0]);
        let key2 = RegionKey::new(vec![1]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.add_organism(OrganismEntry::new(1, 0, Some(0.0)));
        }
        {
            let region2 = regions.get_or_insert(key2.clone());
            region2.add_organism(OrganismEntry::new(2, 0, Some(0.0)));
        }

        regions.update_carrying_capacities();

        let cap1 = regions
            .get_region(&key1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        let cap2 = regions
            .get_region(&key2)
            .unwrap()
            .carrying_capacity()
            .unwrap();

        assert_eq!(cap1 + cap2, 100, "Total should equal population size");
        assert_eq!(cap1, 50, "Capacity should be split equally");
        assert_eq!(cap2, 50, "Capacity should be split equally");
    }

    #[test]
    fn given_no_scored_regions_when_update_capacities_then_all_zero() {
        let mut regions = make_regions(100);

        let key1 = RegionKey::new(vec![0]);

        {
            let region1 = regions.get_or_insert(key1.clone());
            region1.add_organism(OrganismEntry::new(1, 0, None)); // No score
        }

        regions.update_carrying_capacities();

        let cap1 = regions
            .get_region(&key1)
            .unwrap()
            .carrying_capacity()
            .unwrap();
        assert_eq!(cap1, 0, "Region without score should have 0 capacity");
    }
}
