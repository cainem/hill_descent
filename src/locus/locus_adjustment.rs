use crate::E0;
use rand::Rng;
use std::ops::RangeInclusive;
use xxhash_rust::xxh3::xxh3_64;

use crate::parameters::parameter::Parameter;

/// Direction of how adjustment is applied: add or subtract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionOfTravel {
    Add,
    Subtract,
}

/// A potential modification to a locus value.
#[derive(Debug, Clone, PartialEq)]
pub struct LocusAdjustment {
    adjustment_value: Parameter,
    direction_of_travel: DirectionOfTravel,
    doubling_or_halving_flag: bool,
    checksum: u64,
}

impl LocusAdjustment {
    /// Compute checksum from adjustment parameters.
    fn compute_checksum(
        adjustment_value: &Parameter,
        direction_of_travel: DirectionOfTravel,
        doubling_or_halving_flag: bool,
    ) -> u64 {
        let mut buf = Vec::with_capacity(10);
        buf.extend_from_slice(&adjustment_value.get().to_le_bytes());
        buf.push(match direction_of_travel {
            DirectionOfTravel::Add => 0,
            DirectionOfTravel::Subtract => 1,
        });
        buf.push(if doubling_or_halving_flag { 1 } else { 0 });
        xxh3_64(&buf)
    }

    /// Constructs a new LocusAdjustment, enforcing adjustment_value >= 0
    /// and computing the checksum via XXH3 over (value, direction, flag).
    pub fn new(
        adjustment_value: Parameter,
        direction_of_travel: DirectionOfTravel,
        doubling_or_halving_flag: bool,
    ) -> Self {
        assert!(
            adjustment_value.get() >= 0.0,
            "adjustment_value must be >= 0"
        );
        let checksum = Self::compute_checksum(
            &adjustment_value,
            direction_of_travel,
            doubling_or_halving_flag,
        );
        Self {
            adjustment_value,
            direction_of_travel,
            doubling_or_halving_flag,
            checksum,
        }
    }

    /// Returns the adjustment value.
    pub fn adjustment_value(&self) -> &Parameter {
        &self.adjustment_value
    }

    /// Returns the direction of travel for this adjustment.
    pub fn direction_of_travel(&self) -> DirectionOfTravel {
        self.direction_of_travel
    }

    /// Returns whether the adjustment value will double (true) or halve (false) on mutation.
    pub fn doubling_or_halving_flag(&self) -> bool {
        self.doubling_or_halving_flag
    }

    /// Returns the precomputed checksum over the adjustment state.
    pub fn checksum(&self) -> u64 {
        self.checksum
    }

    /// Creates a new LocusAdjustment with random properties.
    ///
    /// - `direction_of_travel` is chosen randomly (50/50 Add/Subtract).
    /// - `doubling_or_halving_flag` is chosen randomly (50/50 true/false).
    /// - `adjustment_value` is a random non-negative f64. The upper bound for this random
    ///   value is 10% of the span of `value_bounds_for_locus`, or E0 if 10% of the span is less than E0.
    pub fn new_random(rng: &mut impl Rng, value_bounds_for_locus: &RangeInclusive<f64>) -> Self {
        let direction_of_travel = if rng.r#gen::<bool>() {
            DirectionOfTravel::Add
        } else {
            DirectionOfTravel::Subtract
        };
        let doubling_or_halving_flag = rng.r#gen::<bool>();
        let locus_span = *value_bounds_for_locus.end() - *value_bounds_for_locus.start();
        let max_adj_val = (locus_span.abs() * 0.1).max(E0);
        let random_adj_val = rng.gen_range(0.0..=max_adj_val);
        let adjustment_value = Parameter::new(random_adj_val);
        Self::new(
            adjustment_value,
            direction_of_travel,
            doubling_or_halving_flag,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::E0;
    use crate::parameters::parameter::Parameter;
    use rand::SeedableRng;
    use rand::rngs::StdRng; // Ensure E0 is available in test scope

    use super::*;

    #[test]
    fn given_valid_params_when_new_then_fields_set() {
        let p = Parameter::new(1.23);
        let adj1 = LocusAdjustment::new(p.clone(), DirectionOfTravel::Add, false);
        assert_eq!(adj1.adjustment_value(), &p);
        assert_eq!(adj1.direction_of_travel(), DirectionOfTravel::Add);
        assert!(!adj1.doubling_or_halving_flag());
        // deterministic checksum
        let adj2 = LocusAdjustment::new(p.clone(), DirectionOfTravel::Add, false);
        assert_eq!(adj1.checksum(), adj2.checksum());
    }

    #[test]
    #[should_panic]
    fn given_negative_adjustment_when_new_then_panic() {
        let pneg = Parameter::new(-1.0);
        let _ = LocusAdjustment::new(pneg, DirectionOfTravel::Subtract, true);
    }

    #[test]
    fn given_rng_and_bounds_when_new_random_then_adjustment_is_sensible() {
        let mut rng = StdRng::seed_from_u64(42);
        let bounds = 0.0..=100.0;

        for _ in 0..100 {
            let adj = LocusAdjustment::new_random(&mut rng, &bounds);
            assert!(adj.adjustment_value().get() >= 0.0);
            assert!(
                adj.adjustment_value().get() <= 10.0,
                "adj_val: {} should be <= 10.0",
                adj.adjustment_value().get()
            );
        }

        let narrow_bounds = 5.0..=5.1;
        let expected_max_narrow = 0.01f64.max(E0);
        for _ in 0..100 {
            let adj_narrow = LocusAdjustment::new_random(&mut rng, &narrow_bounds);
            assert!(adj_narrow.adjustment_value().get() >= 0.0);
            assert!(
                adj_narrow.adjustment_value().get() <= expected_max_narrow,
                "adj_val: {} should be <= expected_max_narrow: {}",
                adj_narrow.adjustment_value().get(),
                expected_max_narrow
            );
        }

        let zero_span_bounds = 10.0..=10.0;
        for _ in 0..100 {
            let adj_zero_span = LocusAdjustment::new_random(&mut rng, &zero_span_bounds);
            assert!(adj_zero_span.adjustment_value().get() >= 0.0);
            assert!(
                adj_zero_span.adjustment_value().get() <= E0,
                "adj_val: {} should be <= E0: {}",
                adj_zero_span.adjustment_value().get(),
                E0
            );
        }
    }

    #[test]
    fn given_seeded_rng_when_new_random_then_flags_are_deterministic_for_test() {
        let mut rng = StdRng::seed_from_u64(123);
        let bounds = 0.0..=10.0;

        let adj1 = LocusAdjustment::new_random(&mut rng, &bounds);
        assert_eq!(
            adj1.direction_of_travel(),
            DirectionOfTravel::Add,
            "First direction should be Add"
        );
        assert!(
            !adj1.doubling_or_halving_flag(),
            "First D/H flag should be false"
        );

        let adj2 = LocusAdjustment::new_random(&mut rng, &bounds);
        assert_eq!(
            adj2.direction_of_travel(),
            DirectionOfTravel::Subtract,
            "Second direction should be Subtract"
        );
        assert!(
            adj2.doubling_or_halving_flag(),
            "Second D/H flag should be true"
        );

        let adj3 = LocusAdjustment::new_random(&mut rng, &bounds);
        assert_eq!(
            adj3.direction_of_travel(),
            DirectionOfTravel::Subtract,
            "Third direction should be Subtract"
        );
        assert!(
            adj3.doubling_or_halving_flag(),
            "Third D/H flag should be true"
        );
    }
}
