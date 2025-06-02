// src/parameter.rs

/// Parameter struct for bounded/unbounded f64 values
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    value: f64,
    min:   Option<f64>,
    max:   Option<f64>,
}

impl Parameter {
    /// Creates a new unconstrained Parameter.
    /// Panics if `value` is NaN or infinite.
    pub fn new(value: f64) -> Self {
        assert!(value.is_finite(), "value must be finite and not NaN or infinite");
        Self { value, min: None, max: None }
    }

    /// Creates a new Parameter with given bounds.
    /// Clamps the initial value to [min, max].
    /// Panics if `min` > `max` or if either bound is not finite.
    pub fn with_bounds(value: f64, min: f64, max: f64) -> Self {
        assert!(min.is_finite() && max.is_finite(), "bounds must be finite");
        assert!(min <= max, "min must be ≤ max");
        let mut v = value;
        assert!(v.is_finite(), "value must be finite and not NaN or infinite");
        if v < min { v = min; }
        if v > max { v = max; }
        Self { value: v, min: Some(min), max: Some(max) }
    }

    /// Returns the current value.
    pub fn get(&self) -> f64 {
        self.value
    }

    /// Sets a new value, clamping it to [min, max] (or [f64::MIN, f64::MAX] if unconstrained).
    /// Panics if `new_value` is NaN or infinite.
    pub fn set(&mut self, new_value: f64) {
        assert!(new_value.is_finite(), "value must be finite and not NaN or infinite");
        let lo = self.min.unwrap_or(f64::MIN);
        let hi = self.max.unwrap_or(f64::MAX);
        let v = if new_value < lo { lo } else if new_value > hi { hi } else { new_value };
        self.value = v;
    }
}

#[cfg(test)]
mod tests {
    use super::Parameter;

    #[test]
    fn given_unconstrained_new_and_set_values_work() {
        let mut p = Parameter::new(1.23);
        assert_eq!(p.get(), 1.23);
        p.set(-9.87);
        assert_eq!(p.get(), -9.87);
    }

    #[test]
    fn given_constrained_new_clamps_initial_value() {
        let p = Parameter::with_bounds(0.5, 1.0, 2.0);
        assert_eq!(p.get(), 1.0);
        let p2 = Parameter::with_bounds(3.0, 1.0, 2.0);
        assert_eq!(p2.get(), 2.0);
    }

    #[test]
    fn given_constrained_set_clamps_value() {
        let mut p = Parameter::with_bounds(1.5, 1.0, 2.0);
        p.set(0.0);
        assert_eq!(p.get(), 1.0);
        p.set(10.0);
        assert_eq!(p.get(), 2.0);
    }

    #[test]
    #[should_panic]
    fn new_panics_on_nan_or_infinite() {
        let _ = Parameter::new(f64::NAN);
    }

    #[test]
    #[should_panic]
    fn with_bounds_panics_on_invalid_bounds() {
        let _ = Parameter::with_bounds(0.0, f64::INFINITY, 1.0);
    }

    #[test]
    #[should_panic]
    fn set_panics_on_nan_or_infinite() {
        let mut p = Parameter::new(0.0);
        p.set(f64::INFINITY);
    }
}
