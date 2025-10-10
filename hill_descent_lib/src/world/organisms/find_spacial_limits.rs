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
        let first_phenotype = &self.organisms[0];
        let first_phenotype_problem_values =
            first_phenotype.phenotype().expression_problem_values();
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
            let problem_expressed_values = phenotype.phenotype().expression_problem_values();
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
    use crate::gamete::Gamete;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameters::parameter::Parameter;
    use crate::phenotype::Phenotype;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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
    // Phenotype::new requires at least 9 loci for system parameters.
    fn create_test_phenotype(vals: &[f64], rng: &mut impl rand::Rng) -> Phenotype {
        if vals.len() < 9 {
            // Create a default set of 9 values if not enough are provided, to satisfy Phenotype::new
            let mut complete_vals = vals.to_vec();
            while complete_vals.len() < 9 {
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
        let mut rng = SmallRng::seed_from_u64(0); // Deterministic RNG
        // Values: 9 system params + 1 problem param
        let phenotype_vals = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 8.0];
        let phenotype = create_test_phenotype(phenotype_vals, &mut rng);

        // We expect limits only for the problem-specific parameters.
        // Clone phenotype for access if phenotype is moved later.
        let cloned_phenotype = phenotype.clone();
        let expected_problem_expressed_values = cloned_phenotype.expression_problem_values();

        let organisms_collection = super::Organisms::new_from_phenotypes(vec![phenotype]);
        let limits = organisms_collection.find_spacial_limits();

        assert_eq!(
            limits.len(),
            expected_problem_expressed_values.len(),
            "Mismatch in number of dimensions"
        );

        for (limit, &expected_val) in limits.iter().zip(expected_problem_expressed_values.iter()) {
            assert_eq!(*limit.start(), expected_val);
            assert_eq!(*limit.end(), expected_val);
        }
    }

    #[test]
    fn given_multiple_organisms_when_find_spacial_limits_then_returns_correct_min_max_ranges() {
        let mut rng = SmallRng::seed_from_u64(0); // Deterministic RNG

        // Phenotype 1: 9 system params + 2 problem params
        let phenotype1_vals = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 10.0, 200.0];
        let phenotype1 = create_test_phenotype(phenotype1_vals, &mut rng);

        // Phenotype 2: 9 system params + 2 problem params
        let phenotype2_vals = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 5.0, 250.0];
        let phenotype2 = create_test_phenotype(phenotype2_vals, &mut rng);

        // Phenotype 3: 9 system params + 2 problem params
        let phenotype3_vals = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 15.0, 150.0];
        let phenotype3 = create_test_phenotype(phenotype3_vals, &mut rng);

        let organisms_collection =
            super::Organisms::new_from_phenotypes(vec![phenotype1, phenotype2, phenotype3]);
        let limits = organisms_collection.find_spacial_limits();

        assert_eq!(limits.len(), 2, "Expected limits for 2 problem dimensions");

        // Expected limits for dimension 1: min(10.0, 5.0, 15.0) = 5.0, max(10.0, 5.0, 15.0) = 15.0
        // Expected limits for dimension 2: min(200.0, 250.0, 150.0) = 150.0, max(200.0, 250.0, 150.0) = 250.0
        // Note: The exact expressed values depend on the `compute_expressed` logic,
        // which combines gametes. For simplicity, we assume the test helper `create_test_phenotype`
        // results in expressed values that are directly comparable to the input `vals`.
        // A more robust test would mock or pre-calculate the exact expressed values.

        // Assuming the expressed values are close to the input values for this test.
        // Let's find the actual expressed values to be precise.
        let phenotype1_for_expr =
            create_test_phenotype(phenotype1_vals, &mut SmallRng::seed_from_u64(0));
        let p1_expressed = phenotype1_for_expr.expression_problem_values();
        let phenotype2_for_expr =
            create_test_phenotype(phenotype2_vals, &mut SmallRng::seed_from_u64(0));
        let p2_expressed = phenotype2_for_expr.expression_problem_values();
        let phenotype3_for_expr =
            create_test_phenotype(phenotype3_vals, &mut SmallRng::seed_from_u64(0));
        let p3_expressed = phenotype3_for_expr.expression_problem_values();

        let expected_min_dim1 = p1_expressed[0].min(p2_expressed[0]).min(p3_expressed[0]);
        let expected_max_dim1 = p1_expressed[0].max(p2_expressed[0]).max(p3_expressed[0]);
        let expected_min_dim2 = p1_expressed[1].min(p2_expressed[1]).min(p3_expressed[1]);
        let expected_max_dim2 = p1_expressed[1].max(p2_expressed[1]).max(p3_expressed[1]);

        assert_eq!(*limits[0].start(), expected_min_dim1);
        assert_eq!(*limits[0].end(), expected_max_dim1);
        assert_eq!(*limits[1].start(), expected_min_dim2);
        assert_eq!(*limits[1].end(), expected_max_dim2);
    }
}
