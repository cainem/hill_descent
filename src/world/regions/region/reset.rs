impl super::Region {
    /// Resets the region for the next iteration of the simulation.
    ///
    /// This clears the list of organisms and resets the carrying capacity to its
    /// default `None` state. The minimum score is persistent and not reset.
    pub fn reset(&mut self) {
        self.organisms.clear();
        self.set_carrying_capacity(None);
    }

    /// Checks if the region contains any organisms.
    pub fn is_empty(&self) -> bool {
        self.organisms.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{phenotype::Phenotype, world::regions::region::Region};

    fn create_test_phenotype() -> Rc<Phenotype> {
        // A phenotype requires at least 7 values for its system parameters.
        let system_params = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0];
        Rc::new(Phenotype::new_for_test(system_params))
    }

    #[test]
    fn given_a_populated_region_when_reset_is_called_then_the_region_is_emptied_and_reset() {
        // Given
        let mut region = Region::default();
        let phenotype = create_test_phenotype();
        region.add_phenotype(phenotype);
        region.set_min_score(Some(0.5));
        region.set_carrying_capacity(Some(10));

        assert!(
            !region.is_empty(),
            "Pre-condition: region should not be empty"
        );
        assert!(
            region.min_score().is_some(),
            "Pre-condition: min_score should be set"
        );
        assert!(
            region.carrying_capacity().is_some(),
            "Pre-condition: carrying_capacity should be set"
        );

        // When
        region.reset();

        // Then
        assert!(region.is_empty(), "Post-condition: region should be empty");
        assert_eq!(
            region.min_score(),
            Some(0.5),
            "Post-condition: min_score should be preserved"
        );
        assert!(
            region.carrying_capacity().is_none(),
            "Post-condition: carrying_capacity should be None"
        );
    }
}
