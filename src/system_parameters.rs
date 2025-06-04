/// System-wide evolvable parameters for the GA (e.g., mutation rates).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemParameters {
    m1: f64,
    m2: f64,
    m3: f64,
    m4: f64,
    m5: f64,
}

impl SystemParameters {
    /// Constructs SystemParameters from a slice `[m1, m2, m3, m4, m5]`. Missing entries default to zero.
    pub fn new(values: &[f64]) -> Self {
        let mut sp = SystemParameters::default();
        if let Some(&v) = values.get(0) { sp.m1 = v; }
        if let Some(&v) = values.get(1) { sp.m2 = v; }
        if let Some(&v) = values.get(2) { sp.m3 = v; }
        if let Some(&v) = values.get(3) { sp.m4 = v; }
        if let Some(&v) = values.get(4) { sp.m5 = v; }
        sp
    }

    /// Returns mutation probability m1.
    pub fn m1(&self) -> f64 { self.m1 }
    /// Returns mutation probability m2.
    pub fn m2(&self) -> f64 { self.m2 }
    /// Returns mutation probability m3.
    pub fn m3(&self) -> f64 { self.m3 }
    /// Returns mutation probability m4.
    pub fn m4(&self) -> f64 { self.m4 }
    /// Returns mutation probability m5.
    pub fn m5(&self) -> f64 { self.m5 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_full_slice_sets_fields() {
        let params = SystemParameters::new(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(params.m1(), 1.0);
        assert_eq!(params.m2(), 2.0);
        assert_eq!(params.m3(), 3.0);
        assert_eq!(params.m4(), 4.0);
        assert_eq!(params.m5(), 5.0);
    }

    #[test]
    fn new_with_partial_slice_defaults_remaining() {
        let params = SystemParameters::new(&[1.0, 2.0]);
        assert_eq!(params.m1(), 1.0);
        assert_eq!(params.m2(), 2.0);
        assert_eq!(params.m3(), 0.0);
        assert_eq!(params.m4(), 0.0);
        assert_eq!(params.m5(), 0.0);
    }
}
