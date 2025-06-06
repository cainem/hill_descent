// src/locus/mod.rs
pub mod locus_adjustment;
pub mod mutate; // Declare the new mutate module
pub mod new_random_locus;
pub mod new_random_locus_adjustment;

use self::locus_adjustment::LocusAdjustment;
use crate::parameters::parameter::Parameter; // LocusAdjustment for struct

#[derive(Debug, Clone, PartialEq)]
pub struct Locus {
    pub value: Parameter, // Represents LocusValue
    pub adjustment: LocusAdjustment,
    pub apply_adjustment_flag: bool,
}

impl Locus {
    /// Creates a new Locus.
    pub fn new(value: Parameter, adjustment: LocusAdjustment, apply_adjustment_flag: bool) -> Self {
        Self {
            value,
            adjustment,
            apply_adjustment_flag,
        }
    }

    /// Returns a reference to the LocusValue (Parameter).
    pub fn value(&self) -> &Parameter {
        &self.value
    }

    /// Returns a reference to the LocusAdjustment.
    pub fn adjustment(&self) -> &LocusAdjustment {
        &self.adjustment
    }

    /// Returns the ApplyAdjustmentFlag.
    pub fn apply_adjustment_flag(&self) -> bool {
        self.apply_adjustment_flag
    }

    // The 'mutate' method has been moved to src/locus/mutate.rs
    // The 'new_random' method has been moved to src/locus/new_random_locus.rs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus::locus_adjustment::DirectionOfTravel;
    use crate::parameters::parameter::Parameter; // Used by create_test_parameter, create_test_adjustment // Brings Locus, LocusAdjustment, DirectionOfTravel etc. into scope
    // StdRng, SeedableRng are no longer needed for tests in this file.

    // Helper function to create a LocusAdjustment for tests
    fn create_test_adjustment(
        value: f64,
        direction: DirectionOfTravel,
        flag: bool,
    ) -> LocusAdjustment {
        LocusAdjustment::new(Parameter::new(value), direction, flag)
    }

    // Helper function to create a Parameter for tests
    fn create_test_parameter(value: f64) -> Parameter {
        Parameter::new(value)
    }

    // create_test_locus has been moved to src/locus/mutate.rs tests

    #[test]
    fn given_valid_params_when_new_then_locus_fields_are_set_correctly() {
        let param_val = create_test_parameter(10.5);
        let adj = create_test_adjustment(1.0, DirectionOfTravel::Add, false);
        let flag = true;

        let locus = Locus::new(param_val.clone(), adj.clone(), flag);

        assert_eq!(locus.value(), &param_val);
        assert_eq!(locus.adjustment(), &adj);
        assert_eq!(locus.apply_adjustment_flag(), flag);
    }

    #[test]
    fn given_locus_when_value_called_then_returns_correct_value() {
        let param_val = create_test_parameter(-5.0);
        let adj = create_test_adjustment(0.5, DirectionOfTravel::Subtract, true);
        let flag = false;
        let locus = Locus::new(param_val.clone(), adj.clone(), flag);

        assert_eq!(locus.value(), &param_val);
    }

    #[test]
    fn given_locus_when_adjustment_called_then_returns_correct_adjustment() {
        let param_val = create_test_parameter(20.0);
        let adj = create_test_adjustment(2.0, DirectionOfTravel::Add, false);
        let flag = true;
        let locus = Locus::new(param_val.clone(), adj.clone(), flag);

        assert_eq!(locus.adjustment(), &adj);
    }

    #[test]
    fn given_locus_when_apply_adjustment_flag_called_then_returns_correct_flag() {
        let param_val = create_test_parameter(0.0);
        let adj = create_test_adjustment(0.0, DirectionOfTravel::Subtract, false);

        let flag_true = true;
        let locus_true = Locus::new(param_val.clone(), adj.clone(), flag_true);
        assert_eq!(locus_true.apply_adjustment_flag(), flag_true);

        let flag_false = false;
        let locus_false = Locus::new(param_val.clone(), adj.clone(), flag_false);
        assert_eq!(locus_false.apply_adjustment_flag(), flag_false);
    }
}
