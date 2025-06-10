use crate::{gamete::Gamete, parameters::system_parameters::SystemParameters};
use rand::Rng;

pub mod asexual_reproduction;
pub mod calculate_crossovers;
pub mod compute_expressed;
pub mod compute_expressed_hash;
pub mod new_random_phenotype;
pub mod sexual_reproduction;
pub mod update_dimensions_key;

/// A Phenotype is constructed from a pair of gametes.
#[derive(Debug, Clone, PartialEq)]
pub struct Phenotype {
    /// The first gamete.
    gamete1: Gamete,
    /// The second gamete.
    gamete2: Gamete,
    /// The expressed parameter values derived from the two gametes.
    expressed: Vec<f64>,
    /// System parameters extracted from the first seven expressed values.
    system_parameters: SystemParameters,
    /// Hash of the expressed parameter values.
    expressed_hash: u64,
    dimensions_key: Option<Vec<usize>>,
    last_score: Option<f64>,
}

impl Phenotype {
    /// Creates a new Phenotype from two gametes, computing expressed values using the given RNG.
    pub fn new<R: Rng>(gamete1: Gamete, gamete2: Gamete, rng: &mut R) -> Self {
        let expressed = compute_expressed::compute_expressed(&gamete1, &gamete2, rng);
        if expressed.len() < 7 {
            panic!(
                "Cannot create Phenotype: expressed values (genes) length {} is less than required 7 for SystemParameters. Gametes need to provide at least 7 loci.",
                expressed.len()
            );
        }
        // Extract the first seven expressed values as system parameters
        let system_parameters = SystemParameters::new(&expressed[0..7]);
        let expressed_hash = Self::compute_expressed_hash(&expressed);
        Self {
            gamete1,
            gamete2,
            expressed, // Stores all expressed values
            system_parameters,
            expressed_hash,
            dimensions_key: None, // Optional field for dimensions key, can be set later
            last_score: None,     // Optional field for last score, can be set later
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

    /// Returns the hash of the expressed parameter values.
    pub fn expressed_hash(&self) -> u64 {
        self.expressed_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameters::parameter::Parameter;
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
        // Provide at least 7 values for loci to ensure SystemParameters can be derived.
        // The actual values for the first 7 can be arbitrary for this test's purpose,
        // as it only checks gamete storage.
        let g1_loci_values = &[1.0, 2.0, 0.1, 0.5, 0.001, 100.0, 2.0, 8.0, 9.0]; // 9 loci
        let g2_loci_values = &[3.0, 4.0, 0.1, 0.5, 0.001, 100.0, 2.0, 10.0, 11.0]; // 9 loci
        let g1 = create_test_gamete(g1_loci_values);
        let g2 = create_test_gamete(g2_loci_values);
        let mut rng = thread_rng();
        let ph = Phenotype::new(g1.clone(), g2.clone(), &mut rng);
        assert_eq!(ph.gametes(), (&g1, &g2));
        // Assert that expressed values and system parameters are also set.
        // The length of expressed_values should match the number of loci in the gametes.
        // This assumes compute_expressed returns one value per locus pair.
        assert_eq!(ph.expressed_values().len(), g1_loci_values.len());
        let _ = ph.system_parameters(); // Access to ensure it was created without panic

        // Assert that expressed_hash is correctly calculated and set
        let expected_hash = Phenotype::compute_expressed_hash(ph.expressed_values());
        assert_eq!(
            ph.expressed_hash(),
            expected_hash,
            "Expressed hash should match the re-calculated hash"
        );
    }
}
