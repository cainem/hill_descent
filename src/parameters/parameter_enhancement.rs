use crate::parameters::parameter::Parameter;

/// Enhances a vector of parameters by prepending system-specific parameters.
///
/// The system parameters include mutation probabilities (m1-m5), maximum organism age,
/// and the number of crossover points. These are added to the beginning of the
/// supplied vector.
///
/// # Arguments
///
/// * `existing_parameters`: A `Vec<Parameter>` to which system parameters will be prepended.
///
/// # Returns
///
/// A new `Vec<Parameter>` containing the system parameters followed by the `existing_parameters`.
pub fn enhance_parameters(existing_parameters: Vec<Parameter>) -> Vec<Parameter> {
    let mut system_params = vec![
        // m1: Probability of ApplyAdjustmentFlag mutating from False to True
        Parameter::with_bounds(0.1, 0.0, 1.0), // m1_prob_false_to_true
        // m2: Probability of ApplyAdjustmentFlag mutating from True to False
        Parameter::with_bounds(0.5, 0.0, 1.0), // m2_prob_true_to_false
        // m3: Probability of LocusAdjustment.DoublingOrHalvingFlag mutating
        Parameter::with_bounds(0.001, 0.0, 1.0), // m3_prob_adj_double_halve_flag
        // m4: Probability of LocusAdjustment.DirectionOfTravel mutating
        Parameter::with_bounds(0.001, 0.0, 1.0), // m4_prob_adj_direction_flag
        // m5: Probability of LocusValue mutating
        Parameter::with_bounds(0.001, 0.0, 1.0), // m5_prob_locus_value_mutation
        // max_age: Maximum age of an organism
        Parameter::with_bounds(100.0, 10.0, 1000.0), // max_age
        // crossover_points: Number of crossover points for sexual reproduction
        Parameter::with_bounds(2.0, 1.0, 10.0), // crossover_points
    ];

    // Prepend system parameters to the existing ones
    system_params.extend(existing_parameters);
    system_params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::parameter::Parameter;

    #[test]
    fn given_empty_vector_when_enhance_parameters_called_then_returns_only_system_parameters() {
        let params = Vec::new();
        let enhanced_params = enhance_parameters(params);
        assert_eq!(enhanced_params.len(), 7);

        // Check m1
        assert_eq!(enhanced_params[0].get(), 0.1);
        // Check m2
        assert_eq!(enhanced_params[1].get(), 0.5);
        // Check m3
        assert_eq!(enhanced_params[2].get(), 0.001);
        // Check m4
        assert_eq!(enhanced_params[3].get(), 0.001);
        // Check m5
        assert_eq!(enhanced_params[4].get(), 0.001);
        // Check max_age
        assert_eq!(enhanced_params[5].get(), 100.0);
        // Check crossover_points
        assert_eq!(enhanced_params[6].get(), 2.0);
    }

    #[test]
    fn given_non_empty_vector_when_enhance_parameters_called_then_prepends_system_parameters() {
        let mut initial_params = Vec::new();
        let p1 = Parameter::new(10.0);
        let p2 = Parameter::with_bounds(20.0, 15.0, 25.0);
        initial_params.push(p1.clone());
        initial_params.push(p2.clone());

        let enhanced_params = enhance_parameters(initial_params);
        assert_eq!(enhanced_params.len(), 7 + 2);

        // Check system parameters are at the beginning
        assert_eq!(enhanced_params[0].get(), 0.1); // m1
        assert_eq!(enhanced_params[6].get(), 2.0); // crossover_points

        // Check original parameters are at the end and unchanged
        assert_eq!(enhanced_params[7].get(), p1.get());
        assert_eq!(*enhanced_params[7].bounds().start(), f64::MIN);
        assert_eq!(*enhanced_params[7].bounds().end(), f64::MAX);

        assert_eq!(enhanced_params[8].get(), p2.get());
        assert_eq!(*enhanced_params[8].bounds().start(), 15.0);
        assert_eq!(*enhanced_params[8].bounds().end(), 25.0);
    }

    #[test]
    fn given_parameters_when_enhanced_then_bounds_are_set_correctly_for_system_parameters() {
        let params = Vec::new();
        let enhanced_params = enhance_parameters(params);

        // m1
        let p_m1 = &enhanced_params[0];
        assert_eq!(*p_m1.bounds().start(), 0.0);
        assert_eq!(*p_m1.bounds().end(), 1.0);

        // m2
        let p_m2 = &enhanced_params[1];
        assert_eq!(*p_m2.bounds().start(), 0.0);
        assert_eq!(*p_m2.bounds().end(), 1.0);

        // m3
        let p_m3 = &enhanced_params[2];
        assert_eq!(*p_m3.bounds().start(), 0.0);
        assert_eq!(*p_m3.bounds().end(), 1.0);

        // m4
        let p_m4 = &enhanced_params[3];
        assert_eq!(*p_m4.bounds().start(), 0.0);
        assert_eq!(*p_m4.bounds().end(), 1.0);

        // m5
        let p_m5 = &enhanced_params[4];
        assert_eq!(*p_m5.bounds().start(), 0.0);
        assert_eq!(*p_m5.bounds().end(), 1.0);

        // max_age
        let p_max_age = &enhanced_params[5];
        assert_eq!(*p_max_age.bounds().start(), 10.0);
        assert_eq!(*p_max_age.bounds().end(), 1000.0);

        // crossover_points
        let p_crossover = &enhanced_params[6];
        assert_eq!(*p_crossover.bounds().start(), 1.0);
        assert_eq!(*p_crossover.bounds().end(), 10.0);
    }
}
