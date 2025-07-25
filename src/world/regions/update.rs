use crate::world::organisms::organism::update_region_key::OrganismUpdateRegionKeyResult;
use crate::world::{dimensions::Dimensions, organisms::Organisms};

// update_carrying_capacities function has been moved to its own file.

impl super::Regions {
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, organisms, dimensions))
    )]
    /// Update region state based on the current collection of `organisms` and the
    /// mutable spatial `dimensions`.
    ///
    /// Algorithm overview:
    /// 1. Repeatedly attempt to update every organism’s region key.
    ///    * If an organism falls **outside** the current bounds we expand the
    ///      offending dimension via `handle_out_of_bounds` and restart the loop.
    /// 2. Once all organism keys are valid, we call `handle_successful_update`.
    ///    * This may further **divide** a dimension (to increase spatial
    ///      resolution) and returns `Some(dim_idx)` to signal which dimension
    ///      changed.  When it returns `None`, the space is considered stable and
    ///      we exit the loop.
    /// 3. Finally we recompute per-region statistics:
    ///    * `update_all_region_min_scores` – lowest fitness observed in each region.
    ///    * `update_carrying_capacities`   – ecological capacity derived from
    ///      these scores and global constants.
    ///
    /// This three-step process guarantees that every organism has a valid region
    /// key and that the spatial partitioning respects both out-of-bounds cases
    /// (expansion) and over-population within a region (division) until the
    /// target number of regions is reached or no further meaningful split is
    /// possible.
    pub fn update(&mut self, organisms: &mut Organisms, dimensions: &mut Dimensions) {
        use crate::world::regions::adjust_regions::AdjustRegionsResult;

        let mut changed_dimension: Option<usize> = None;

        // --- main update loop ------------------------------------------------
        loop {
            if let OrganismUpdateRegionKeyResult::OutOfBounds(dimension_index) =
                organisms.update_all_region_keys(dimensions, changed_dimension)
            {
                self.handle_out_of_bounds(dimensions, dimension_index);
                changed_dimension = Some(dimension_index);
                continue;
            }

            let result = self.adjust_regions(organisms, dimensions);
            match result {
                AdjustRegionsResult::DimensionExpanded { dimension_index } => {
                    changed_dimension = Some(dimension_index);
                    continue;
                }
                AdjustRegionsResult::ExpansionNotNecessary
                | AdjustRegionsResult::AtResolutionLimit => {
                    break;
                }
            }
        }
        // Update min scores for regions first, then carrying capacities
        // --- post-processing -------------------------------------------------
        // All region keys are now valid and space is stable – recalculate
        // dependent metrics before returning.
        self.update_all_region_min_scores(organisms);
        self.update_carrying_capacities();
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::{dimensions::Dimensions, organisms::Organisms, regions::Regions};

    use std::ops::RangeInclusive;

    // --- helper fns ---------------------------------------------------------
    fn default_system_parameters() -> Vec<f64> {
        vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]
    }

    fn phen(problem_values: &[f64]) -> Phenotype {
        let mut expr = default_system_parameters();
        expr.extend_from_slice(problem_values);
        Phenotype::new_for_test(expr)
    }

    fn orgs(p_values: Vec<Vec<f64>>) -> Organisms {
        Organisms::new_from_phenotypes(p_values.iter().map(|pv| phen(pv)).collect())
    }

    fn regions_and_dims(
        target: usize,
        pop: usize,
        bounds: Vec<RangeInclusive<f64>>,
    ) -> (Regions, Dimensions, GlobalConstants) {
        let gc = GlobalConstants::new(pop, target);
        (Regions::new(&gc), Dimensions::new(&bounds), gc)
    }

    // --- tests --------------------------------------------------------------

    #[test]
    fn given_no_organisms_when_update_then_no_regions() {
        let (mut regions, mut dims, _) = regions_and_dims(4, 10, vec![0.0..=1.0]);
        let mut organisms = orgs(vec![]);
        regions.update(&mut organisms, &mut dims);
        assert!(regions.regions().is_empty());
    }

    #[test]
    fn given_one_organism_fits_bounds_when_update_then_single_region_no_extra_division() {
        let (mut regions, mut dims, _) = regions_and_dims(4, 10, vec![0.0..=1.0, 0.0..=1.0]);
        let mut organisms = orgs(vec![vec![0.5, 0.5]]);
        // precondition: zero doublings per dim from Dimensions::new (1 interval each)
        assert_eq!(dims.get_total_possible_regions(), 1);
        regions.update(&mut organisms, &mut dims);
        assert_eq!(regions.regions().len(), 1);
        assert_eq!(dims.get_total_possible_regions(), 1);
    }

    #[test]
    fn given_organism_out_of_bounds_when_update_then_dimension_expands() {
        let (mut regions, mut dims, _) = regions_and_dims(4, 10, vec![0.0..=1.0, 0.0..=1.0]);
        let mut organisms = orgs(vec![vec![1.5, 0.5]]);
        regions.update(&mut organisms, &mut dims);
        let range0 = dims.get_dimension(0).range();
        assert_eq!(*range0.start(), -0.5);
        assert_eq!(*range0.end(), 1.5);
        assert_eq!(dims.get_dimension(1).range().clone(), 0.0..=1.0);
        assert_eq!(regions.regions().len(), 1);
    }

    #[test]
    fn given_two_distant_organisms_when_update_then_space_divides() {
        let (mut regions, mut dims, _) = regions_and_dims(10, 16, vec![0.0..=1.0, 0.0..=1.0]);
        let mut organisms = orgs(vec![vec![0.2, 0.2], vec![0.8, 0.8]]);
        regions.update(&mut organisms, &mut dims);
        assert_eq!(regions.regions().len(), 2);

        // The algorithm divides the most diverse dimension first
        // With equal variance, dimension 0 gets divided first
        assert_eq!(dims.get_dimension(0).number_of_doublings(), 1);
        assert_eq!(dims.get_dimension(1).number_of_doublings(), 0);
    }
}
