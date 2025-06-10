use super::Organisms;
use std::ops::RangeInclusive;

impl Organisms {
    /// Calculates the spatial limits (min and max values) for each dimension
    /// based on the expressed values of all phenotypes in the collection.
    ///
    /// Returns a vector of `RangeInclusive<f64>`, where each range represents
    /// the observed minimum and maximum for a dimension.
    ///
    /// If there are no organisms, an empty vector is returned.
    /// Assumes all phenotypes have the same number of expressed values (dimensions).
    pub fn find_spacial_limits(&self) -> Vec<RangeInclusive<f64>> {
        if self.organisms.is_empty() {
            return Vec::new();
        }

        // Phenotype::new ensures expressed_values.len() >= 7, so num_dimensions will be > 0.
        let num_dimensions = self.organisms[0].expressed_values().len();

        let mut limits: Vec<RangeInclusive<f64>> = self.organisms[0]
            .expressed_values()
            .iter()
            .map(|&val| val..=val)
            .collect();

        for phenotype in self.organisms.iter().skip(1) {
            let expressed_values = phenotype.expressed_values();
            // As per PDD, 'n' (number of dimensions) is fixed for the run.
            assert_eq!(
                expressed_values.len(),
                num_dimensions,
                "Inconsistent number of dimensions in phenotypes"
            );

            for i in 0..num_dimensions {
                // Ensure we are within bounds for expressed_values, though lengths should match.
                if i < expressed_values.len() {
                    let value = expressed_values[i];
                    let current_min = *limits[i].start();
                    let current_max = *limits[i].end();
                    limits[i] = current_min.min(value)..=current_max.max(value);
                }
            }
        }
        limits
    }
}

#[cfg(test)]
mod tests {
    // Required to bring the impl Organisms block into scope for Organisms struct defined in parent mod
    use crate::gamete::Gamete;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameters::parameter::Parameter;
    use crate::phenotype::Phenotype;
    use rand::rngs::mock::StepRng; // For a deterministic RNG

    // Helper to create a Locus (simplified for testing purposes)
    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj_param = Parameter::new(0.0); // Dummy adjustment value
        let adj = LocusAdjustment::new(adj_param, DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    // Helper to create a Gamete
    fn create_test_gamete(vals: &[f64]) -> Gamete {
        let loci = vals.iter().map(|&v| create_test_locus(v)).collect();
        Gamete::new(loci)
    }

    // Helper to create a Phenotype using Phenotype::new
    // Phenotype::new requires at least 7 loci for system parameters.
    fn create_test_phenotype(vals: &[f64], rng: &mut impl rand::Rng) -> Phenotype {
        if vals.len() < 7 {
            // Create a default set of 7 values if not enough are provided, to satisfy Phenotype::new
            let mut complete_vals = vals.to_vec();
            while complete_vals.len() < 7 {
                complete_vals.push(0.0); // Default padding value
            }
            let gamete1 = create_test_gamete(&complete_vals);
            let gamete2 = create_test_gamete(&complete_vals); // Use identical gametes for simplicity
            return Phenotype::new(gamete1, gamete2, rng);
        }
        let gamete1 = create_test_gamete(vals);
        let gamete2 = create_test_gamete(vals); // Use identical gametes for simplicity
        Phenotype::new(gamete1, gamete2, rng)
    }

    #[test]
    fn given_empty_organisms_when_find_spacial_limits_then_returns_empty_vec() {
        let organisms_collection = super::Organisms::new_from_phenotypes(Vec::new());
        let limits = organisms_collection.find_spacial_limits();
        assert!(limits.is_empty());
    }

    #[test]
    fn given_one_organism_when_find_spacial_limits_then_returns_ranges_from_that_organism() {
        let mut rng = StepRng::new(0, 1); // Deterministic RNG
        let phenotype_vals = &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]; // 8 values
        let phenotype = create_test_phenotype(phenotype_vals, &mut rng);

        let expected_expressed_values = phenotype.expressed_values().to_vec();

        let organisms_collection = super::Organisms::new_from_phenotypes(vec![phenotype]);
        let limits = organisms_collection.find_spacial_limits();

        assert_eq!(limits.len(), expected_expressed_values.len());
        for (i, val) in expected_expressed_values.iter().enumerate() {
            assert_eq!(limits[i], *val..=*val, "Mismatch for dimension {}", i);
        }
    }

    #[test]
    fn given_multiple_organisms_when_find_spacial_limits_then_returns_correct_min_max_ranges() {
        let mut rng_ph1 = StepRng::new(0, 1);
        let vals1 = &[1.0, 10.0, -5.0, 0.1, 0.2, 0.3, 0.4]; // 7 values
        let phenotype1 = create_test_phenotype(vals1, &mut rng_ph1);
        let expressed1 = phenotype1.expressed_values().to_vec();

        let mut rng_ph2 = StepRng::new(0, 1);
        let vals2 = &[5.0, 2.0, 0.0, 0.1, 0.2, 0.3, 0.4];
        let phenotype2 = create_test_phenotype(vals2, &mut rng_ph2);
        let expressed2 = phenotype2.expressed_values().to_vec();

        let mut rng_ph3 = StepRng::new(0, 1);
        let vals3 = &[-2.0, 6.0, -2.0, 0.1, 0.2, 0.3, 0.4];
        let phenotype3 = create_test_phenotype(vals3, &mut rng_ph3);
        let expressed3 = phenotype3.expressed_values().to_vec();

        let organisms_collection =
            super::Organisms::new_from_phenotypes(vec![phenotype1, phenotype2, phenotype3]);
        let limits = organisms_collection.find_spacial_limits();

        let num_dims = expressed1.len();
        assert_eq!(
            expressed2.len(),
            num_dims,
            "Phenotype 2 has different dimension count"
        );
        assert_eq!(
            expressed3.len(),
            num_dims,
            "Phenotype 3 has different dimension count"
        );
        assert_eq!(
            limits.len(),
            num_dims,
            "Limits have different dimension count"
        );

        for i in 0..num_dims {
            let min_val = expressed1[i].min(expressed2[i]).min(expressed3[i]);
            let max_val = expressed1[i].max(expressed2[i]).max(expressed3[i]);
            assert_eq!(limits[i], min_val..=max_val, "Mismatch for dimension {}", i);
        }
    }
}
