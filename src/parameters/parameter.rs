// src/parameter.rs
use std::ops::RangeInclusive;

/// Parameter struct for bounded/unbounded f64 values
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    value: f64,
    bounds: RangeInclusive<f64>,
}

impl Parameter {
    /// Creates a new Parameter with default bounds [f64::MIN, f64::MAX].
    /// Panics if `value` is NaN or infinite.
    pub fn new(value: f64) -> Self {
        assert!(
            value.is_finite(),
            "value must be finite and not NaN or infinite"
        );
        Self {
            value,
            bounds: f64::MIN..=f64::MAX,
        }
    }

    /// Creates a new Parameter with given bounds.
    /// Clamps the initial value to [min, max].
    /// Panics if `min` > `max` or if either bound is not finite.
    pub fn with_bounds(value: f64, min_val: f64, max_val: f64) -> Self {
        assert!(
            min_val.is_finite() && max_val.is_finite(),
            "bounds must be finite"
        );
        assert!(min_val <= max_val, "min must be ≤ max");
        let mut v = value;
        assert!(
            v.is_finite(),
            "value must be finite and not NaN or infinite"
        );
        if v < min_val {
            v = min_val;
        }
        if v > max_val {
            v = max_val;
        }
        Self {
            value: v,
            bounds: min_val..=max_val,
        }
    }

    /// Returns the current value.
    pub fn get(&self) -> f64 {
        self.value
    }

    /// Sets a new value, clamping it to the parameter's bounds.
    /// Panics if `new_value` is NaN or infinite.
    pub fn set(&mut self, new_value: f64) {
        assert!(
            new_value.is_finite(),
            "value must be finite and not NaN or infinite"
        );
        let lo = *self.bounds.start();
        let hi = *self.bounds.end();
        let v = if new_value < lo {
            lo
        } else if new_value > hi {
            hi
        } else {
            new_value
        };
        self.value = v;
    }

    /// Sets a new value without clamping to bounds (for unbounded parameters).
    /// Panics if `new_value` is NaN or infinite.
    pub fn set_unbound(&mut self, new_value: f64) {
        assert!(
            new_value.is_finite(),
            "value must be finite and not NaN or infinite"
        );
        self.value = new_value;
    }

    /// Returns the bounds of the parameter.
    pub fn bounds(&self) -> &RangeInclusive<f64> {
        &self.bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_new_when_value_is_valid_then_parameter_is_created_with_default_bounds() {
        let p = Parameter::new(1.23);
        assert_eq!(p.get(), 1.23);
        assert_eq!(p.bounds(), &(f64::MIN..=f64::MAX));
    }

    #[test]
    fn given_new_and_set_when_values_are_valid_then_values_are_set_correctly() {
        let mut p = Parameter::new(1.23);
        assert_eq!(p.get(), 1.23);
        p.set(-9.87);
        assert_eq!(p.get(), -9.87);
        // Test clamping with default bounds (effectively unconstrained for typical values)
        p.set(f64::MAX / 2.0);
        assert_eq!(p.get(), f64::MAX / 2.0);
        p.set(f64::MIN / 2.0);
        assert_eq!(p.get(), f64::MIN / 2.0);
    }

    #[test]
    fn given_with_bounds_when_value_is_within_bounds_then_parameter_is_created() {
        let p = Parameter::with_bounds(1.5, 1.0, 2.0);
        assert_eq!(p.get(), 1.5);
        assert_eq!(p.bounds(), &(1.0..=2.0));
    }

    #[test]
    fn given_with_bounds_when_value_is_below_min_then_value_is_clamped_to_min() {
        let p = Parameter::with_bounds(0.5, 1.0, 2.0);
        assert_eq!(p.get(), 1.0);
    }

    #[test]
    fn given_with_bounds_when_value_is_above_max_then_value_is_clamped_to_max() {
        let p = Parameter::with_bounds(3.0, 1.0, 2.0);
        assert_eq!(p.get(), 2.0);
    }

    #[test]
    fn given_set_when_value_is_within_bounds_then_value_is_set() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set(1.7);
        assert_eq!(p.get(), 1.7);
    }

    #[test]
    fn given_set_when_value_is_below_min_then_value_is_clamped_to_min() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set(0.0);
        assert_eq!(p.get(), 1.0);
    }

    #[test]
    fn given_set_when_value_is_above_max_then_value_is_clamped_to_max() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set(10.0);
        assert_eq!(p.get(), 2.0);
    }

    #[test]
    fn given_set_unbound_when_value_is_within_bounds_then_value_is_set() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set_unbound(1.7);
        assert_eq!(p.get(), 1.7);
    }

    #[test]
    fn given_set_unbound_when_value_is_below_min_then_value_is_set_without_clamping() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set_unbound(0.5);
        assert_eq!(p.get(), 0.5);
    }

    #[test]
    fn given_set_unbound_when_value_is_above_max_then_value_is_set_without_clamping() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set_unbound(10.0);
        assert_eq!(p.get(), 10.0);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_new_when_value_is_nan_then_panics() {
        let _ = Parameter::new(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_new_when_value_is_infinite_then_panics() {
        let _ = Parameter::new(f64::INFINITY);
    }

    #[test]
    #[should_panic(expected = "bounds must be finite")]
    fn given_with_bounds_when_min_is_nan_then_panics() {
        let _ = Parameter::with_bounds(0.0, f64::NAN, 1.0);
    }

    #[test]
    #[should_panic(expected = "bounds must be finite")]
    fn given_with_bounds_when_max_is_infinite_then_panics() {
        let _ = Parameter::with_bounds(0.0, 0.0, f64::INFINITY);
    }

    #[test]
    #[should_panic(expected = "min must be ≤ max")]
    fn given_with_bounds_when_min_greater_than_max_then_panics() {
        let _ = Parameter::with_bounds(0.0, 1.0, 0.0);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_with_bounds_when_value_is_nan_then_panics() {
        let _ = Parameter::with_bounds(f64::NAN, 0.0, 1.0);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_set_when_value_is_nan_then_panics() {
        let mut p = Parameter::new(0.0);
        p.set(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_set_when_value_is_infinite_then_panics() {
        let mut p = Parameter::new(0.0);
        p.set(f64::INFINITY);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_set_unbound_when_value_is_nan_then_panics() {
        let mut p = Parameter::new(0.0);
        p.set_unbound(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "value must be finite and not NaN or infinite")]
    fn given_set_unbound_when_value_is_infinite_then_panics() {
        let mut p = Parameter::new(0.0);
        p.set_unbound(f64::INFINITY);
    }
}
