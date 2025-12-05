//! Get best organism from the world.

use super::World;
use crate::TrainingData;

/// Data about the best organism in the population.
///
/// This struct provides access to the commonly-needed properties of the best
/// organism without exposing the internal thread pool architecture.
///
/// # Note
///
/// Unlike lib1 which returns an `Arc<Organism>`, lib2 returns this lightweight
/// struct containing the organism's data. This is because organisms in lib2
/// live inside a thread pool and cannot be directly shared.
#[derive(Debug, Clone)]
pub struct BestOrganism {
    id: u64,
    score: f64,
    params: Vec<f64>,
}

impl BestOrganism {
    /// Returns the unique ID of this organism.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the fitness score of this organism.
    ///
    /// Lower scores are better (minimization).
    pub fn score(&self) -> Option<f64> {
        if self.score >= f64::MAX * 0.99999 {
            None
        } else {
            Some(self.score)
        }
    }

    /// Returns the raw score value (for compatibility).
    pub fn raw_score(&self) -> f64 {
        self.score
    }

    /// Returns the problem parameters (expressed phenotype values).
    ///
    /// These are the optimized parameter values, excluding system parameters
    /// like mutation rates and max_age.
    pub fn params(&self) -> &[f64] {
        &self.params
    }

    /// Returns the problem parameters as a Vec (for compatibility).
    pub fn problem_parameters(&self) -> Vec<f64> {
        self.params.clone()
    }
}

impl World {
    /// Returns the ID of the best organism.
    ///
    /// # Returns
    ///
    /// The ID of the organism with the best fitness score, or `None` if
    /// no evaluations have occurred yet.
    pub fn get_best_organism_id(&self) -> Option<u64> {
        self.best_organism_id
    }

    /// Runs one training epoch and returns the best organism.
    ///
    /// This method matches the lib1 API signature for compatibility.
    /// It runs a complete training run, then returns data about the
    /// best-scoring organism.
    ///
    /// # Arguments
    ///
    /// * `data` - Training data for the epoch
    ///
    /// # Returns
    ///
    /// A `BestOrganism` containing the ID, score, and parameters of the
    /// best organism after training.
    ///
    /// # Panics
    ///
    /// Panics if no organisms have been evaluated (empty population).
    pub fn get_best_organism(&mut self, data: TrainingData) -> BestOrganism {
        // Run one training epoch
        self.training_run(data);

        // Return data about the best organism
        BestOrganism {
            id: self
                .best_organism_id
                .expect("Population contains no scored organisms"),
            score: self.best_score,
            params: self.best_params.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobalConstants, world::single_valued_function::SingleValuedFunction};
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_world_without_evaluations_when_get_best_organism_id_then_returns_none() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let world = World::new(&bounds, constants, Box::new(SumOfSquares));

        assert!(world.get_best_organism_id().is_none());
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_organism_id_then_returns_id() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Run training to evaluate organisms
        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        // Should have a best organism ID
        assert!(world.get_best_organism_id().is_some());
    }

    #[test]
    fn given_world_with_evaluations_when_get_best_organism_id_then_id_is_valid() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let training_data = TrainingData::None { floor_value: 0.0 };
        world.training_run(training_data);

        let best_id = world.get_best_organism_id().unwrap();
        // The best organism ID should be in the current population
        assert!(world.organism_ids.contains(&best_id));
    }

    #[test]
    fn given_world_when_get_best_organism_then_runs_training_and_returns_data() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(50, 5, 123);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let best = world.get_best_organism(TrainingData::None { floor_value: 0.0 });

        // Should have a valid score
        assert!(best.score().is_some());
        // Should have correct number of params
        assert_eq!(best.params().len(), 2);
        // Params should be within bounds
        for &p in best.params() {
            assert!((-10.0..=10.0).contains(&p));
        }
    }

    #[test]
    fn given_world_when_get_best_organism_then_score_matches_params() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(100, 10, 456);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let best = world.get_best_organism(TrainingData::None { floor_value: 0.0 });

        // Verify params produce the score
        let computed: f64 = best.params().iter().map(|x| x * x).sum();
        assert!(
            (computed - best.raw_score()).abs() < 1e-10,
            "Computed {} should match score {}",
            computed,
            best.raw_score()
        );
    }

    #[test]
    fn given_best_organism_when_problem_parameters_then_returns_clone() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(30, 3, 789);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        let best = world.get_best_organism(TrainingData::None { floor_value: 0.0 });
        let params = best.problem_parameters();

        assert_eq!(params, best.params());
    }
}
