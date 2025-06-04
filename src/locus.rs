// src/locus.rs
use crate::locus_adjustment::DirectionOfTravel;
use crate::locus_adjustment::LocusAdjustment;
use crate::parameter::Parameter;
use crate::system_parameters::SystemParameters;
use rand::Rng;

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

    /// Applies PDD mutation rules to this locus, returning a new one.
    pub fn mutate<R: Rng>(&self, rng: &mut R, sys: &SystemParameters) -> Self {
        let mut new_adj_val = self.adjustment.adjustment_value().clone();
        let mut new_direction = self.adjustment.direction_of_travel();
        let mut new_double_flag = self.adjustment.doubling_or_halving_flag();
        let mut new_apply_flag = self.apply_adjustment_flag();
        // Direction mutation (m4)
        if rng.gen_bool(sys.m4()) {
            new_direction = match new_direction {
                DirectionOfTravel::Add => DirectionOfTravel::Subtract,
                DirectionOfTravel::Subtract => DirectionOfTravel::Add,
            };
            new_double_flag = !new_double_flag;
        }
        // Doubling flag mutation (m3)
        if rng.gen_bool(sys.m3()) {
            new_double_flag = !new_double_flag;
        }
        // Adjustment value mutation (m5)
        if rng.gen_bool(sys.m5()) {
            if new_double_flag {
                new_adj_val.set(new_adj_val.get() * 2.0);
            } else {
                new_adj_val.set(new_adj_val.get() / 2.0);
            }
        }
        // Rebuild adjustment (checksum updated)
        let new_adjustment = LocusAdjustment::new(new_adj_val, new_direction, new_double_flag);
        // Apply flag mutation (m1/m2)
        if new_apply_flag {
            if rng.gen_bool(sys.m2()) {
                new_apply_flag = false;
            }
        } else if rng.gen_bool(sys.m1()) {
            new_apply_flag = true;
        }
        // Apply adjustment to value if flag is true
        let mut new_value = self.value.clone();
        if new_apply_flag {
            let sign = match new_adjustment.direction_of_travel() {
                DirectionOfTravel::Add => 1.0,
                DirectionOfTravel::Subtract => -1.0,
            };
            let delta = sign * new_adjustment.adjustment_value().get();
            new_value.set(new_value.get() + delta);
        }
        Locus::new(new_value, new_adjustment, new_apply_flag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameter::Parameter;
    use crate::system_parameters::SystemParameters;
    use rand::rngs::mock::StepRng;

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

    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

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

    #[test]
    fn mutate_no_mutation_returns_same() {
        let l = create_test_locus(1.5);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[]);
        let l2 = l.mutate(&mut rng, &sys);
        assert_eq!(l2, l);
    }

    #[test]
    fn mutate_with_full_probs_applies_flag_flip() {
        let l = create_test_locus(2.0);
        let mut rng = StepRng::new(u64::MAX, 0);
        let sys = SystemParameters::new(&[1.0, 1.0, 1.0, 1.0, 1.0]);
        let l2 = l.mutate(&mut rng, &sys);
        assert_ne!(l2, l);
        assert_eq!(l2.apply_adjustment_flag(), true);
        assert_eq!(
            l2.adjustment().direction_of_travel(),
            DirectionOfTravel::Subtract
        );
    }
}
