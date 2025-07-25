use crate::world::{dimensions::Dimensions, organisms::Organisms};

impl super::Regions {
    /// Handles the successful update of all organism region keys.
    ///
    /// This function repopulates the regions with the organisms, prunes any
    /// regions that are now empty, and then determines if the simulation
    /// should continue dividing dimensions or stop.
    ///
    /// # Returns
    ///
    /// Returns `true` if the simulation has reached a stable state and should
    /// stop, `false` otherwise.
    pub(super) fn handle_successful_update(
        &mut self,
        organisms: &mut Organisms,
        dimensions: &mut Dimensions,
    ) -> bool {
        // Before adding the current generation of organisms, clear the regions of any
        // organisms from the previous generation. This ensures the region state is
        // always in sync with the master organism list.
        for region in self.regions.values_mut() {
            region.clear_organisms();
        }

        self.add_phenotypes(organisms);
        self.prune_empty_regions();

        // Stop if we've hit the max number of regions, if all organisms are in one region,
        // or if every distinct location already has its own region (further subdivision
        // cannot increase the populated region count).
        if self.regions.len() >= self.max_regions
            || organisms.distinct_locations_count() <= 1
            || self.regions.len() == organisms.distinct_locations_count()
        {
            return true; // Stable state reached.
        }

        // Try to divide the dimension with the highest organism count.
        if dimensions.divide_next_dimension() {
            // The dimension change invalidates all existing region keys.
            // Clear all regions so they can be rebuilt in the next iteration.
            self.regions.clear();
            false // Continue loop
        } else {
            // No more divisions possible.
            true // Stable state: cannot refine further.
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};
    use std::ops::RangeInclusive;

    fn default_system_parameters() -> Vec<f64> {
        vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
    }

    fn phenotype_with_problem_values(problem_values: &[f64]) -> Phenotype {
        let mut expressed = default_system_parameters();
        expressed.extend_from_slice(problem_values);
        Phenotype::new_for_test(expressed)
    }

    fn organisms_from_problem_values(values: Vec<Vec<f64>>) -> Organisms {
        let phenotypes: Vec<Phenotype> = values
            .into_iter()
            .map(|pv| phenotype_with_problem_values(&pv))
            .collect();
        Organisms::new_from_phenotypes(phenotypes)
    }

    fn setup(max_regions: usize, bounds: Vec<RangeInclusive<f64>>) -> (Regions, Dimensions) {
        let gc = GlobalConstants::new(100, max_regions);
        let regions = Regions::new(&gc);
        let dimensions = Dimensions::new(&bounds, &gc);
        (regions, dimensions)
    }

    #[test]
    fn given_max_regions_already_reached_when_handle_successful_update_then_returns_true() {
        let (mut regions, mut dims) = setup(1, vec![0.0..=1.0]);
        let mut organisms = organisms_from_problem_values(vec![vec![0.5]]);
        let finished = regions.handle_successful_update(&mut organisms, &mut dims);
        assert!(finished);
    }

    #[test]
    fn given_all_organisms_same_location_when_handle_successful_update_then_returns_true() {
        let (mut regions, mut dims) = setup(10, vec![0.0..=1.0, 0.0..=1.0]);
        let mut organisms = organisms_from_problem_values(vec![vec![0.5, 0.5]; 3]);
        let finished = regions.handle_successful_update(&mut organisms, &mut dims);
        assert!(finished);
    }

    #[test]
    fn given_each_location_has_own_region_when_handle_successful_update_then_returns_true() {
        let (mut regions, mut dims) = setup(10, vec![0.0..=1.0, 0.0..=1.0]);
        dims.divide_next_dimension();
        dims.divide_next_dimension();
        let mut organisms = organisms_from_problem_values(vec![vec![0.1, 0.1], vec![0.9, 0.9]]);
        let _ = organisms.update_all_region_keys(&dims);
        let finished = regions.handle_successful_update(&mut organisms, &mut dims);
        assert!(finished);
    }

    #[test]
    fn given_possible_to_divide_further_when_handle_successful_update_then_returns_false_and_clears_regions()
     {
        let (mut regions, mut dims) = setup(10, vec![0.0..=1.0]);
        let mut organisms = organisms_from_problem_values(vec![vec![0.1], vec![0.9]]);
        let finished = regions.handle_successful_update(&mut organisms, &mut dims);
        assert!(!finished);
        assert!(regions.regions().is_empty());
    }

    #[test]
    fn given_no_dimensions_to_divide_when_handle_successful_update_then_returns_true() {
        let (mut regions, mut dims) = setup(10, vec![]);
        let mut organisms = organisms_from_problem_values(vec![vec![]]);
        let finished = regions.handle_successful_update(&mut organisms, &mut dims);
        assert!(finished);
    }
}
