pub mod reproduce;

use crate::locus::Locus;

/// A gamete is a string of loci contributed by a parent organism.
#[derive(Debug, Clone, PartialEq)]
pub struct Gamete {
    /// The list of loci for this gamete, one per genetic dimension.
    loci: Vec<Locus>,
}

impl Gamete {
    /// Creates a new Gamete from a vector of loci.
    pub fn new(loci: Vec<Locus>) -> Self {
        Self { loci }
    }

    /// Returns a slice of loci.
    pub fn loci(&self) -> &[Locus] {
        &self.loci
    }

    /// Consumes the gamete and returns the underlying loci.
    pub fn into_loci(self) -> Vec<Locus> {
        self.loci
    }

    /// Returns the number of loci in this gamete.
    pub fn len(&self) -> usize {
        self.loci.len()
    }

    /// Returns true if this gamete contains no loci.
    pub fn is_empty(&self) -> bool {
        self.loci.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameter::Parameter;

    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    #[test]
    fn given_empty_loci_when_new_then_len_is_zero_and_is_empty() {
        let loci = vec![];
        let gamete = Gamete::new(loci);
        assert_eq!(gamete.len(), 0);
        assert!(gamete.is_empty());
        assert!(gamete.loci().is_empty());
    }

    #[test]
    fn given_non_empty_loci_when_new_then_len_and_accessors_work() {
        let loci = vec![create_test_locus(1.0), create_test_locus(2.0)];
        let gamete = Gamete::new(loci.clone());
        assert_eq!(gamete.len(), 2);
        assert!(!gamete.is_empty());
        assert_eq!(gamete.loci(), loci.as_slice());
        assert_eq!(gamete.into_loci(), loci);
    }
}
