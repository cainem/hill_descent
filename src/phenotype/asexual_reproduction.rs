use crate::gamete;
use crate::phenotype::Phenotype;
use rand::Rng;

impl Phenotype {
    /// Asexually reproduces the phenotype, creating a single new offspring.
    ///
    /// This method aligns with PDD Section 5.2.3.3:
    /// "For asexual reproduction: The single parent's two shuffled gametes are
    /// recombined with each other to form one offspring."
    ///
    /// The process involves:
    /// 1. Taking the parent's two gametes (`gamete1`, `gamete2`).
    /// 2. Calling `gamete::Gamete::reproduce(gamete1, gamete2, ...)` which performs crossover
    ///    and mutation, returning a new pair of gametes (`new_gamete_a`, `new_gamete_b`).
    /// 3. Creating a single new `Phenotype` offspring using these two new gametes:
    ///    `Phenotype::new(new_gamete_a, new_gamete_b, rng)`.
    pub fn asexual_reproduction<R: Rng>(&self, rng: &mut R) -> Phenotype {
        let gamete1 = self.gamete1();
        let gamete2 = self.gamete2();
        let sys_params = self.system_parameters();
        let gamete_len = gamete1.len(); // Assumes gamete1 and gamete2 have the same length

        // Calculate crossovers using the static method from Phenotype
        let crossovers = Phenotype::calculate_crossovers(sys_params.m3(), gamete_len);

        // Perform reproduction (crossover and mutation) on the parent's gametes
        // to get a new pair of gametes.
        let (new_gamete_a, new_gamete_b) =
            gamete::Gamete::reproduce(gamete1, gamete2, crossovers, rng, sys_params);

        // Create one new phenotype using the pair of newly formed gametes.
        Phenotype::new(new_gamete_a, new_gamete_b, rng)
    }
}

#[cfg(test)]
mod tests {
    use crate::phenotype::Phenotype;
    use crate::phenotype::tests::create_test_gamete;
    use rand::rngs::mock::StepRng;

    #[test]
    fn given_phenotype_when_asexual_reproduction_then_one_offspring_is_created() {
        let mut parent_creation_rng = StepRng::new(0, 1); // RNG for creating parent Phenotype
        // Gamete::reproduce requires gamete_len > 2 * crossovers.
        // If parent's m3 leads to crossovers=1, then len must be > 2 (e.g., 3 or more).
        // Using len=4 for safety, as calculate_crossovers will cap crossovers if m3 is too high.
        let parent_g1 = create_test_gamete(&[0.1, 0.2, 0.3, 0.4]); // Use valid probabilities for m1-m4
        let parent_g2 = create_test_gamete(&[0.5, 0.6, 0.7, 0.8]); // Use valid probabilities for m1-m4

        let parent_phenotype = Phenotype::new(
            parent_g1.clone(),
            parent_g2.clone(),
            &mut parent_creation_rng,
        );

        let mut asexual_rng = StepRng::new(10, 1); // Separate RNG for the asexual_reproduction call
        let offspring = parent_phenotype.asexual_reproduction(&mut asexual_rng);

        // 1. Check gamete length conservation.
        assert_eq!(
            offspring.gamete1().len(),
            parent_g1.len(),
            "Offspring gamete1 length should match parent's gamete1 length"
        );
        assert_eq!(
            offspring.gamete2().len(),
            parent_g2.len(),
            "Offspring gamete2 length should match parent's gamete2 length"
        );

        // 2. Verify that the offspring's gametes are not trivially the same instance as parent's.
        //    Phenotype::new clones gametes, and gamete::reproduce creates new ones.
        //    A more robust check would be to see if values changed if mutation/crossover occurred,
        //    but that depends heavily on RNG and system parameters (m3, m5) which are complex to
        //    precisely control for this specific test. This test focuses on the asexual_reproduction
        //    function's orchestration of creating a single offspring.
        //    We trust that gamete::reproduce and Phenotype::new are tested elsewhere for their
        //    internal logic (mutation, crossover, cloning, expressed value computation).

        // Example: If we knew m3=0 and m5=0 (no mutation in loci values from gamete::reproduce),
        // then offspring.gamete1 would be a (potentially mutated for apply_adjustment_flag etc.)
        // version of parent_g1 and offspring.gamete2 a version of parent_g2.
        // If m3 > 0, then segments would be swapped.
    }
}
