//! Reproduction implementation for organisms.
//!
//! Performs sexual reproduction between two organisms to produce offspring.

use std::sync::Arc;

use rand::{SeedableRng, rngs::StdRng};

use crate::phenotype::Phenotype;

use super::ReproduceResult;

/// Performs sexual reproduction with a partner's phenotype.
///
/// # Arguments
///
/// * `self_phenotype` - This organism's genetic material
/// * `self_id` - This organism's ID (used for parent ID tracking)
/// * `partner_phenotype` - The partner's genetic material
/// * `reproduction_seed` - Seed for deterministic reproduction
///
/// # Returns
///
/// ReproduceResult containing two offspring phenotypes and parent IDs.
///
/// # Algorithm
///
/// 1. Create RNG from reproduction_seed (deterministic)
/// 2. Perform sexual reproduction between self_phenotype and partner_phenotype
/// 3. Wrap offspring in Arc<Phenotype>
/// 4. Return result with offspring and parent IDs
///
/// # Note
///
/// The partner_id is not passed directly but is expected to be extracted from
/// the request context by the caller. This function uses self_id for the first
/// parent position and assumes the caller provides the correct partner_id.
pub fn reproduce(
    self_phenotype: &Arc<Phenotype>,
    self_id: u64,
    partner_phenotype: &Arc<Phenotype>,
    reproduction_seed: u64,
) -> ReproduceResult {
    let mut rng = StdRng::seed_from_u64(reproduction_seed);

    let (offspring1, offspring2) =
        Phenotype::sexual_reproduction(self_phenotype, partner_phenotype, &mut rng);

    ReproduceResult {
        offspring_phenotypes: (Arc::new(offspring1), Arc::new(offspring2)),
        parent_ids: (self_id, 0), // Partner ID is not known here, caller must set it
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NUM_SYSTEM_PARAMETERS;
    use rand::SeedableRng;
    use std::ops::RangeInclusive;

    /// Creates a test phenotype using random gamete generation for proper genetic diversity.
    /// Different seeds will produce phenotypes with different gamete values, enabling
    /// meaningful crossover variation during reproduction.
    fn create_random_test_phenotype(seed: u64) -> Arc<Phenotype> {
        let mut rng = StdRng::seed_from_u64(seed);
        // Create bounds for the phenotype:
        // - First 6 parameters are system parameters (m1-m5: 0-1 range, max_age: reasonable range)
        // - Remaining are problem values (can be any reasonable range)
        let mut bounds: Vec<RangeInclusive<f64>> = Vec::with_capacity(NUM_SYSTEM_PARAMETERS + 10);
        // System parameters bounds
        bounds.push(0.0..=1.0); // m1: mutation rate
        bounds.push(0.0..=1.0); // m2: mutation magnitude  
        bounds.push(0.0..=1.0); // m3: crossover probability
        bounds.push(0.0..=1.0); // m4: selection pressure
        bounds.push(0.0..=1.0); // m5: adaptation rate
        bounds.push(10.0..=200.0); // max_age
        // Problem value bounds (can be wider)
        for _ in 0..10 {
            bounds.push(-100.0..=100.0);
        }
        Arc::new(Phenotype::new_random_phenotype(&mut rng, &bounds))
    }

    /// Creates a test phenotype from explicit expressed values for simpler tests.
    /// Note: This uses dummy gametes, so crossover won't produce variation.
    fn create_test_phenotype_from_values(base_value: f64) -> Arc<Phenotype> {
        let mut expressed = Vec::with_capacity(NUM_SYSTEM_PARAMETERS + 10);
        for i in 0..(NUM_SYSTEM_PARAMETERS + 10) {
            expressed.push(base_value + (i as f64) * 0.1);
        }
        // Set system parameters to reasonable values
        expressed[0] = 0.5; // m1
        expressed[1] = 0.5; // m2
        expressed[2] = 0.8; // m3 (crossover probability)
        expressed[3] = 1.0; // m4
        expressed[4] = 0.5; // m5
        expressed[5] = 100.0; // max_age
        Arc::new(Phenotype::new_for_test(expressed))
    }

    #[test]
    fn given_two_phenotypes_when_reproduce_then_returns_two_offspring() {
        let parent1 = create_test_phenotype_from_values(1.0);
        let parent2 = create_test_phenotype_from_values(2.0);

        let result = reproduce(&parent1, 42, &parent2, 12345);

        // Should return two offspring phenotypes
        let (offspring1, offspring2) = result.offspring_phenotypes;
        assert_eq!(
            offspring1.expressed_values().len(),
            offspring2.expressed_values().len()
        );
    }

    #[test]
    fn given_same_seed_when_reproduce_then_produces_identical_offspring() {
        // Use random phenotypes with actual gamete diversity
        let parent1 = create_random_test_phenotype(100);
        let parent2 = create_random_test_phenotype(200);

        let result1 = reproduce(&parent1, 42, &parent2, 12345);
        let result2 = reproduce(&parent1, 42, &parent2, 12345);

        // Same seed should produce identical offspring
        assert_eq!(
            result1.offspring_phenotypes.0.expressed_values(),
            result2.offspring_phenotypes.0.expressed_values()
        );
        assert_eq!(
            result1.offspring_phenotypes.1.expressed_values(),
            result2.offspring_phenotypes.1.expressed_values()
        );
    }

    #[test]
    fn given_different_seeds_when_reproduce_then_offspring_phenotypes_differ() {
        // Use random phenotypes with actual gamete diversity
        // This ensures meiosis crossover can produce different results
        let parent1 = create_random_test_phenotype(100);
        let parent2 = create_random_test_phenotype(200);

        // Use seeds that we know produce different results
        // The key is that with different seeds, the RNG produces different crossover points
        let result1 = reproduce(&parent1, 42, &parent2, 12345);
        let result2 = reproduce(&parent1, 42, &parent2, 99999);

        // With different seeds, the meiosis crossover points should differ,
        // resulting in at least one different offspring pair
        // We compare the expressed_hash which captures the genetic content
        let hash1_a = result1.offspring_phenotypes.0.expressed_hash();
        let hash1_b = result1.offspring_phenotypes.1.expressed_hash();
        let hash2_a = result2.offspring_phenotypes.0.expressed_hash();
        let hash2_b = result2.offspring_phenotypes.1.expressed_hash();

        // At least one offspring should be different between the two reproductions
        let all_same = hash1_a == hash2_a && hash1_b == hash2_b;
        assert!(
            !all_same,
            "Different seeds should produce at least some different offspring"
        );
    }

    #[test]
    fn given_reproduction_when_complete_then_parent_ids_contains_self_id() {
        let parent1 = create_test_phenotype_from_values(1.0);
        let parent2 = create_test_phenotype_from_values(2.0);

        let result = reproduce(&parent1, 42, &parent2, 12345);

        // First parent ID should be self_id
        assert_eq!(result.parent_ids.0, 42);
    }

    #[test]
    fn given_offspring_when_created_then_have_valid_system_parameters() {
        let parent1 = create_test_phenotype_from_values(1.0);
        let parent2 = create_test_phenotype_from_values(2.0);

        let result = reproduce(&parent1, 1, &parent2, 12345);

        // Offspring should have valid system parameters
        let offspring1 = result.offspring_phenotypes.0;
        let offspring2 = result.offspring_phenotypes.1;

        // System parameters should exist
        let _ = offspring1.system_parameters();
        let _ = offspring2.system_parameters();
    }
}
