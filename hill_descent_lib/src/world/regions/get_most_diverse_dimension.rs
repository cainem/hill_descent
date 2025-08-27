use super::{calculate_dimension_stats, find_most_diverse_index};

impl super::Regions {
    // This function is designed to identify the most diverse dimension among organisms
    // within a specific region. The determination of diversity is based on two main criteria:
    // 1. The number of distinct values in a dimension.
    // 2. The standard deviation of the values in a dimension.
    //
    // The function operates as follows:
    // - It first retrieves the organisms belonging to the region specified by the `key`.
    // - If the region contains fewer than two organisms, diversity cannot be measured, so it returns `None`.
    // - It then gathers the expressed genetic values (phenotypes) of these organisms.
    // - For each dimension, it calculates the count of unique values and the standard deviation.
    // - The dimensions are then compared: the one with the most unique values is considered the most diverse.
    // - In case of a tie in the number of unique values, the standard deviation is used as a tie-breaker; the dimension with the higher standard deviation is chosen.
    // - If all dimensions have only one unique value, it indicates no diversity, and the function returns `None`.
    // - Otherwise, it returns the index of the most diverse dimension.
    pub fn get_most_diverse_dimension(&self, key: &[usize]) -> Option<usize> {
        let region = self.regions.get(key)?;
        let organisms = region.organisms();

        // dividing the most diverse dimension will not result in any more populated regions
        if organisms.len() < 2 {
            return None;
        }

        let expressed_values: Vec<_> = organisms
            .iter()
            .map(|o| o.phenotype().expression_problem_values())
            .collect();

        let num_dimensions = expressed_values.first().map_or(0, |v| v.len());
        let dimension_stats =
            calculate_dimension_stats::calculate_dimension_stats(&expressed_values, num_dimensions);
        crate::trace!("dimension stats {dimension_stats:?}");

        let most_diverse_index = find_most_diverse_index::find_most_diverse_index(dimension_stats);

        #[cfg(feature = "enable-tracing")]
        if most_diverse_index.is_some() {
            for dim_idx in 0..num_dimensions {
                let mut values_in_dimension: Vec<f64> = expressed_values
                    .iter()
                    .map(|values| values[dim_idx])
                    .collect();
                values_in_dimension
                    .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                crate::trace!("dimension {dim_idx} sorted values: {values_in_dimension:?}");
            }
        }

        most_diverse_index
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::organisms::organism::Organism;
    use crate::world::regions::Regions;
    use crate::world::regions::region::Region;
    use std::rc::Rc;

    // Helper to create a valid phenotype for testing.
    // The first 7 values are system parameters and are ignored by the diversity function.
    // The subsequent values are the problem-specific dimensions we want to test.
    fn create_test_phenotype(problem_values: Vec<f64>) -> Phenotype {
        let mut expressed_values = vec![0.0; 7]; // Dummy system parameters
        expressed_values.extend(problem_values);
        Phenotype::new_for_test(expressed_values)
    }

    fn create_test_organism(phenotype: Phenotype) -> Rc<Organism> {
        Rc::new(Organism::new(Rc::new(phenotype), 0))
    }

    #[test]
    fn given_region_with_one_organism_when_get_most_diverse_dimension_then_returns_none() {
        let constants = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&constants);
        let key = vec![0];
        let mut region = Region::new();
        let organism = create_test_organism(create_test_phenotype(vec![1.0, 2.0]));
        region.add_organism(organism);
        regions.insert_region(key.clone(), region);

        assert_eq!(regions.get_most_diverse_dimension(&key), None);
    }

    #[test]
    fn given_organisms_with_one_diverse_dimension_when_get_most_diverse_dimension_then_returns_correct_index()
     {
        let constants = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&constants);
        let key = vec![0];
        let mut region = Region::new();

        // Problem dimension 0: no diversity
        // Problem dimension 1: diversity
        let org1 = create_test_organism(create_test_phenotype(vec![1.0, 1.0]));
        let org2 = create_test_organism(create_test_phenotype(vec![1.0, 2.0]));
        region.add_organism(org1);
        region.add_organism(org2);
        regions.insert_region(key.clone(), region);

        // The function looks at problem dimensions, so index 1 is correct.
        assert_eq!(regions.get_most_diverse_dimension(&key), Some(1));
    }

    #[test]
    fn given_tie_in_uniqueness_when_get_most_diverse_dimension_then_std_dev_breaks_tie() {
        let constants = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&constants);
        let key = vec![0];
        let mut region = Region::new();

        // Problem dimension 0: 2 unique, std dev low
        // Problem dimension 1: 2 unique, std dev high
        let org1 = create_test_organism(create_test_phenotype(vec![1.0, 1.0]));
        let org2 = create_test_organism(create_test_phenotype(vec![2.0, 100.0]));
        region.add_organism(org1);
        region.add_organism(org2);
        regions.insert_region(key.clone(), region);

        assert_eq!(regions.get_most_diverse_dimension(&key), Some(1));
    }

    #[test]
    fn given_no_diversity_when_get_most_diverse_dimension_then_returns_none() {
        let constants = GlobalConstants::new(10, 2);
        let mut regions = Regions::new(&constants);
        let key = vec![0];
        let mut region = Region::new();
        let org1 = create_test_organism(create_test_phenotype(vec![1.0, 1.0]));
        let org2 = create_test_organism(create_test_phenotype(vec![1.0, 1.0]));
        region.add_organism(org1);
        region.add_organism(org2);
        regions.insert_region(key.clone(), region);

        assert_eq!(regions.get_most_diverse_dimension(&key), None);
    }

    #[test]
    fn given_non_existent_key_when_get_most_diverse_dimension_then_returns_none() {
        let constants = GlobalConstants::new(10, 2);
        let regions = Regions::new(&constants);
        let key = vec![0];

        assert_eq!(regions.get_most_diverse_dimension(&key), None);
    }
}
