use crate::{
    NUM_SYSTEM_PARAMETERS, gamete::Gamete, parameters::system_parameters::SystemParameters,
};
use rand::Rng; // Retain Rng for Phenotype::new, though not used in new_for_test directly.

// Note: Locus, LocusAdjustment, DirectionOfTravel, Parameter imports moved into new_for_test

pub mod calculate_crossovers;
pub mod compute_expressed;
pub mod compute_expressed_hash;
pub mod new_random_phenotype;
pub mod sexual_reproduction;

/// A Phenotype is constructed from a pair of gametes.
#[derive(Debug, Clone, PartialEq)]
// Derived traits and parameter values expressed from two gametes.
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
}

impl Phenotype {
    /// Creates a new Phenotype from two gametes, computing expressed values using the given RNG.
    pub fn new<R: Rng>(gamete1: Gamete, gamete2: Gamete, rng: &mut R) -> Self {
        let expressed = compute_expressed::compute_expressed(&gamete1, &gamete2, rng);
        if expressed.len() < NUM_SYSTEM_PARAMETERS {
            panic!(
                "Cannot create Phenotype: expressed values (genes) length {} is less than required {} for SystemParameters. Gametes need to provide at least {} loci.",
                expressed.len(),
                NUM_SYSTEM_PARAMETERS,
                NUM_SYSTEM_PARAMETERS
            );
        }
        let expressed_hash = Self::compute_expressed_hash(&expressed, NUM_SYSTEM_PARAMETERS);
        let system_parameters = SystemParameters::new(&expressed[0..NUM_SYSTEM_PARAMETERS]);

        Self {
            gamete1,
            gamete2,
            expressed,
            system_parameters,
            expressed_hash,
        }
    }

    #[cfg(test)]
    /// Creates a new `Phenotype` instance specifically for testing purposes.
    ///
    /// This constructor takes a vector of `f64` representing the desired expressed values.
    /// It computes the `system_parameters` and `expressed_hash` based on these values,
    /// using `NUM_SYSTEM_PARAMETERS` to differentiate system vs. spatial parameters for hashing.
    /// Dummy gametes are created internally as they are required by the struct but their
    /// specific genetic content is not relevant when `expressed_values` are provided directly.
    ///
    /// # Panics
    /// Panics if `expressed_values.len()` is less than `NUM_SYSTEM_PARAMETERS`.
    pub fn new_for_test(expressed_values: Vec<f64>) -> Self {
        use crate::{
            locus::{
                Locus,
                locus_adjustment::{DirectionOfTravel, LocusAdjustment},
            },
            parameters::parameter::Parameter,
        };
        if expressed_values.len() < NUM_SYSTEM_PARAMETERS {
            panic!(
                "Cannot create test Phenotype: expressed values length {} is less than required {}",
                expressed_values.len(),
                NUM_SYSTEM_PARAMETERS
            );
        }
        let expressed_hash = Self::compute_expressed_hash(&expressed_values, NUM_SYSTEM_PARAMETERS);
        let system_parameters = SystemParameters::new(&expressed_values[0..NUM_SYSTEM_PARAMETERS]);

        // Dummy gametes for test instance, ensuring they have enough loci for expressed_values
        // The actual content of gametes doesn't matter here as `expressed_values` is directly used.
        let min_loci = expressed_values.len().max(NUM_SYSTEM_PARAMETERS);
        // Create dummy Locus objects for Gamete creation
        let dummy_parameter = Parameter::new(0.0);
        let dummy_adjustment =
            LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        let dummy_locus = Locus::new(dummy_parameter, dummy_adjustment, false);
        let dummy_gamete_loci = vec![dummy_locus; min_loci];
        let gamete1 = Gamete::new(dummy_gamete_loci.clone());
        let gamete2 = Gamete::new(dummy_gamete_loci);

        Self {
            gamete1,
            gamete2,
            expressed: expressed_values,
            system_parameters,
            expressed_hash,
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

    /// Returns the expressed parameter values excluding system parameters.
    pub fn expression_problem_values(&self) -> &[f64] {
        if self.expressed.len() < NUM_SYSTEM_PARAMETERS {
            // This case should ideally not happen if Phenotype::new ensures sufficient length.
            // However, returning an empty slice is safer than panicking here.
            &[]
        } else {
            &self.expressed[NUM_SYSTEM_PARAMETERS..]
        }
    }

    /// Returns a reference to the system parameters (m1..m5).
    pub fn system_parameters(&self) -> &SystemParameters {
        &self.system_parameters
    }

    /// Returns all expressed values (including system parameters).
    pub fn expressed_values(&self) -> &[f64] {
        &self.expressed
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
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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
        // Provide at least 9 values for loci to ensure SystemParameters can be derived.
        // The actual values for the first 9 can be arbitrary for this test's purpose,
        // as it only checks gamete storage.
        let g1_loci_values = &[1.0, 2.0, 0.1, 0.5, 0.001, 0.01, 0.1, 100.0, 2.0, 8.0, 9.0]; // 11 loci
        let g2_loci_values = &[3.0, 4.0, 0.1, 0.5, 0.001, 0.01, 0.1, 100.0, 2.0, 10.0, 11.0]; // 11 loci
        let g1 = create_test_gamete(g1_loci_values);
        let g2 = create_test_gamete(g2_loci_values);
        let mut rng = SmallRng::seed_from_u64(0);
        let ph = Phenotype::new(g1.clone(), g2.clone(), &mut rng);
        assert_eq!(ph.gametes(), (&g1, &g2));
        // Assert that expressed values and system parameters are also set.
        // The length of expressed values should match the number of loci in the gametes.
        // This assumes compute_expressed returns one value per locus pair.
        assert_eq!(ph.expressed.len(), g1_loci_values.len()); // Check total expressed length
        let _ = ph.system_parameters(); // Access to ensure it was created without panic

        // Assert that expressed_hash is correctly calculated and set
        let expected_hash =
            Phenotype::compute_expressed_hash(&ph.expressed, crate::NUM_SYSTEM_PARAMETERS);
        assert_eq!(
            ph.expressed_hash(),
            expected_hash,
            "Expressed hash should match the re-calculated hash"
        );
    }
}
