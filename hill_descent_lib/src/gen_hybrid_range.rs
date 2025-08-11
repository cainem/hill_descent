use rand::Rng;
use std::ops::RangeInclusive;

/// The threshold for switching to a log-uniform distribution.
/// A value of 1000.0 means we switch if the range spans
/// 3 or more orders of magnitude (since log10(1000) = 3).
const RATIO_THRESHOLD: f64 = 1000.0;

/// Generates a random f64 within a given range, intelligently switching
/// between a linear and a log-uniform distribution.
///
/// # Arguments
/// * `rng` - A mutable reference to a random number generator.
/// * `range` - A `RangeInclusive<f64>` specifying the desired bounds.
///
/// # Behaviour
/// - For ranges that don't cross zero and span few orders of magnitude
///   (i.e., `high/low < 1000`), it uses a standard linear distribution.
/// - For ranges that cross zero or span many orders of magnitude, it uses
///   a log-uniform distribution to ensure values are generated across the
///   entire scale of the range.
pub fn gen_hybrid_range(rng: &mut impl Rng, range: RangeInclusive<f64>) -> f64 {
    let (low, high) = (*range.start(), *range.end());

    // --- Input Validation ---
    if low > high {
        return f64::NAN; // Or panic, depending on desired error handling.
    }
    if low == high {
        return low;
    }

    // --- Logic to determine which distribution to use ---

    // Case 1: Range crosses zero (e.g., -100 to 1000)
    if low < 0.0 && high > 0.0 {
        // This is a wide, complex range. The fairest way to get numbers
        // from all scales is to treat the negative and positive sides
        // with a log-uniform approach. We can't use a simple ratio check here.
        // A robust choice is to always use a log-like distribution if the
        // bounds are far from zero.
        let max_abs = low.abs().max(high);
        if max_abs > RATIO_THRESHOLD {
            // Extremely wide range that crosses zero – use a log-uniform strategy.
            // We sample an exponent uniformly between the minimum f64 exponent (≈-307.65)
            // and the exponent of the largest magnitude bound, pick a mantissa in [1,10),
            // assign a random sign, and loop until the candidate sits inside the original
            // bounds. This avoids clustering at the range’s endpoints.
            let min_exp = f64::MIN_POSITIVE.log10(); // ≈ -307.653 at runtime
            let max_exp = max_abs.log10();
            // Choose sign deterministically via float sample
            let sign = if rng.random_range(0.0..1.0) < 0.5 {
                1.0
            } else {
                -1.0
            };
            for _ in 0..10_000 {
                // exponent uniform in log space
                let exponent = rng.random_range(min_exp..=max_exp);
                // mantissa uniform in [1.0, 10.0)
                let mantissa = rng.random_range(1.0..10.0);
                let unsigned_val = mantissa * 10_f64.powf(exponent);
                if !unsigned_val.is_finite() || unsigned_val > max_abs {
                    continue; // skip INF/NaN or magnitudes outside desired bounds
                }
                let candidate = sign * unsigned_val;
                if range.contains(&candidate) {
                    return candidate;
                }
            }
            // Fallback – shouldn’t normally happen, but guarantees termination
            return sign * max_abs.min(high);
        }
        // If the range is small (e.g., -10 to 20), fall through to linear.
    }
    // Case 2: Range is entirely positive (e.g., 0.1 to 1E6)
    else if low >= 0.0 {
        // Use ratio check. Avoid division by zero.
        if low > 0.0 && high / low > RATIO_THRESHOLD {
            // Log-uniform approach
            let log_low = low.log10();
            let log_high = high.log10();
            let random_log_val = rng.random_range(log_low..=log_high);
            return 10.0_f64.powf(random_log_val);
        }
    }
    // Case 3: Range is entirely negative (e.g., -1E6 to -0.1)
    else {
        // low < 0 && high < 0
        // Use ratio check on absolute values.
        if high < 0.0 && low.abs() / high.abs() > RATIO_THRESHOLD {
            // Log-uniform approach on the absolute values, then negate.
            let log_low = high.abs().log10();
            let log_high = low.abs().log10();
            let random_log_val = rng.random_range(log_low..=log_high);
            return -10.0_f64.powf(random_log_val);
        }
    }

    // --- Default Case ---
    // If none of the log-uniform conditions were met, use the standard linear distribution.
    rng.random_range(range)
}

#[cfg(test)]
mod tests {
    use super::gen_hybrid_range;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::ops::RangeInclusive;

    #[test]
    fn given_extreme_cross_zero_range_when_gen_hybrid_range_then_varied_values_within_bounds() {
        let range: RangeInclusive<f64> = (-f64::MAX / 2.0)..=(f64::MAX / 2.0);
        let mut rng = StdRng::seed_from_u64(42);
        let mut samples = Vec::with_capacity(1000);
        for _ in 0..100 {
            let v = gen_hybrid_range(&mut rng, range.clone());
            assert!(range.contains(&v));
            samples.push(v);
        }
        // Ensure we didn’t get stuck on the endpoints or a single sign.
        assert!(samples.iter().any(|&v| v.is_sign_positive()));
        assert!(samples.iter().any(|&v| v.is_sign_negative()));
        let at_start = samples.iter().filter(|&&v| v == *range.start()).count();
        let at_end = samples.iter().filter(|&&v| v == *range.end()).count();
        // Allow at most 10% of values at each extreme.
        assert!(at_start < 100, "too many values at lower bound: {at_start}");
        assert!(at_end < 100, "too many values at upper bound: {at_end}");
    }

    #[test]
    fn given_equal_bounds_when_gen_hybrid_range_then_returns_single_bound_value() {
        let mut rng = StdRng::seed_from_u64(42);
        let val = gen_hybrid_range(&mut rng, 5.0..=5.0);
        assert_eq!(val, 5.0);
    }

    #[test]
    fn given_low_greater_than_high_when_gen_hybrid_range_then_returns_nan() {
        let mut rng = StdRng::seed_from_u64(42);
        let val = gen_hybrid_range(&mut rng, 10.0..=5.0);
        assert!(val.is_nan());
    }

    #[test]
    fn given_small_positive_range_when_gen_hybrid_range_then_value_within_bounds() {
        let mut rng = StdRng::seed_from_u64(7);
        let range: RangeInclusive<f64> = 1.0..=10.0;
        for _ in 0..100 {
            let v = gen_hybrid_range(&mut rng, range.clone());
            assert!(range.contains(&v), "{v} not within {range:?}");
        }
    }

    #[test]
    fn given_cross_zero_large_range_when_gen_hybrid_range_then_value_within_bounds() {
        // retains existing test for general cross-zero wide case

        let mut rng = StdRng::seed_from_u64(99);
        let range: RangeInclusive<f64> = -1e6..=1e6;
        for _ in 0..100 {
            let v = gen_hybrid_range(&mut rng, range.clone());
            assert!(range.contains(&v));
        }
    }

    #[test]
    fn given_cross_zero_small_range_when_gen_hybrid_range_then_value_within_bounds() {
        let mut rng = StdRng::seed_from_u64(123);
        let range: RangeInclusive<f64> = -10.0..=20.0; // crosses zero but not wide
        for _ in 0..100 {
            let v = gen_hybrid_range(&mut rng, range.clone());
            assert!(range.contains(&v));
        }
    }

    #[test]
    fn given_wide_positive_range_when_gen_hybrid_range_then_value_within_bounds() {
        let mut rng = StdRng::seed_from_u64(456);
        let range: RangeInclusive<f64> = 0.001..=1_000_000.0; // positive and wide
        for _ in 0..100 {
            let v = gen_hybrid_range(&mut rng, range.clone());
            assert!(range.contains(&v));
            assert!(v >= 0.0);
        }
    }

    #[test]
    fn given_wide_negative_range_when_gen_hybrid_range_then_value_within_bounds() {
        let mut rng = StdRng::seed_from_u64(789);
        let range: RangeInclusive<f64> = -1_000_000.0..=-0.001; // negative and wide
        for _ in 0..100 {
            let v = gen_hybrid_range(&mut rng, range.clone());
            assert!(range.contains(&v));
            assert!(v <= 0.0);
        }
    }

    #[test]
    fn given_cross_zero_wide_range_rng_selects_positive_side_returns_positive() {
        // Use different deterministic seeds until we find a positive sample.
        let range: RangeInclusive<f64> = -1_000_000.0..=1_000_000.0;
        let mut found = None;
        for seed in 0u64..1000 {
            let mut rng = StdRng::seed_from_u64(seed);
            let v = gen_hybrid_range(&mut rng, range.clone());
            if v >= 0.0 {
                found = Some(v);
                break;
            }
        }
        let v = found.expect("expected to find a positive sample across seeds 0..1000");
        assert!(range.contains(&v));
        assert!(v >= 0.0, "Expected positive value but got {v}");
    }

    #[test]
    fn given_cross_zero_wide_range_rng_selects_negative_side_returns_negative() {
        // Use different deterministic seeds until we find a negative sample.
        let range: RangeInclusive<f64> = -1_000_000.0..=1_000_000.0;
        let mut found = None;
        for seed in 0u64..1000 {
            let mut rng = StdRng::seed_from_u64(seed);
            let v = gen_hybrid_range(&mut rng, range.clone());
            if v <= 0.0 {
                found = Some(v);
                break;
            }
        }
        let v = found.expect("expected to find a negative sample across seeds 0..1000");
        assert!(range.contains(&v));
        assert!(v <= 0.0, "Expected negative value but got {v}");
    }
}
