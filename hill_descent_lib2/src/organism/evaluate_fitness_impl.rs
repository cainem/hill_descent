//! Fitness evaluation implementation for organisms.
//!
//! Evaluates the organism's fitness by passing its expressed phenotype values
//! through the world function.

use std::sync::Arc;

use crate::{
    phenotype::Phenotype,
    world::{WorldFunction, regions::region_key::RegionKey},
};

use super::EvaluateFitnessResult;

/// Evaluates the organism's fitness using the world function.
///
/// # Arguments
///
/// * `phenotype` - The organism's genetic material
/// * `world_function` - The fitness evaluation function
/// * `region_key` - The organism's current region key
/// * `age` - The organism's current age
/// * `training_data_index` - Index into shared training data (ignored for function optimization)
///
/// # Returns
///
/// Tuple of (EvaluateFitnessResult, calculated score for caching).
///
/// # Algorithm
///
/// 1. Extract expressed values from phenotype
/// 2. Call world_function.run() with expressed values
/// 3. Calculate fitness score from outputs
/// 4. Return result with score, age, and region key
pub fn evaluate_fitness(
    _phenotype: &Arc<Phenotype>,
    _world_function: &Arc<dyn WorldFunction + Send + Sync>,
    _region_key: &RegionKey,
    _age: usize,
    _training_data_index: usize,
) -> (EvaluateFitnessResult, f64) {
    todo!("Stage 3: Implement fitness evaluation")
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending - Stage 3"]
    fn given_organism_when_evaluate_fitness_then_returns_correct_score() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending - Stage 3"]
    fn given_organism_when_evaluate_fitness_then_result_contains_age() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending - Stage 3"]
    fn given_organism_when_evaluate_fitness_then_result_contains_region_key() {
        todo!()
    }
}
