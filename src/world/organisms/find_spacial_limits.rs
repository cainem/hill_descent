use super::Organisms;
use std::ops::RangeInclusive;

impl Organisms {
    /// Calculates the spatial limits (min and max values) for each problem-specific dimension
    /// based on the expressed values of all phenotypes in the collection.
    ///
    /// Returns a vector of `RangeInclusive<f64>`, where each range represents
    /// the observed minimum and maximum for a problem-specific dimension.
    ///
    /// If there are no organisms or no problem-specific parameters, an empty vector is returned.
    /// Assumes all phenotypes have the same number of expressed values (dimensions).
    pub fn find_spacial_limits(&self) -> Vec<RangeInclusive<f64>> {
        if self.organisms.is_empty() {
            return Vec::new();
        }

        // Get the first phenotype to determine the number of problem-specific dimensions.
        let first_phenotype_problem_values = self.organisms[0].expression_problem_values();
        if first_phenotype_problem_values.is_empty() {
            // No problem-specific parameters to find limits for.
            return Vec::new();
        }
        let num_problem_dimensions = first_phenotype_problem_values.len();

        // Initialize limits using only problem-specific parameters from the first phenotype.
        let mut limits: Vec<RangeInclusive<f64>> = first_phenotype_problem_values
            .iter()
            .map(|&val| val..=val)
            .collect();

        for phenotype in self.organisms.iter().skip(1) {
            let problem_expressed_values = phenotype.expression_problem_values();
            assert_eq!(
                problem_expressed_values.len(),
                num_problem_dimensions,
                "Inconsistent number of problem dimensions in phenotypes"
            );
            assert_eq!(
                limits.len(),
                num_problem_dimensions,
                "Mismatch between limits and problem dimensions count"
            );

            for (limit_range, &value) in limits.iter_mut().zip(problem_expressed_values.iter()) {
                let current_min = *limit_range.start();
                let current_max = *limit_range.end();
                *limit_range = current_min.min(value)..=current_max.max(value);
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
        // Values: 7 system params + 1 problem param
        let phenotype_vals = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 8.0];
        let phenotype = create_test_phenotype(phenotype_vals, &mut rng);

        // We expect limits only for the problem-specific parameters.
        // Clone phenotype for access if phenotype is moved later.
        let cloned_phenotype = phenotype.clone();
        let expected_problem_expressed_values = cloned_phenotype.expression_problem_values();

        let organisms_collection = super::Organisms::new_from_phenotypes(vec![phenotype]);
        let limits = organisms_collection.find_spacial_limits();

        assert_eq!(limits.len(), expected_problem_expressed_values.len());
        for (i, val) in expected_problem_expressed_values.iter().enumerate() {
            assert_eq!(
                limits[i],
                *val..=*val,
                "Mismatch for problem dimension {}",
                i
            );
        }
    }

    #[test]
    fn given_multiple_organisms_when_find_spacial_limits_then_returns_correct_min_max_ranges() {
        let mut rng_ph1 = StepRng::new(0, 1);
        // System params (first 7) + Problem params (next 3)
        let vals1 = &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 10.0, -5.0];
        let phenotype1 = create_test_phenotype(vals1, &mut rng_ph1);
        let cloned_phenotype1 = phenotype1.clone();
        let problem_expressed1 = cloned_phenotype1.expression_problem_values();

        let mut rng_ph2 = StepRng::new(0, 1);
        let vals2 = &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 2.0, 0.0];
        let phenotype2 = create_test_phenotype(vals2, &mut rng_ph2);
        let cloned_phenotype2 = phenotype2.clone();
        let problem_expressed2 = cloned_phenotype2.expression_problem_values();

        let mut rng_ph3 = StepRng::new(0, 1);
        let vals3 = &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -2.0, 6.0, -2.0];
        let phenotype3 = create_test_phenotype(vals3, &mut rng_ph3);
        let cloned_phenotype3 = phenotype3.clone();
        let problem_expressed3 = cloned_phenotype3.expression_problem_values();

        let organisms_collection =
            super::Organisms::new_from_phenotypes(vec![phenotype1, phenotype2, phenotype3]);
        let limits = organisms_collection.find_spacial_limits();

        let num_problem_dims = problem_expressed1.len();
        assert_eq!(
            problem_expressed2.len(),
            num_problem_dims,
            "Phenotype 2 has different problem dimension count"
        );
        assert_eq!(
            problem_expressed3.len(),
            num_problem_dims,
            "Phenotype 3 has different problem dimension count"
        );
        assert_eq!(
            limits.len(),
            num_problem_dims,
            "Limits have different problem dimension count"
        );

        for i in 0..num_problem_dims {
            let min_val = problem_expressed1[i]
                .min(problem_expressed2[i])
                .min(problem_expressed3[i]);
            let max_val = problem_expressed1[i]
                .max(problem_expressed2[i])
                .max(problem_expressed3[i]);
            assert_eq!(
                limits[i],
                min_val..=max_val,
                "Mismatch for problem dimension {}",
                i
            );
        }
    }
}
