// src/locus/mutate.rs
use super::Locus; // Locus struct is defined in src/locus/mod.rs
use crate::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
use crate::system_parameters::SystemParameters;
use rand::Rng;

impl Locus {
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
    use crate::locus::Locus; // To use the Locus struct definition
    use crate::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameter::Parameter;
    use crate::system_parameters::SystemParameters;
    use rand::rngs::mock::StepRng;

    // Original helper function, kept for existing tests if they rely on its specific setup.
    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    // Helper function to create a Locus with specific initial values for testing
    fn create_test_locus_detailed(
        value: f64,
        adj_value: f64,
        direction: DirectionOfTravel,
        double_flag: bool,
        apply_flag: bool,
    ) -> Locus {
        let locus_val = Parameter::new(value);
        let adj_param = Parameter::new(adj_value);
        let adjustment = LocusAdjustment::new(adj_param, direction, double_flag);
        Locus::new(locus_val, adjustment, apply_flag)
    }

    #[test]
    fn given_all_mutation_probs_zero_when_mutate_then_locus_is_unchanged() {
        let initial_locus =
            create_test_locus_detailed(1.5, 0.5, DirectionOfTravel::Add, false, false);
        let mut rng = StepRng::new(0, 0);
        // m1, m2, m3, m4, m5 all 0.0
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0]);
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert_eq!(mutated_locus, initial_locus);
    }

    #[test]
    fn given_m4_true_direction_add_when_mutate_then_direction_subtract_double_flag_inverted() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, false, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 1.0, 0.0]); // m4 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert_eq!(
            mutated_locus.adjustment().direction_of_travel(),
            DirectionOfTravel::Subtract
        );
        assert!(mutated_locus.adjustment().doubling_or_halving_flag()); // false -> true
    }

    #[test]
    fn given_m4_true_direction_subtract_when_mutate_then_direction_add_double_flag_inverted() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Subtract, true, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 1.0, 0.0]); // m4 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert_eq!(
            mutated_locus.adjustment().direction_of_travel(),
            DirectionOfTravel::Add
        );
        assert!(!mutated_locus.adjustment().doubling_or_halving_flag()); // true -> false
    }

    #[test]
    fn given_m3_true_double_flag_false_when_mutate_then_double_flag_true() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, false, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 1.0, 0.0, 0.0]); // m3 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(mutated_locus.adjustment().doubling_or_halving_flag());
    }

    #[test]
    fn given_m3_true_double_flag_true_when_mutate_then_double_flag_false() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, true, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 1.0, 0.0, 0.0]); // m3 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(!mutated_locus.adjustment().doubling_or_halving_flag());
    }

    #[test]
    fn given_m5_true_double_flag_true_when_mutate_then_adj_value_doubled() {
        let initial_locus =
            create_test_locus_detailed(1.0, 2.0, DirectionOfTravel::Add, true, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 1.0]); // m5 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert_eq!(mutated_locus.adjustment().adjustment_value().get(), 4.0);
    }

    #[test]
    fn given_m5_true_double_flag_false_when_mutate_then_adj_value_halved() {
        let initial_locus =
            create_test_locus_detailed(1.0, 2.0, DirectionOfTravel::Add, false, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 1.0]); // m5 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert_eq!(mutated_locus.adjustment().adjustment_value().get(), 1.0);
    }

    #[test]
    fn given_m1_true_apply_flag_false_when_mutate_then_apply_flag_true() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, false, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[1.0, 0.0, 0.0, 0.0, 0.0]); // m1 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(mutated_locus.apply_adjustment_flag());
    }

    #[test]
    fn given_m2_true_apply_flag_true_when_mutate_then_apply_flag_false() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, false, true);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 1.0, 0.0, 0.0, 0.0]); // m2 = 1.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(!mutated_locus.apply_adjustment_flag());
    }

    #[test]
    fn given_m1_false_apply_flag_false_when_mutate_then_apply_flag_remains_false() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, false, false);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0]); // m1 = 0.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(!mutated_locus.apply_adjustment_flag());
    }

    #[test]
    fn given_m2_false_apply_flag_true_when_mutate_then_apply_flag_remains_true() {
        let initial_locus =
            create_test_locus_detailed(1.0, 0.1, DirectionOfTravel::Add, false, true);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0]); // m2 = 0.0
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(mutated_locus.apply_adjustment_flag());
    }

    #[test]
    fn given_final_apply_flag_true_direction_add_when_mutate_then_value_increases() {
        let initial_value = 10.0;
        let adj_val = 2.0;
        // Start with apply_flag = false, m1 will flip it to true
        let initial_locus = create_test_locus_detailed(
            initial_value,
            adj_val,
            DirectionOfTravel::Add,
            false,
            false,
        );
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[1.0, 0.0, 0.0, 0.0, 0.0]); // m1 = 1.0 to activate apply_flag
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(mutated_locus.apply_adjustment_flag());
        assert_eq!(mutated_locus.value().get(), initial_value + adj_val);
    }

    #[test]
    fn given_final_apply_flag_true_direction_subtract_when_mutate_then_value_decreases() {
        let initial_value = 10.0;
        let adj_val = 2.0;
        // Start with apply_flag = false, m1 will flip it to true
        let initial_locus = create_test_locus_detailed(
            initial_value,
            adj_val,
            DirectionOfTravel::Subtract,
            false,
            false,
        );
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[1.0, 0.0, 0.0, 0.0, 0.0]); // m1 = 1.0 to activate apply_flag
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(mutated_locus.apply_adjustment_flag());
        assert_eq!(mutated_locus.value().get(), initial_value - adj_val);
    }

    #[test]
    fn given_final_apply_flag_false_when_mutate_then_value_unchanged_by_application() {
        let initial_value = 10.0;
        let adj_val = 2.0;
        // Start with apply_flag = true, m2 will flip it to false
        let initial_locus =
            create_test_locus_detailed(initial_value, adj_val, DirectionOfTravel::Add, false, true);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[0.0, 1.0, 0.0, 0.0, 0.0]); // m2 = 1.0 to deactivate apply_flag
        let mutated_locus = initial_locus.mutate(&mut rng, &sys);
        assert!(!mutated_locus.apply_adjustment_flag());
        assert_eq!(mutated_locus.value().get(), initial_value); // Value should not change as apply_flag is false
    }

    // Existing tests - renamed one for clarity and kept the other as is.
    #[test]
    fn mutate_no_mutation_returns_same() {
        // Renamed from: given_all_mutation_probs_zero_when_mutate_then_locus_is_unchanged (original name)
        let l = create_test_locus(1.5); // Uses simpler helper, specific initial state
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::new(&[]); // All probs default to 0.0
        let l2 = l.mutate(&mut rng, &sys);
        assert_eq!(l2, l);
    }

    #[test]
    fn mutate_with_full_probs_applies_flag_flip_and_other_mutations() {
        // Clarified name for existing test
        let l = create_test_locus(2.0); // Uses simpler helper
        let mut rng = StepRng::new(0, 0); // Changed from u64::MAX,0 as 0/1 probabilities make StepRng state less critical
        let sys = SystemParameters::new(&[1.0, 1.0, 1.0, 1.0, 1.0]);
        let l2 = l.mutate(&mut rng, &sys);

        // Expected sequence with all probs = 1.0 and initial locus from create_test_locus(2.0):
        // Initial: val=2.0, adj_val=0.0, dir=Add, double=false, apply=false
        // 1. m4 (Direction): dir=Add -> Subtract, double=false -> true
        //    Locus state: adj_val=0.0, dir=Subtract, double=true
        // 2. m3 (Doubling): double=true -> false
        //    Locus state: adj_val=0.0, dir=Subtract, double=false
        // 3. m5 (Adj Value): double=false -> adj_val /= 2.0 (0.0 / 2.0 = 0.0)
        //    Locus state: adj_val=0.0, dir=Subtract, double=false
        // new_adjustment created with these values.
        // 4. m1/m2 (Apply Flag): initial apply=false. m1=1.0 -> apply=true
        //    Locus state: final apply_flag = true
        // 5. Value Application: apply=true. new_value = old_value + (sign * adj_val)
        //    sign for Subtract is -1.0. delta = -1.0 * 0.0 = 0.0. new_value = 2.0 + 0.0 = 2.0

        assert_ne!(
            l2, l,
            "Locus should change due to adjustment mutations even if value does not"
        );
        assert!(l2.apply_adjustment_flag(), "Apply flag should be true");
        assert_eq!(
            l2.adjustment().direction_of_travel(),
            DirectionOfTravel::Subtract,
            "Direction should be Subtract"
        );
        assert!(
            !l2.adjustment().doubling_or_halving_flag(),
            "Doubling flag should be false"
        );
        assert_eq!(
            l2.adjustment().adjustment_value().get(),
            0.0,
            "Adjustment value should be 0.0"
        );
        assert_eq!(l2.value().get(), 2.0, "Locus value should be 2.0");
    }
}
