use crate::world::{dimensions::Dimensions, organisms::Organisms};
use crate::{debug, trace, warn};

#[derive(Debug)]
pub enum AdjustRegionsResult {
    DimensionExpanded { dimension_index: usize },
    ExpansionNotNecessary,
    AtResolutionLimit,
}

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
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "trace", skip(self, organisms, dimensions))
    )]
    pub(super) fn adjust_regions(
        &mut self,
        organisms: &mut Organisms,
        dimensions: &mut Dimensions,
    ) -> AdjustRegionsResult {
        // place the organisms in their appropriate regions
        self.refill(organisms);

        // current regions are greater than or equal to the allowed regions;
        // refill and return
        if self.regions.len() >= self.target_regions {
            debug!(
                "regions at target,  {} > {}",
                self.regions.len(),
                self.target_regions
            );
            return AdjustRegionsResult::ExpansionNotNecessary;
        }

        // otherwise we have not got enough regions
        // we need to divide a dimension.
        // we need to work out what is the best dimension to divide based on the distribution within the most populous region.
        // we are essentially using the most populous regions as a sample for the whole population

        // Determine the most diverse dimension in the most populous region
        let most_diverse_dimension = self.get_most_common_key().and_then(|key| {
            trace!("analyzing most populous region with key: {:?}", key);
            self.get_most_diverse_dimension(&key)
        });

        if let Some(most_diverse_dimension) = most_diverse_dimension {
            debug!("expanding dimension {}", most_diverse_dimension);

            // divide the most diverse dimension
            dimensions.divide_next_dimension(most_diverse_dimension);

            trace!("most diverse dimension {}", most_diverse_dimension);
            trace!("dimensions {:?}", dimensions);

            AdjustRegionsResult::DimensionExpanded {
                dimension_index: most_diverse_dimension,
            }
        } else {
            // get_most_diverse_dimension returns None if there is no variation in any dimensions
            // in this case no dimension divisions are necessary
            warn!(
                "no variation in data in most diverse dimension there is probably not point in continuing"
            );
            AdjustRegionsResult::AtResolutionLimit
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::regions::adjust_regions::AdjustRegionsResult;
    use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};
    use std::ops::RangeInclusive;

    // helper: 7 system parameters default values
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

    fn setup(target_regions: usize, bounds: Vec<RangeInclusive<f64>>) -> (Regions, Dimensions) {
        let gc = GlobalConstants::new(100, target_regions);
        let regions = Regions::new(&gc);
        let dimensions = Dimensions::new(&bounds);
        (regions, dimensions)
    }

    #[test]
    fn given_target_regions_already_reached_when_handle_successful_update_then_returns_expansion_not_necessary()
     {
        let (mut regions, mut dims) = setup(1, vec![0.0..=1.0]);
        let mut organisms = organisms_from_problem_values(vec![vec![0.5]]);

        // First update region keys to place organisms in regions
        let _ = organisms.update_all_region_keys(&dims, None);

        let result = regions.adjust_regions(&mut organisms, &mut dims);
        assert!(matches!(result, AdjustRegionsResult::ExpansionNotNecessary));
    }

    #[test]
    fn given_variance_when_handle_successful_update_then_returns_dimension_index() {
        let (mut regions, mut dims) = setup(10, vec![0.0..=1.0]);
        // Two organisms in same initial region with variation
        let mut organisms = organisms_from_problem_values(vec![vec![0.05], vec![0.06]]);
        let _ = organisms.update_all_region_keys(&dims, None);
        let result = regions.adjust_regions(&mut organisms, &mut dims);
        assert!(matches!(
            result,
            AdjustRegionsResult::DimensionExpanded { dimension_index: 0 }
        ));
    }

    #[test]
    fn given_no_variance_when_handle_successful_update_then_returns_at_resolution_limit() {
        let (mut regions, mut dims) = setup(10, vec![0.0..=1.0]);
        let mut organisms = organisms_from_problem_values(vec![vec![0.5], vec![0.5]]);
        // Assign region keys to organisms before calling adjust_regions
        let _ = organisms.update_all_region_keys(&dims, None);
        let result = regions.adjust_regions(&mut organisms, &mut dims);
        assert!(matches!(result, AdjustRegionsResult::AtResolutionLimit));
    }

    #[test]
    fn given_zero_dimensions_when_handle_successful_update_then_returns_at_resolution_limit() {
        let (mut regions, mut dims) = setup(10, vec![]);
        let mut organisms = organisms_from_problem_values(vec![vec![]]);
        // Assign region keys to organisms before calling adjust_regions
        let _ = organisms.update_all_region_keys(&dims, None);
        let result = regions.adjust_regions(&mut organisms, &mut dims);
        assert!(matches!(result, AdjustRegionsResult::AtResolutionLimit));
    }
}
