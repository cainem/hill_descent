use crate::gamete::Gamete;
use crate::system_parameters::SystemParameters;
use rand::Rng;

pub mod compute_expressed;

/// A Phenotype is constructed from a pair of gametes.
#[derive(Debug, Clone, PartialEq)]
pub struct Phenotype {
    /// The first gamete.
    gamete1: Gamete,
    /// The second gamete.
    gamete2: Gamete,
    /// The expressed parameter values derived from the two gametes.
    expressed: Vec<f64>,
    /// System parameters extracted from the first five expressed values.
    system_parameters: SystemParameters,
}

impl Phenotype {
    /// Creates a new Phenotype from two gametes, computing expressed values using the given RNG.
    pub fn new<R: Rng>(gamete1: Gamete, gamete2: Gamete, rng: &mut R) -> Self {
        let expressed = compute_expressed::compute_expressed(&gamete1, &gamete2, rng);
        // Extract the first five expressed values as system parameters
        let system_parameters = SystemParameters::new(&expressed);
        Self {
            gamete1,
            gamete2,
            expressed,
            system_parameters,
        }
    }

    /// Returns a reference to the first gamete.
    pub fn gamete1(&self) -> &Gamete {
        &self.gamete1
    }

    /// Returns a reference to the second gamete.
    pub fn gamete2(&self) -> &Gamete {
        &self.gamete2
    }

    /// Returns references to the two gametes.
    pub fn gametes(&self) -> (&Gamete, &Gamete) {
        (&self.gamete1, &self.gamete2)
    }

    /// Returns the expressed parameter values.
    pub fn expressed_values(&self) -> &[f64] {
        &self.expressed
    }

    /// Returns a reference to the system parameters (m1..m5).
    pub fn system_parameters(&self) -> &SystemParameters {
        &self.system_parameters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus::Locus;
    use crate::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameter::Parameter;
    use rand::thread_rng;

    pub(crate) fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    pub(crate) fn create_test_gamete(vals: &[f64]) -> Gamete {
        let loci = vals.iter().map(|&v| create_test_locus(v)).collect();
        Gamete::new(loci)
    }

    #[test]
    fn given_two_gametes_when_new_then_fields_are_set() {
        let g1 = create_test_gamete(&[1.0, 2.0]);
        let g2 = create_test_gamete(&[3.0, 4.0]);
        let mut rng = thread_rng();
        let ph = Phenotype::new(g1.clone(), g2.clone(), &mut rng);
        assert_eq!(ph.gametes(), (&g1, &g2));
    }
}
