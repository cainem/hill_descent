use std::collections::HashSet;

use crate::world::organisms::Organisms;

impl Organisms {
    /// Computes the number of distinct spatial locations among all organisms.
    ///
    /// Spatial locations are derived from the expressed gene values, excluding
    /// the initial system parameters.
    ///
    /// # Returns
    ///
    /// A `usize` representing the count of unique spatial locations.
    pub fn distinct_locations_count(&self) -> usize {
        let mut distinct_locations = HashSet::new();

        for organism in &self.organisms {
            let problem_expressed_values = organism.phenotype().expression_problem_values();
            if !problem_expressed_values.is_empty() {
                // Convert f64s to u64s for hashing
                let location_coords: Vec<u64> = problem_expressed_values
                    .iter()
                    .map(|&val| val.to_bits())
                    .collect();
                distinct_locations.insert(location_coords);
            } else {
                // Organism has no spatial parameters.
                // Count this as one unique "empty" spatial location.
                distinct_locations.insert(Vec::new());
            }
        }
        distinct_locations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::phenotype::Phenotype; // Ensure Phenotype is in scope for test helpers // For create_phenotype_with_spatial_coords

    // Helper function to create a Phenotype for testing, allowing overrides
    fn create_test_phenotype_with_override(
        expressed_override: Option<Vec<f64>>,
        _expressed_hash_override: Option<u64>, // This parameter is no longer used directly
    ) -> Phenotype {
        let default_expressed = vec![
            0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, // System parameters
            0.5, 0.5, // Example problem parameters
        ];

        let expressed_to_use = expressed_override.unwrap_or(default_expressed);

        Phenotype::new_for_test(expressed_to_use)
    }

    fn create_phenotype_with_spatial_coords(coords: &[f64]) -> Phenotype {
        let mut expressed_values = vec![0.0; NUM_SYSTEM_PARAMETERS]; // Dummy system params
        expressed_values.extend_from_slice(coords);
        create_test_phenotype_with_override(Some(expressed_values), None)
    }

    #[test]
    fn given_no_organisms_when_distinct_locations_count_then_returns_zero() {
        let organisms_collection = Organisms {
            // This is OK as Vec::new() can be Vec<Organism>
            organisms: Vec::new(),
        };
        assert_eq!(organisms_collection.distinct_locations_count(), 0);
    }

    #[test]
    fn given_one_organism_when_distinct_locations_count_then_returns_one() {
        let p = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let organisms_collection = Organisms::new_from_phenotypes(vec![p]);
        assert_eq!(organisms_collection.distinct_locations_count(), 1);
    }

    #[test]
    fn given_multiple_organisms_at_same_location_when_distinct_locations_count_then_returns_one() {
        let p1 = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let p2 = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let organisms_collection = Organisms::new_from_phenotypes(vec![p1, p2]);
        assert_eq!(organisms_collection.distinct_locations_count(), 1);
    }

    #[test]
    fn given_multiple_organisms_at_different_locations_when_distinct_locations_count_then_returns_correct_count()
     {
        let p1 = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let p2 = create_phenotype_with_spatial_coords(&[3.0, 4.0]);
        let p3 = create_phenotype_with_spatial_coords(&[1.0, 2.0]); // Duplicate of p1
        let p4 = create_phenotype_with_spatial_coords(&[5.0, 6.0]);
        let organisms_collection = Organisms::new_from_phenotypes(vec![p1, p2, p3, p4]);
        assert_eq!(organisms_collection.distinct_locations_count(), 3);
    }

    #[test]
    fn given_organisms_with_only_system_parameters_when_distinct_locations_count_then_returns_one_for_empty_location()
     {
        let phenotypes: Vec<Phenotype> = (0..3)
            .map(|_| {
                let sys_params_only = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]; // Only 7 system params
                create_test_phenotype_with_override(Some(sys_params_only), None)
            })
            .collect();
        // All organisms will have an empty spatial key Vec<f64>
        let organisms_collection = Organisms::new_from_phenotypes(phenotypes);
        assert_eq!(
            organisms_collection.distinct_locations_count(),
            1,
            "Expected 1 distinct location for organisms with no spatial parameters"
        );
    }

    #[test]
    fn given_organisms_with_varying_system_params_same_spatial_when_distinct_locations_count_then_returns_one()
     {
        let mut phenotypes: Vec<Phenotype> = Vec::new();

        // Organism 1
        let mut expressed1 = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]; // System params
        expressed1.extend_from_slice(&[1.0, 2.0]); // Spatial params
        phenotypes.push(create_test_phenotype_with_override(Some(expressed1), None));

        // Organism 2 - different system params, same spatial params
        let mut expressed2 = vec![0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1]; // Different system params
        expressed2.extend_from_slice(&[1.0, 2.0]); // Same spatial params
        phenotypes.push(create_test_phenotype_with_override(Some(expressed2), None));

        let organisms_collection = Organisms::new_from_phenotypes(phenotypes);
        assert_eq!(
            organisms_collection.distinct_locations_count(),
            1,
            "Expected 1 distinct location as spatial parameters are the same"
        );
    }
}
