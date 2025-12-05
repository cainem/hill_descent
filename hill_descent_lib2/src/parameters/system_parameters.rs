/// System-wide evolvable parameters for the GA (e.g., mutation rates, max age, crossover points).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemParameters {
    m1: f64,               // Probability of ApplyAdjustmentFlag mutating from False to True
    m2: f64,               // Probability of ApplyAdjustmentFlag mutating from True to False
    m3: f64,               // Probability of LocusAdjustment.DoublingOrHalvingFlag mutating
    m4: f64,               // Probability of LocusAdjustment.DirectionOfTravel mutating
    m5: f64,               // Probability of LocusValue mutating
    max_age: f64,          // Maximum age of an organism
    crossover_points: f64, // Number of crossover points for sexual reproduction
}

impl SystemParameters {
    /// Constructs SystemParameters from a slice representing the 7 system parameters.
    ///
    /// The expected order and meaning of the `values` slice elements are:
    /// 1. `m1_prob_false_to_true`: Probability of ApplyAdjustmentFlag mutating from False to True.
    /// 2. `m2_prob_true_to_false`: Probability of ApplyAdjustmentFlag mutating from True to False.
    /// 3. `m3_prob_adj_double_halve_flag`: Probability of LocusAdjustment.DoublingOrHalvingFlag mutating.
    /// 4. `m4_prob_adj_direction_flag`: Probability of LocusAdjustment.DirectionOfTravel mutating.
    /// 5. `m5_prob_locus_value_mutation`: Probability of LocusValue mutating.
    /// 6. `max_age`: Maximum age of an organism.
    /// 7. `crossover_points`: Number of crossover points for sexual reproduction.
    ///
    /// This order must strictly match the order in which system parameters are prepended
    /// to the problem parameters when creating phenotypes.
    ///
    /// Panics if the provided slice does not contain exactly 7 elements.
    pub fn new(values: &[f64]) -> Self {
        if values.len() != 7 {
            panic!(
                "SystemParameters::new expects a slice with exactly 7 elements, got {}",
                values.len()
            );
        }
        Self {
            m1: values[0],
            m2: values[1],
            m3: values[2],
            m4: values[3],
            m5: values[4],
            max_age: values[5],
            crossover_points: values[6],
        }
    }

    /// Returns mutation probability m1 (ApplyAdjustmentFlag: False -> True).
    pub fn m1(&self) -> f64 {
        self.m1
    }
    /// Returns mutation probability m2 (ApplyAdjustmentFlag: True -> False).
    pub fn m2(&self) -> f64 {
        self.m2
    }
    /// Returns mutation probability m3 (LocusAdjustment.DoublingOrHalvingFlag).
    pub fn m3(&self) -> f64 {
        self.m3
    }
    /// Returns mutation probability m4 (LocusAdjustment.DirectionOfTravel).
    pub fn m4(&self) -> f64 {
        self.m4
    }
    /// Returns mutation probability m5 (LocusValue mutation).
    pub fn m5(&self) -> f64 {
        self.m5
    }
    /// Returns the maximum age for an organism.
    pub fn max_age(&self) -> f64 {
        self.max_age
    }
    /// Returns the number of crossover points for sexual reproduction.
    pub fn crossover_points(&self) -> f64 {
        self.crossover_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_correct_length_slice_when_new_then_all_fields_are_set_correctly() {
        let values = [0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0];
        let sp = SystemParameters::new(&values);
        assert_eq!(sp.m1(), 0.1, "m1 mismatch");
        assert_eq!(sp.m2(), 0.5, "m2 mismatch");
        assert_eq!(sp.m3(), 0.001, "m3 mismatch");
        assert_eq!(sp.m4(), 0.001, "m4 mismatch");
        assert_eq!(sp.m5(), 0.001, "m5 mismatch");
        assert_eq!(sp.max_age(), 100.0, "max_age mismatch");
        assert_eq!(sp.crossover_points(), 2.0, "crossover_points mismatch");
    }

    #[test]
    #[should_panic(
        expected = "SystemParameters::new expects a slice with exactly 7 elements, got 5"
    )]
    fn given_shorter_slice_when_new_then_panics() {
        let values = [0.1, 0.2, 0.3, 0.4, 0.5];
        SystemParameters::new(&values); // Should panic
    }

    #[test]
    #[should_panic(
        expected = "SystemParameters::new expects a slice with exactly 7 elements, got 0"
    )]
    fn given_empty_slice_when_new_then_panics() {
        let values: [f64; 0] = [];
        SystemParameters::new(&values); // Should panic
    }

    #[test]
    #[should_panic(
        expected = "SystemParameters::new expects a slice with exactly 7 elements, got 8"
    )]
    fn given_longer_slice_when_new_then_panics() {
        let values = [0.1, 0.2, 0.3, 0.4, 0.5, 6.0, 7.0, 8.0];
        SystemParameters::new(&values); // Should panic
    }

    #[test]
    fn given_default_when_called_then_all_fields_are_zero() {
        let sp = SystemParameters::default();
        assert_eq!(sp.m1(), 0.0, "Default m1 should be 0.0");
        assert_eq!(sp.m2(), 0.0, "Default m2 should be 0.0");
        assert_eq!(sp.m3(), 0.0, "Default m3 should be 0.0");
        assert_eq!(sp.m4(), 0.0, "Default m4 should be 0.0");
        assert_eq!(sp.m5(), 0.0, "Default m5 should be 0.0");
        assert_eq!(sp.max_age(), 0.0, "Default max_age should be 0.0");
        assert_eq!(
            sp.crossover_points(),
            0.0,
            "Default crossover_points should be 0.0"
        );
    }
}
