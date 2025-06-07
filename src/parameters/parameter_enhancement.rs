use std::ops::RangeInclusive;

/// Enhances a slice of parameter bounds by prepending system-specific parameter bounds.
///
/// The system parameter bounds include those for mutation probabilities (m1-m5), maximum organism age,
/// and the number of crossover points. These are added to the beginning of the
/// supplied slice of bounds.
///
/// # Arguments
///
/// * `existing_parameter_bounds`: A slice of `RangeInclusive<f64>` to which system parameter bounds will be prepended.
///
/// # Returns
///
/// A new `Vec<RangeInclusive<f64>>` containing the system parameter bounds followed by the `existing_parameter_bounds`.
pub fn enhance_parameters(
    existing_parameter_bounds: &[RangeInclusive<f64>],
) -> Vec<RangeInclusive<f64>> {
    let mut system_parameter_bounds = vec![
        // m1: Probability of ApplyAdjustmentFlag mutating from False to True
        0.0..=1.0, // m1_prob_false_to_true
        // m2: Probability of ApplyAdjustmentFlag mutating from True to False
        0.0..=1.0, // m2_prob_true_to_false
        // m3: Probability of LocusAdjustment.DoublingOrHalvingFlag mutating
        0.0..=1.0, // m3_prob_adj_double_halve_flag
        // m4: Probability of LocusAdjustment.DirectionOfTravel mutating
        0.0..=1.0, // m4_prob_adj_direction_flag
        // m5: Probability of LocusValue mutating
        0.0..=1.0, // m5_prob_locus_value_mutation
        // max_age: Maximum age of an organism
        10.0..=1000.0, // max_age
        // crossover_points: Number of crossover points for sexual reproduction
        1.0..=10.0, // crossover_points
    ];

    // Prepend system parameter bounds to the existing ones
    system_parameter_bounds.extend_from_slice(existing_parameter_bounds);
    system_parameter_bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::RangeInclusive;

    #[test]
    fn given_empty_slice_when_enhance_parameters_called_then_returns_only_system_parameter_bounds()
    {
        let bounds: [RangeInclusive<f64>; 0] = [];
        let enhanced_bounds = enhance_parameters(&bounds);
        assert_eq!(enhanced_bounds.len(), 7);

        // Check m1 bounds
        assert_eq!(enhanced_bounds[0], 0.0..=1.0);
        // Check m2 bounds
        assert_eq!(enhanced_bounds[1], 0.0..=1.0);
        // Check m3 bounds
        assert_eq!(enhanced_bounds[2], 0.0..=1.0);
        // Check m4 bounds
        assert_eq!(enhanced_bounds[3], 0.0..=1.0);
        // Check m5 bounds
        assert_eq!(enhanced_bounds[4], 0.0..=1.0);
        // Check max_age bounds
        assert_eq!(enhanced_bounds[5], 10.0..=1000.0);
        // Check crossover_points bounds
        assert_eq!(enhanced_bounds[6], 1.0..=10.0);
    }

    #[test]
    fn given_non_empty_slice_when_enhance_parameters_called_then_prepends_system_parameter_bounds()
    {
        let initial_bounds = vec![
            0.0..=100.0,  // Custom param 1 bounds
            -50.0..=50.0, // Custom param 2 bounds
        ];

        let enhanced_bounds = enhance_parameters(&initial_bounds);
        assert_eq!(enhanced_bounds.len(), 7 + 2);

        // Check system parameter bounds are at the beginning
        assert_eq!(enhanced_bounds[0], 0.0..=1.0); // m1_prob_false_to_true
        assert_eq!(enhanced_bounds[6], 1.0..=10.0); // crossover_points

        // Check original parameter bounds are at the end and unchanged
        assert_eq!(enhanced_bounds[7], 0.0..=100.0);
        assert_eq!(enhanced_bounds[8], -50.0..=50.0);
    }

    #[test]
    fn given_bounds_when_enhanced_then_system_bounds_are_correct() {
        let bounds: [RangeInclusive<f64>; 0] = []; // Input doesn't matter for this test
        let enhanced_bounds = enhance_parameters(&bounds);

        // m1
        assert_eq!(enhanced_bounds[0].start(), &0.0);
        assert_eq!(enhanced_bounds[0].end(), &1.0);

        // m2
        assert_eq!(enhanced_bounds[1].start(), &0.0);
        assert_eq!(enhanced_bounds[1].end(), &1.0);

        // m3
        assert_eq!(enhanced_bounds[2].start(), &0.0);
        assert_eq!(enhanced_bounds[2].end(), &1.0);

        // m4
        assert_eq!(enhanced_bounds[3].start(), &0.0);
        assert_eq!(enhanced_bounds[3].end(), &1.0);

        // m5
        assert_eq!(enhanced_bounds[4].start(), &0.0);
        assert_eq!(enhanced_bounds[4].end(), &1.0);

        // max_age
        assert_eq!(enhanced_bounds[5].start(), &10.0);
        assert_eq!(enhanced_bounds[5].end(), &1000.0);

        // crossover_points
        assert_eq!(enhanced_bounds[6].start(), &1.0);
        assert_eq!(enhanced_bounds[6].end(), &10.0);
    }
}
