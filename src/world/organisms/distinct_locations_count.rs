use std::collections::HashSet;

use crate::{NUM_SYSTEM_PARAMETERS, world::organisms::Organisms};

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

        for phenotype in &self.organisms {
            let expressed_values = phenotype.expressed_values();
            if expressed_values.len() > NUM_SYSTEM_PARAMETERS {
                // Convert f64s to u64s for hashing, skip system parameters
                let location_coords: Vec<u64> = expressed_values[NUM_SYSTEM_PARAMETERS..]
                    .iter()
                    .map(|&val| val.to_bits())
                    .collect();
                distinct_locations.insert(location_coords);
            } else {
                // Organism has no spatial parameters or only system parameters.
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
    use crate::{NUM_SYSTEM_PARAMETERS, phenotype::Phenotype};

    // Helper function to create a Phenotype for testing, allowing overrides
    fn create_test_phenotype_with_override(
        expressed_override: Option<Vec<f64>>,
        _expressed_hash_override: Option<u64>, // This parameter is no longer used directly
    ) -> Phenotype {
        let default_genes_for_system_params = vec![0.0; NUM_SYSTEM_PARAMETERS];
        let default_spatial_genes = vec![1.0, 2.0, 3.0];
        let mut default_expressed = default_genes_for_system_params;
        default_expressed.extend(default_spatial_genes);

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
            organisms: Vec::new(),
        };
        assert_eq!(organisms_collection.distinct_locations_count(), 0);
    }

    #[test]
    fn given_one_organism_when_distinct_locations_count_then_returns_one() {
        let p = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let organisms_collection = Organisms { organisms: vec![p] };
        assert_eq!(organisms_collection.distinct_locations_count(), 1);
    }

    #[test]
    fn given_multiple_organisms_at_same_location_when_distinct_locations_count_then_returns_one() {
        let p1 = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let p2 = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let organisms_collection = Organisms {
            organisms: vec![p1, p2],
        };
        assert_eq!(organisms_collection.distinct_locations_count(), 1);
    }

    #[test]
    fn given_multiple_organisms_at_different_locations_when_distinct_locations_count_then_returns_correct_count()
     {
        let p1 = create_phenotype_with_spatial_coords(&[1.0, 2.0]);
        let p2 = create_phenotype_with_spatial_coords(&[3.0, 4.0]);
        let p3 = create_phenotype_with_spatial_coords(&[1.0, 2.0]); // Duplicate of p1
        let p4 = create_phenotype_with_spatial_coords(&[5.0, 6.0]);
        let organisms_collection = Organisms {
            organisms: vec![p1, p2, p3, p4],
        };
        assert_eq!(organisms_collection.distinct_locations_count(), 3);
    }

    #[test]
    fn given_organisms_with_only_system_parameters_when_distinct_locations_count_then_returns_one_for_empty_location()
     {
        let expressed_only_system = vec![0.1; NUM_SYSTEM_PARAMETERS];
        let p1 = create_test_phenotype_with_override(Some(expressed_only_system), None);

        let organisms_collection = Organisms {
            organisms: vec![p1],
        };
        // An empty Vec<u64> (representing no spatial coords) is inserted into the HashSet.
        assert_eq!(
            organisms_collection.distinct_locations_count(),
            1,
            "Should count one unique 'empty' location"
        );
    }

    #[test]
    fn given_organisms_with_varying_system_params_same_spatial_when_distinct_locations_count_then_returns_one()
     {
        let mut sys_params1 = vec![0.1; NUM_SYSTEM_PARAMETERS];
        sys_params1.extend_from_slice(&[10.0, 20.0]);
        let p1 = create_test_phenotype_with_override(Some(sys_params1), None);

        let mut sys_params2 = vec![0.2; NUM_SYSTEM_PARAMETERS];
        sys_params2.extend_from_slice(&[10.0, 20.0]);
        let p2 = create_test_phenotype_with_override(Some(sys_params2), None);

        let organisms_collection = Organisms {
            organisms: vec![p1, p2],
        };
        assert_eq!(organisms_collection.distinct_locations_count(), 1);
    }
}
