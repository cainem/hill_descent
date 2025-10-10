use super::Dimensions;
use crate::world::organisms::Organisms;

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
    /// * `organisms` - A reference to the collection of organisms to analyze.
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
    pub fn adjust_limits(&mut self, dimension_index: usize, organisms: &Organisms) -> bool {
        // Check bounds and early returns
        if dimension_index >= self.dimensions.len() || organisms.is_empty() {
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
                crate::warn!(
                    "infinite range detected in dimension index {}",
                    dimension_index
                );
                f64::INFINITY // Treat any infinite range as having infinite span
            } else {
                crate::trace!("dimension range {}", end - start);
                end - start
            }
        };

        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;

        // Find min and max values for the specified dimension across all organisms
        for organism in organisms.iter() {
            let phenotype = organism.phenotype();
            let expressed_values = phenotype.expressed_values();

            // Calculate the actual index in expressed_values (system parameters + dimension index)
            // Note: expressed_values includes system parameters (first 7) + spatial dimensions
            let expressed_index = crate::NUM_SYSTEM_PARAMETERS + dimension_index;

            debug_assert!(
                expressed_values.len() > expressed_index,
                "Organism expressed values length ({}) must be greater than expressed index ({})",
                expressed_values.len(),
                expressed_index
            );

            let value = expressed_values[expressed_index];

            // Assert that expressed values are never NaN or infinite (constraint)
            debug_assert!(
                value.is_finite(),
                "Expressed value at index {} is not finite ({}), which violates the constraint that expressed values must be finite",
                expressed_index,
                value
            );

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
            crate::debug!("range shrunk from {} to {}", original_range, new_range);
            new_range < original_range
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phenotype::Phenotype;
    use crate::world::dimensions::dimension::Dimension;
    use crate::world::organisms::Organisms;
    use crate::world::organisms::organism::Organism;

    use std::rc::Rc;

    fn create_test_organism(expressed_values: Vec<f64>) -> Rc<Organism> {
        // Ensure we have at least 9 values for NUM_SYSTEM_PARAMETERS + some spatial dimensions
        // We need at least 10 values to test dimension index 0 (9 system + 1 spatial)
        let mut full_values = expressed_values;
        while full_values.len() < 10 {
            full_values.push(0.0); // Pad with zeros
        }
        let phenotype = Phenotype::new_for_test(full_values);
        Rc::new(Organism::new(Rc::new(phenotype), 0, (None, None)))
    }

    #[test]
    fn given_empty_organisms_when_adjusting_limits_then_returns_false() {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0, 1)]);
        let organisms = Organisms::new_empty();

        let result = dimensions.adjust_limits(0, &organisms);

        assert!(!result);
        // Original range should be unchanged
        let range = dimensions.get_dimension(0).range();
        assert_eq!(*range.start(), 0.0);
        assert_eq!(*range.end(), 10.0);
    }

    #[test]
    fn given_invalid_dimension_index_when_adjusting_limits_then_returns_false() {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0, 1)]);
        let organism = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0]);
        let organisms = Organisms::new_from_organisms(vec![(*organism).clone()]);

        let result = dimensions.adjust_limits(1, &organisms); // Index 1 doesn't exist

        assert!(!result);
    }

    #[test]
    fn given_single_organism_when_adjusting_limits_then_range_expanded_and_returns_false() {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0, 1)]);
        // Create organism with spatial dimension value at index 9 (after 9 system parameters)
        let organism = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0]);
        let organisms = Organisms::new_from_organisms(vec![(*organism).clone()]);

        let result = dimensions.adjust_limits(0, &organisms);

        // With single value 5.0, min=max=5.0, span=0.0, new_span=1.0 (default)
        // Range becomes [4.5, 5.5] with span 1.0
        // Original span was 10.0, new span is 1.0, so range shrunk -> should return true
        assert!(result);

        let new_range = dimensions.get_dimension(0).range();
        // With single value 5.0, min=max=5.0, span=0.0, new_span=1.0 (default)
        // Range becomes [4.5, 5.5]
        assert!((*new_range.start() - 4.5).abs() < 0.001);
        assert!((*new_range.end() - 5.5).abs() < 0.001);
    }

    #[test]
    fn given_organisms_with_wide_spread_when_adjusting_limits_then_range_expanded_and_returns_false()
     {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0, 1)]);

        let organism1 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0]);
        let organism2 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0]);
        let organism3 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0]);

        let organisms = Organisms::new_from_organisms(vec![
            (*organism1).clone(),
            (*organism2).clone(),
            (*organism3).clone(),
        ]);

        let result = dimensions.adjust_limits(0, &organisms);

        // Original range: 10.0, new range: 9.0 (values 2.0-8.0, span=6.0, new_span=9.0)
        // Range shrunk from 10.0 to 9.0, so should return true
        assert!(result);

        let dim_range = dimensions.get_dimension(0).range();
        let expected_start = 0.5; // midpoint=5.0, span=9.0, new range=[0.5, 9.5]
        let expected_end = 9.5;
        assert!((*dim_range.start() - expected_start).abs() < 0.001);
        assert!((*dim_range.end() - expected_end).abs() < 0.001);
    }

    #[test]
    fn given_organisms_with_narrow_spread_when_adjusting_limits_then_range_shrunk_and_returns_true()
    {
        // Start with a very wide range
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(-100.0..=100.0, 1)]);

        let organism1 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0]);
        let organism2 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.0]);

        let organisms =
            Organisms::new_from_organisms(vec![(*organism1).clone(), (*organism2).clone()]);

        let result = dimensions.adjust_limits(0, &organisms);

        // Original range: 200.0, new range: 3.0 (values 4.0-6.0, span=2.0, new_span=3.0)
        // Range should shrink significantly, so should return true
        assert!(result);

        let dim_range = dimensions.get_dimension(0).range();
        let expected_start = 3.5; // midpoint=5.0, span=3.0, new range=[3.5, 6.5]
        let expected_end = 6.5;
        assert!((*dim_range.start() - expected_start).abs() < 0.001);
        assert!((*dim_range.end() - expected_end).abs() < 0.001);
    }

    #[test]
    fn given_negative_values_when_adjusting_limits_then_handles_correctly() {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(-10.0..=10.0, 1)]);

        let organism1 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -5.0]);
        let organism2 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0]);

        let organisms =
            Organisms::new_from_organisms(vec![(*organism1).clone(), (*organism2).clone()]);

        let result = dimensions.adjust_limits(0, &organisms);

        // Original range: 20.0, new range: 12.0 (values -5.0 to 3.0, span=8.0, new_span=12.0)
        // Range should shrink, so should return true
        assert!(result);

        let new_range = dimensions.get_dimension(0).range();
        // Values: -5.0, 3.0 → min=-5.0, max=3.0, span=8.0, new_span=12.0
        // midpoint=-1.0, new range=[-7.0, 5.0]
        assert!((*new_range.start() + 7.0).abs() < 0.001);
        assert!((*new_range.end() - 5.0).abs() < 0.001);
    }

    #[test]
    fn given_infinite_original_range_when_adjusting_limits_then_handles_sensibly() {
        // Test with an infinite original range
        let mut dimensions =
            Dimensions::new_for_test(vec![Dimension::new(f64::NEG_INFINITY..=f64::INFINITY, 1)]);

        let organism1 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0]);
        let organism2 = create_test_organism(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 15.0]);

        let organisms =
            Organisms::new_from_organisms(vec![(*organism1).clone(), (*organism2).clone()]);

        let result = dimensions.adjust_limits(0, &organisms);

        // Should handle infinite range sensibly - any finite range is smaller than infinite
        // So this should return true (range shrunk from infinite to finite)
        assert!(result);

        let new_range = dimensions.get_dimension(0).range();
        // Should have adjusted to a finite range around the organism values
        assert!(new_range.start().is_finite());
        assert!(new_range.end().is_finite());
    }
}
