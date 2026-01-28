//! Dimension limit adjustment based on organism values.
//!
//! When dimension subdivision fails due to precision limits, this fallback
//! mechanism adjusts the dimension range based on actual organism values,
//! allowing further optimization progress.

use super::Dimensions;
use crate::NUM_SYSTEM_PARAMETERS;
use crate::world::World;

impl Dimensions {
    /// Adjusts the limits of a single dimension based on actual organism expressed values.
    ///
    /// This function examines the specified dimension and finds the minimum and maximum
    /// expressed values across all organisms for that dimension. The dimension's
    /// range is then adjusted to be 50% larger than needed to hold these values.
    ///
    /// # Arguments
    ///
    /// * `dimension_index` - The index of the dimension to adjust.
    /// * `world` - A reference to the World to access organisms.
    ///
    /// # Returns
    ///
    /// * `true` if the dimension's range shrunk (total range decreased).
    /// * `false` if the range stayed the same or grew.
    ///
    /// # Example
    ///
    /// If organisms have values ranging from 10.0 to 20.0 for the specified dimension,
    /// the new range will be centered around 15.0 with a span of 15.0
    /// (20.0 - 10.0 = 10.0, plus 50% = 15.0 span).
    /// The new range would be approximately [7.5, 22.5].
    pub fn adjust_limits(&mut self, dimension_index: usize, world: &World) -> bool {
        // Check bounds and early returns
        if dimension_index >= self.dimensions.len() || world.organisms.is_empty() {
            return false;
        }

        // Calculate the original range of the dimension
        let original_range = {
            let dimension = &self.dimensions[dimension_index];
            let range = dimension.range();
            let start = *range.start();
            let end = *range.end();

            // Handle infinite ranges sensibly
            if start.is_infinite() || end.is_infinite() {
                f64::INFINITY // Treat any infinite range as having infinite span
            } else {
                end - start
            }
        };

        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;

        // Find min and max values for the specified dimension across all organisms
        // Organism uses interior mutability so no locks needed
        for org in world.organisms.values() {
            let expressed_values = org.phenotype().expressed_values();

            // Calculate the actual index in expressed_values (system parameters + dimension index)
            let expressed_index = NUM_SYSTEM_PARAMETERS + dimension_index;

            if expressed_values.len() <= expressed_index {
                continue;
            }

            let value = expressed_values[expressed_index];

            // Skip non-finite values
            if !value.is_finite() {
                continue;
            }

            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }

        // Skip if no valid values found for this dimension
        if min_value == f64::INFINITY || max_value == f64::NEG_INFINITY {
            return false;
        }

        // Calculate new range parameters
        let midpoint = (min_value + max_value) / 2.0;
        let original_span = max_value - min_value;
        let span = if original_span == 0.0 {
            1.0 // Default span for single value
        } else {
            original_span * 1.5
        };

        let new_start = midpoint - span / 2.0;
        let new_end = midpoint + span / 2.0;

        // Apply the new range to the dimension
        let dimension = &mut self.dimensions[dimension_index];
        dimension.set_range(new_start..=new_end);

        // Calculate the new range and compare with original
        let new_range = new_end - new_start;

        // Handle infinite range comparisons sensibly
        if original_range.is_infinite() {
            // If original range was infinite, any finite new range is a shrinkage
            new_range.is_finite()
        } else {
            // Normal comparison for finite ranges
            new_range < original_range
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::GlobalConstants;
    use crate::world::World;
    use crate::world::single_valued_function::SingleValuedFunction;
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SimpleFunction;

    impl SingleValuedFunction for SimpleFunction {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_empty_organisms_when_adjusting_limits_then_returns_false() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0];
        let constants = GlobalConstants::new_with_seed(10, 10, 42);
        let world = World::new(&bounds, constants, Box::new(SimpleFunction));

        // World with no epoch run has empty regions and organisms
        let mut dimensions = (*world.dimensions).clone();
        let result = dimensions.adjust_limits(0, &world);

        // Empty organisms, should return false
        assert!(!result);
    }

    #[test]
    fn given_invalid_dimension_index_when_adjusting_limits_then_returns_false() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 42);
        let mut world = World::new(&bounds, constants, Box::new(SimpleFunction));

        // Run one epoch to populate organisms
        world.process_epoch_all(0);

        let mut dimensions = (*world.dimensions).clone();
        let result = dimensions.adjust_limits(99, &world); // Index 99 doesn't exist

        assert!(!result);
    }

    #[test]
    fn given_organisms_with_narrow_spread_when_adjusting_limits_then_range_adjusts_to_organism_span()
     {
        // Start with a very wide range
        let bounds: Vec<RangeInclusive<f64>> = vec![-100.0..=100.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 42);
        let mut world = World::new(&bounds, constants, Box::new(SimpleFunction));

        // Run one epoch to populate organisms
        world.process_epoch_all(0);

        // Find actual organism min/max for dimension 0
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        for org in world.organisms.values() {
            let expressed = org.phenotype().expressed_values();
            if expressed.len() > crate::NUM_SYSTEM_PARAMETERS {
                let val = expressed[crate::NUM_SYSTEM_PARAMETERS];
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        // Adjust limits
        let mut dimensions = (*world.dimensions).clone();
        let result = dimensions.adjust_limits(0, &world);

        // Result depends on whether the organism spread is smaller than original
        // The function should set range to 1.5x the organism spread
        let new_start = *dimensions.get_dimensions()[0].range().start();
        let new_end = *dimensions.get_dimensions()[0].range().end();
        let new_span = new_end - new_start;

        let organism_span = max_val - min_val;
        let expected_span = if organism_span == 0.0 {
            1.0
        } else {
            organism_span * 1.5
        };

        // The new span should be 1.5x the organism spread
        assert!(
            (new_span - expected_span).abs() < 0.001,
            "Expected span {} (1.5x organism span {}), got {}. result={}",
            expected_span,
            organism_span,
            new_span,
            result
        );
    }

    #[test]
    fn given_organisms_when_adjusting_limits_then_range_contains_150_percent_of_organism_spread() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-50.0..=50.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 42);
        let mut world = World::new(&bounds, constants, Box::new(SimpleFunction));

        // Run one epoch to populate organisms
        world.process_epoch_all(0);

        // Find actual organism min/max for dimension 0
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        for org in world.organisms.values() {
            let expressed = org.phenotype().expressed_values();
            if expressed.len() > crate::NUM_SYSTEM_PARAMETERS {
                let val = expressed[crate::NUM_SYSTEM_PARAMETERS];
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        // Adjust limits
        let mut dimensions = (*world.dimensions).clone();
        dimensions.adjust_limits(0, &world);

        let new_start = *dimensions.get_dimensions()[0].range().start();
        let new_end = *dimensions.get_dimensions()[0].range().end();

        // New range should contain all organisms with 50% padding
        assert!(
            new_start <= min_val,
            "New start {} should be <= min organism value {}",
            new_start,
            min_val
        );
        assert!(
            new_end >= max_val,
            "New end {} should be >= max organism value {}",
            new_end,
            max_val
        );

        // The span should be 1.5x the organism spread
        let organism_span = max_val - min_val;
        let new_span = new_end - new_start;
        let expected_span = if organism_span == 0.0 {
            1.0
        } else {
            organism_span * 1.5
        };

        assert!(
            (new_span - expected_span).abs() < 0.001,
            "Expected span {} but got {}",
            expected_span,
            new_span
        );
    }
}
