use super::Dimensions;
use crate::world::organisms::Organisms;

impl Dimensions {
    /// Adjusts dimension limits based on actual organism expressed values.
    ///
    /// This function examines each dimension and finds the minimum and maximum
    /// expressed values across all organisms for that dimension. Each dimension's
    /// range is then adjusted to be 50% larger than needed to hold these values.
    ///
    /// # Arguments
    ///
    /// * `organisms` - A reference to the collection of organisms to analyze.
    ///
    /// # Example
    ///
    /// If organisms have values ranging from 10.0 to 20.0 for dimension 0,
    /// the new range will be centered around 15.0 with a span of 15.0
    /// (20.0 - 10.0 = 10.0, plus 50% = 15.0 span).
    /// The new range would be approximately [7.5, 22.5].
    #[allow(dead_code)]
    pub fn adjust_dimension_limits(&mut self, organisms: &Organisms) {
        let dimensions_vec = self.get_dimensions();

        if organisms.is_empty() || dimensions_vec.is_empty() {
            return;
        }

        let num_dimensions = dimensions_vec.len();
        let mut min_values = vec![f64::INFINITY; num_dimensions];
        let mut max_values = vec![f64::NEG_INFINITY; num_dimensions];

        // Find min and max values for each dimension across all organisms
        for organism in organisms.iter() {
            let phenotype = organism.phenotype();
            let expressed_values = phenotype.expressed_values();

            // Debug assert that we have enough expressed values for all dimensions
            // Note: expressed_values includes system parameters (first 7) + spatial dimensions
            debug_assert!(
                expressed_values.len() >= num_dimensions,
                "Organism expressed values length ({}) must be at least the number of dimensions ({})",
                expressed_values.len(),
                num_dimensions
            );

            for i in 0..num_dimensions {
                let value = expressed_values[i];

                // Skip NaN values
                if value.is_nan() {
                    continue;
                }

                min_values[i] = min_values[i].min(value);
                max_values[i] = max_values[i].max(value);
            }
        }

        // Adjust each dimension's range to be 50% larger than needed
        for i in 0..num_dimensions {
            let min_val = min_values[i];
            let max_val = max_values[i];

            // Skip if no valid values found for this dimension
            if min_val == f64::INFINITY || max_val == f64::NEG_INFINITY {
                continue;
            }

            let midpoint = (min_val + max_val) / 2.0;
            let original_span = max_val - min_val;
            let span = if original_span == 0.0 {
                1.0 // Default span for single value
            } else {
                original_span * 1.5
            };

            let new_start = midpoint - span / 2.0;
            let new_end = midpoint + span / 2.0;

            // Access the dimension directly to set its range
            let dimension = self.get_dimension_mut(i);
            dimension.set_range(new_start..=new_end);
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
        // Ensure we have at least 7 values for NUM_SYSTEM_PARAMETERS
        let mut full_values = expressed_values;
        while full_values.len() < 7 {
            full_values.push(0.0); // Pad with zeros
        }
        let phenotype = Phenotype::new_for_test(full_values);
        Rc::new(Organism::new(Rc::new(phenotype), 0))
    }

    #[test]
    fn given_empty_organisms_when_adjusting_limits_then_no_changes() {
        let mut dimensions = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=10.0, 1),
            Dimension::new(0.0..=10.0, 1),
        ]);
        let organisms = Organisms::new_empty();

        let original_ranges: Vec<_> = dimensions
            .get_dimensions()
            .iter()
            .map(|d| (*d.range().start(), *d.range().end()))
            .collect();

        dimensions.adjust_dimension_limits(&organisms);

        let new_ranges: Vec<_> = dimensions
            .get_dimensions()
            .iter()
            .map(|d| (*d.range().start(), *d.range().end()))
            .collect();

        assert_eq!(original_ranges, new_ranges);
    }

    #[test]
    fn given_single_organism_when_adjusting_limits_then_range_expanded_correctly() {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(0.0..=10.0, 1)]);

        let organism = create_test_organism(vec![5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let organisms = Organisms::new_from_organisms(vec![(*organism).clone()]);

        dimensions.adjust_dimension_limits(&organisms);

        let new_range = dimensions.get_dimensions()[0].range();
        // With single value 5.0, min=max=5.0, span=0.0, new_span=1.0 (default)
        // Range becomes [4.5, 5.5]
        assert!((*new_range.start() - 4.5).abs() < 0.001);
        assert!((*new_range.end() - 5.5).abs() < 0.001);
    }

    #[test]
    fn given_multiple_organisms_when_adjusting_limits_then_range_expanded_correctly() {
        let mut dimensions = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=10.0, 1),
            Dimension::new(0.0..=10.0, 1),
        ]);

        let organism1 = create_test_organism(vec![2.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let organism2 = create_test_organism(vec![8.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let organism3 = create_test_organism(vec![5.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        let organisms = Organisms::new_from_organisms(vec![
            (*organism1).clone(),
            (*organism2).clone(),
            (*organism3).clone(),
        ]);

        dimensions.adjust_dimension_limits(&organisms);

        // Dimension 0: values 2.0, 8.0, 5.0 → min=2.0, max=8.0, span=6.0, new_span=9.0
        // midpoint=5.0, new range=[0.5, 9.5]
        let dim0_range = dimensions.get_dimensions()[0].range();
        let expected_start = 0.5;
        let expected_end = 9.5;
        assert!((*dim0_range.start() - expected_start).abs() < 0.001);
        assert!((*dim0_range.end() - expected_end).abs() < 0.001);

        // Dimension 1: values 8.0, 2.0, 5.0 → min=2.0, max=8.0, same calculation
        let dim1_range = dimensions.get_dimensions()[1].range();
        assert!((*dim1_range.start() - expected_start).abs() < 0.001);
        assert!((*dim1_range.end() - expected_end).abs() < 0.001);
    }

    #[test]
    fn given_negative_values_when_adjusting_limits_then_range_expanded_correctly() {
        let mut dimensions = Dimensions::new_for_test(vec![Dimension::new(-10.0..=10.0, 1)]);

        let organism1 = create_test_organism(vec![-5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let organism2 = create_test_organism(vec![3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        let organisms =
            Organisms::new_from_organisms(vec![(*organism1).clone(), (*organism2).clone()]);

        dimensions.adjust_dimension_limits(&organisms);

        // Values: -5.0, 3.0 → min=-5.0, max=3.0, span=8.0, new_span=12.0
        // midpoint=-1.0, new range=[-7.0, 5.0]
        let new_range = dimensions.get_dimensions()[0].range();
        assert!((*new_range.start() + 7.0).abs() < 0.001);
        assert!((*new_range.end() - 5.0).abs() < 0.001);
    }

    #[test]
    fn given_partial_dimension_data_when_adjusting_limits_then_only_valid_dimensions_updated() {
        let mut dimensions = Dimensions::new_for_test(vec![
            Dimension::new(0.0..=10.0, 1),
            Dimension::new(100.0..=200.0, 1), // Very different range
        ]);

        // Organism has data that affects first dimension only
        let organism = create_test_organism(vec![5.0, 150.0]);
        let organisms = Organisms::new_from_organisms(vec![(*organism).clone()]);

        dimensions.adjust_dimension_limits(&organisms);

        // First dimension should be updated
        let dim0_range = dimensions.get_dimensions()[0].range();
        let expected_start = 5.0 - 0.5; // midpoint 5.0, span 1.0, new range [4.5, 5.5]
        let expected_end = 5.0 + 0.5;
        assert!((*dim0_range.start() - expected_start).abs() < 0.001);
        assert!((*dim0_range.end() - expected_end).abs() < 0.001);

        // Second dimension should be updated based on 150.0
        let dim1_range = dimensions.get_dimensions()[1].range();
        let expected_start = 150.0 - 0.5; // midpoint 150.0, span 1.0, new range [149.5, 150.5]
        let expected_end = 150.0 + 0.5;
        assert!((*dim1_range.start() - expected_start).abs() < 0.001);
        assert!((*dim1_range.end() - expected_end).abs() < 0.001);
    }
}
