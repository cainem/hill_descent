impl super::Regions {
    /// Resets all regions for the next iteration of the simulation.
    ///
    /// This method iterates through all the regions and calls their respective
    /// `reset` methods. This clears their organism lists and resets transient
    /// stats like carrying capacity, while persistent stats like min_score are kept.
    pub fn reset(&mut self) {
        for region in self.regions.values_mut() {
            region.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::regions::Regions;
    use crate::world::regions::region::Region;

    fn create_test_phenotype() -> Rc<Phenotype> {
        // A phenotype requires at least 7 values for its system parameters.
        let system_params = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0];
        Rc::new(Phenotype::new_for_test(system_params))
    }

    fn create_test_regions(max_regions: usize, population_size: usize) -> Regions {
        let global_constants = GlobalConstants::new(population_size, max_regions);
        Regions::new(&global_constants)
    }

    #[test]
    fn given_populated_regions_when_reset_is_called_then_all_regions_are_reset() {
        // Given
        let mut regions = create_test_regions(4, 10);
        let key = vec![0];

        let mut region = Region::default();
        let phenotype = create_test_phenotype();
        region.add_phenotype(phenotype);
        region.set_min_score(Some(0.5));
        region.set_carrying_capacity(Some(10));

        regions.regions.insert(key.clone(), region);

        assert!(
            !regions.get_region(&key).unwrap().is_empty(),
            "Pre-condition: region should not be empty"
        );

        // When
        regions.reset();

        // Then
        let reset_region = regions.get_region(&key).unwrap();
        assert!(
            reset_region.is_empty(),
            "Post-condition: region should be empty"
        );
        assert_eq!(
            reset_region.min_score(),
            Some(0.5),
            "Post-condition: min_score should be preserved"
        );
        assert!(
            reset_region.carrying_capacity().is_none(),
            "Post-condition: carrying_capacity should be None"
        );
    }
}
