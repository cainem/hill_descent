// src/locus/new_random_locus_adjustment.rs
use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
use crate::parameters::parameter::Parameter;
use crate::E0;
use rand::Rng;
use std::ops::RangeInclusive;

impl LocusAdjustment {
    /// Creates a new LocusAdjustment with random properties.
    ///
    /// - `direction_of_travel` is chosen randomly (50/50 Add/Subtract).
    /// - `doubling_or_halving_flag` is chosen randomly (50/50 true/false).
    /// - `adjustment_value` is a random non-negative f64. The upper bound for this random
    ///   value is 10% of the span of `value_bounds_for_locus`, or E0 if 10% of the span is less than E0.
    pub fn new_random(
        rng: &mut impl Rng,
        value_bounds_for_locus: &RangeInclusive<f64>,
    ) -> Self {
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
        LocusAdjustment::new(
            adjustment_value,
            direction_of_travel,
            doubling_or_halving_flag,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Pulls in the impl LocusAdjustment block
    use crate::locus::locus_adjustment::DirectionOfTravel;
    use crate::E0;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

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
    }
}
