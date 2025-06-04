use crate::phenotype::Phenotype;
use rand::Rng;

impl Phenotype {
    /// Performs sexual reproduction between two parent phenotypes to produce two offspring phenotypes.
    ///
    /// Each parent undergoes a simulated meiosis (`Gamete::reproduce`) using its own two gametes
    /// and its system parameters. The number of crossovers is determined by its `m3` value.
    /// Two offspring are then formed by combining one meiotic product from each parent.
    ///
    /// # Arguments
    /// * `parent1`: A reference to the first parent phenotype.
    /// * `parent2`: A reference to the second parent phenotype.
    /// * `rng`: A mutable reference to a random number generator.
    ///
    /// # Returns
    /// A tuple `(Phenotype, Phenotype)` containing the two new offspring.
    ///
    /// # Panics
    /// * If `parent1` and `parent2` have gametes of different lengths.
    /// * Propagates panics from `Gamete::reproduce` if:
    ///     * Gametes within a single parent have different lengths.
    ///     * Any gamete length is 0.
    ///     * `gamete_len <= 2 * crossovers` (though `calculate_crossovers` tries to prevent this for `gamete_len > 0`).
    pub fn sexual_reproduction<R: Rng>(
        parent1: &Phenotype,
        parent2: &Phenotype,
        rng: &mut R,
    ) -> (Phenotype, Phenotype) {
        let p1_g1 = parent1.gamete1();
        let p1_g2 = parent1.gamete2();
        let p2_g1 = parent2.gamete1();
        let p2_g2 = parent2.gamete2();

        // Ensure parent gametes are compatible for forming offspring phenotypes.
        // Gamete::reproduce will handle internal consistency for each parent's meiosis.
        assert_eq!(
            p1_g1.len(),
            p2_g1.len(),
            "Parent gametes must have the same length for sexual reproduction."
        );
        let gamete_len = p1_g1.len(); // Common gamete length

        let sys_params1 = parent1.system_parameters();
        let sys_params2 = parent2.system_parameters();

        let crossovers1 = Phenotype::calculate_crossovers(sys_params1.m3(), gamete_len);
        let (meiotic_g1_p1, meiotic_g2_p1) =
            crate::gamete::reproduce(p1_g1, p1_g2, crossovers1, rng, sys_params1);

        let crossovers2 = Phenotype::calculate_crossovers(sys_params2.m3(), gamete_len);
        let (meiotic_g1_p2, meiotic_g2_p2) =
            crate::gamete::reproduce(p2_g1, p2_g2, crossovers2, rng, sys_params2);

        let offspring1 = Phenotype::new(meiotic_g1_p1, meiotic_g1_p2, rng);
        let offspring2 = Phenotype::new(meiotic_g2_p1, meiotic_g2_p2, rng);

        (offspring1, offspring2)
    }
}

#[cfg(test)]
mod tests {
    use crate::gamete::Gamete;
    use crate::locus::Locus;
    use crate::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameter::Parameter;
    use crate::phenotype::Phenotype; // For Phenotype::new in helpers
    use crate::system_parameters::SystemParameters;
    use rand::Rng;
    use rand::rngs::mock::StepRng; // For Rng trait in helper signature

    fn helper_create_test_locus(val: f64) -> Locus {
        Locus::new(
            Parameter::new(val),
            LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false),
            false,
        )
    }

    fn helper_create_test_gamete(vals: &[f64]) -> Gamete {
        Gamete::new(vals.iter().map(|&v| helper_create_test_locus(v)).collect())
    }

    fn helper_create_test_phenotype(
        g1_vals: &[f64],
        g2_vals: &[f64],
        rng: &mut impl Rng,
    ) -> Phenotype {
        Phenotype::new(
            helper_create_test_gamete(g1_vals),
            helper_create_test_gamete(g2_vals),
            rng,
        )
    }

    #[test]
    fn given_parents_with_zero_m_values_when_reproduce_then_offspring_inherit_parental_gametes() {
        let mut rng = StepRng::new(0, 1);
        let p_vals = vec![0.0; 5]; // For SystemParameters m_i to be 0.0

        let parent1 = helper_create_test_phenotype(&p_vals, &p_vals, &mut rng);
        let parent2 = helper_create_test_phenotype(&p_vals, &p_vals, &mut rng);

        // Verify test setup: all system parameters should be 0.0
        let zero_sys_params = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            parent1.system_parameters(),
            &zero_sys_params,
            "P1 sys params mismatch"
        );
        assert_eq!(
            parent2.system_parameters(),
            &zero_sys_params,
            "P2 sys params mismatch"
        );

        let (offspring1, offspring2) = Phenotype::sexual_reproduction(&parent1, &parent2, &mut rng);

        // With 0 crossovers and 0 mutation rates, meiotic gametes = original parental gametes.
        // O1 gets (P1.G1, P2.G1), O2 gets (P1.G2, P2.G2)
        assert_eq!(
            offspring1.gamete1().loci(),
            parent1.gamete1().loci(),
            "O1.G1 != P1.G1"
        );
        assert_eq!(
            offspring1.gamete2().loci(),
            parent2.gamete1().loci(),
            "O1.G2 != P2.G1"
        );
        assert_eq!(
            offspring2.gamete1().loci(),
            parent1.gamete2().loci(),
            "O2.G1 != P1.G2"
        );
        assert_eq!(
            offspring2.gamete2().loci(),
            parent2.gamete2().loci(),
            "O2.G2 != P2.G2"
        );
    }

    #[test]
    #[should_panic(expected = "Parent gametes must have the same length for sexual reproduction.")]
    fn given_parents_with_mismatched_gamete_lengths_when_reproduce_then_panics() {
        let mut rng = StepRng::new(0, 1);
        let p1 = helper_create_test_phenotype(&[0.0; 3], &[0.0; 3], &mut rng);
        let p2 = helper_create_test_phenotype(&[0.0; 2], &[0.0; 2], &mut rng);
        Phenotype::sexual_reproduction(&p1, &p2, &mut rng);
    }

    #[test]
    #[should_panic(expected = "Number of crossovers must satisfy len > 2 * crossovers")]
    fn given_parent_with_zero_length_gametes_when_reproduce_then_panics() {
        let mut rng = StepRng::new(0, 1);
        let empty_gamete = Gamete::new(vec![]);
        // Phenotype::new with empty gametes will lead to SystemParameters with m_i=0.
        // calculate_crossovers(0.0, 0) will return 0.
        // Gamete::reproduce with len=0, crossovers=0 will panic.
        let parent1 = Phenotype::new(empty_gamete.clone(), empty_gamete.clone(), &mut rng);
        let parent2 = Phenotype::new(empty_gamete.clone(), empty_gamete.clone(), &mut rng);
        Phenotype::sexual_reproduction(&parent1, &parent2, &mut rng);
    }
}
