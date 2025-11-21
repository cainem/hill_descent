use crate::world::dimensions::{
    CalculateDimensionsKeyResult, Dimensions, calculate_dimensions_key,
};
use crate::world::organisms::organism::Organism;
use crate::world::regions::region::region_key::RegionKey;

pub enum OrganismUpdateRegionKeyResult {
    Success,
    OutOfBounds(usize), // Index of the dimension that caused the error
}

impl Organism {
    /// Updates the `region_key` of the organism based on its phenotype's expressed values
    /// (excluding system parameters) and the provided dimensions.
    ///
    /// # Arguments
    /// * `dimensions_container`: A reference to the `Dimensions` struct containing the
    ///   definitions for each dimension of the problem space.
    ///
    /// # Returns
    /// * `OrganismUpdateRegionKeyResult::Success` if the region key was successfully calculated and updated.
    /// * `OrganismUpdateRegionKeyResult::OutOfBounds(usize)` if a value was out of bounds for a dimension, returning the
    ///   index of the failing dimension.
    ///
    /// # Panics
    /// * Panics if the number of problem-specific expressed values from the phenotype
    ///   does not match the number of dimensions defined in `dimensions_container`,
    ///   unless both are zero. This panic originates from `calculate_dimensions_key`.
    // #[cfg_attr(
    //     feature = "enable-tracing",
    //     tracing::instrument(level = "trace", skip(self, dimensions_container))
    // )]
    pub fn update_region_key(
        &self,
        dimensions_container: &Dimensions,
        dimension_changed: Option<usize>,
    ) -> OrganismUpdateRegionKeyResult {
        if self.region_key().is_none() {
            // Full recalculation if no key exists yet.
            let problem_expressed_values = self.phenotype().expression_problem_values();
            let actual_dimensions = dimensions_container.get_dimensions();

            match calculate_dimensions_key(actual_dimensions, problem_expressed_values) {
                CalculateDimensionsKeyResult::Success(key) => {
                    self.set_region_key(Some(RegionKey::from(key)));
                    OrganismUpdateRegionKeyResult::Success
                }
                CalculateDimensionsKeyResult::Failure {
                    dimension_index, ..
                } => {
                    self.set_region_key(None);
                    OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index)
                }
            }
        } else if let Some(dim_idx) = dimension_changed {
            // Optimized path: only one dimension has changed.
            crate::trace!("update_region_key: optimized path taken for dimension {dim_idx}");
            let mut current_key = self
                .take_region_key()
                .expect("Cached region key expected when dimension_changed is provided");
            let dimension = dimensions_container.get_dimension(dim_idx);
            let value = self.phenotype().expression_problem_values()[dim_idx];

            match dimension.get_interval(value) {
                Some(interval) => {
                    current_key.update_position(dim_idx, interval);
                    self.set_region_key(Some(current_key));
                    OrganismUpdateRegionKeyResult::Success
                }
                None => {
                    // Key is already removed (taken), so just ensure it stays None (which it is)
                    OrganismUpdateRegionKeyResult::OutOfBounds(dim_idx)
                }
            }
        } else {
            // Nothing has changed; return success.
            OrganismUpdateRegionKeyResult::Success
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use crate::phenotype::Phenotype;
    use crate::world::dimensions::Dimensions;
    use crate::world::dimensions::dimension::Dimension;
    use std::ops::RangeInclusive;
    use std::sync::Arc;

    // ---------- helpers ----------
    fn create_organism_for_test(problem_values: Vec<f64>) -> Organism {
        // prepend dummy system parameters so Phenotype::new_for_test works
        let mut expressed: Vec<f64> = vec![0.0; NUM_SYSTEM_PARAMETERS];
        expressed.extend(problem_values);
        let phenotype = Phenotype::new_for_test(expressed);
        let organism = Organism::new(Arc::new(phenotype), 0, (None, None));
        organism.set_region_key(None);
        organism
    }

    fn create_dimensions(spec: &[(RangeInclusive<f64>, usize)]) -> Dimensions {
        let dims: Vec<Dimension> = spec
            .iter()
            .map(|(bounds, div)| {
                let mut d = Dimension::new(bounds.clone(), 0);
                d.set_number_of_doublings(*div);
                d
            })
            .collect();
        Dimensions::new_for_test(dims)
    }

    // ---------- tests: full recompute branch ----------
    #[test]
    fn given_valid_inputs_when_dimension_changed_none_then_success_and_key_set() {
        let organism = create_organism_for_test(vec![7.5, 60.0]);
        let dims = create_dimensions(&[(0.0..=10.0, 2), (0.0..=100.0, 4)]);
        let result = organism.update_region_key(&dims, None);
        assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
        // With binary doubling: 2 doublings = 4 intervals, 4 doublings = 16 intervals
        // Position [7.5, 60.0] maps to intervals [3, 9]
        assert_eq!(
            organism.region_key().map(Vec::<usize>::from),
            Some(vec![3, 9])
        );
    }

    #[test]
    fn given_out_of_bounds_value_when_dimension_changed_none_then_failure_and_key_none() {
        let organism = create_organism_for_test(vec![12.0, 60.0]); // 12.0 out of first dim bounds
        let dims = create_dimensions(&[(0.0..=10.0, 2), (0.0..=100.0, 4)]);
        let result = organism.update_region_key(&dims, None);
        assert!(matches!(
            result,
            OrganismUpdateRegionKeyResult::OutOfBounds(0)
        ));
        assert!(organism.region_key().is_none());
    }

    // ---------- tests: single-dimension fast path ----------
    #[test]
    fn given_single_dimension_in_bounds_when_region_key_exists_then_success_and_key_updated() {
        // initial organism setup
        let organism = create_organism_for_test(vec![7.5, 60.0]);
        let dims = create_dimensions(&[(0.0..=10.0, 2), (0.0..=100.0, 4)]);

        // compute full key first so region_key is Some
        assert!(matches!(
            organism.update_region_key(&dims, None),
            OrganismUpdateRegionKeyResult::Success
        ));
        assert_eq!(
            organism.region_key().map(Vec::<usize>::from),
            Some(vec![3, 9])
        );

        // Manually mark existing key then craft organism with changed value.
        // Simulate cache by explicitly setting an old key then calling fast path.
        organism.set_region_key(Some(vec![3, 9].into()));

        // Build a new organism with second value 25.0 and an old cached key [3,9]
        let organism2 = create_organism_for_test(vec![7.5, 25.0]);
        organism2.set_region_key(Some(vec![3, 9].into()));

        // call fast-path with Some(1) on organism2
        let result = organism2.update_region_key(&dims, Some(1));
        assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
        assert_eq!(
            organism2.region_key().map(Vec::<usize>::from),
            Some(vec![3, 4])
        );
    }

    #[test]
    fn given_single_dimension_out_of_bounds_when_region_key_exists_then_failure_and_key_cleared() {
        let organism = create_organism_for_test(vec![7.5, 60.0]);
        let dims = create_dimensions(&[(0.0..=10.0, 2), (0.0..=100.0, 4)]);
        // establish existing key
        organism.update_region_key(&dims, None);

        // Build new organism with first value out of bounds (-1.0) but cached key present
        let organism2 = create_organism_for_test(vec![-1.0, 60.0]);
        organism2.set_region_key(Some(vec![3, 9].into()));

        let result = organism2.update_region_key(&dims, Some(0));
        assert!(matches!(
            result,
            OrganismUpdateRegionKeyResult::OutOfBounds(0)
        ));
        assert!(organism2.region_key().is_none());
    }

    // ---------- tests: dimension_changed_some_but_no_cached_key ----------
    #[test]
    fn given_dimension_changed_some_but_region_key_none_then_full_recompute_occurs() {
        let organism = create_organism_for_test(vec![5.0, 50.0]);
        // note: region_key is None by default from helper
        let dims = create_dimensions(&[(0.0..=10.0, 2), (0.0..=100.0, 4)]);
        let result = organism.update_region_key(&dims, Some(0)); // Some index but no cached key yet
        assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
        assert_eq!(
            organism.region_key().map(Vec::<usize>::from),
            Some(vec![2, 8])
        );
    }

    // ---------- tests: no-op branch ----------
    #[test]
    fn given_region_key_exists_when_no_dimension_changed_then_no_op_success() {
        let organism = create_organism_for_test(vec![7.5, 60.0]);
        let dims = create_dimensions(&[(0.0..=10.0, 2), (0.0..=100.0, 4)]);

        // First establish a region key
        let result = organism.update_region_key(&dims, None);
        assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
        let initial_key = organism.region_key();
        assert_eq!(
            initial_key.clone().map(Vec::<usize>::from),
            Some(vec![3, 9])
        );

        // Now call with dimension_changed = None when region_key already exists
        // This should be a no-op and return success without changing the key
        let result = organism.update_region_key(&dims, None);
        assert!(matches!(result, OrganismUpdateRegionKeyResult::Success));
        assert_eq!(organism.region_key(), initial_key); // Key should remain unchanged
    }
}
