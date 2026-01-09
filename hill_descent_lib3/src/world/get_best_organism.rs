//! Get best organism from the world.

use super::World;
use crate::phenotype::Phenotype;
use crate::training_data::TrainingData;
use std::sync::Arc;

/// Data about the best organism in the population.
///
/// This struct provides access to the commonly-needed properties of the best
/// organism without exposing internal implementation details.
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
    /// Runs one training epoch and returns the best organism.
    ///
    /// This method matches the lib2 API signature for compatibility.
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

    /// Returns the ID of the best organism seen so far.
    pub fn get_best_organism_id(&self) -> Option<u64> {
        self.best_organism_id
    }

    /// Returns the best organism's phenotype (low-level access).
    ///
    /// Unlike `get_best_organism`, this does NOT run a training epoch.
    /// Returns `None` if no organism has been evaluated yet.
    pub fn get_best_organism_phenotype(&self) -> Option<(u64, Arc<Phenotype>)> {
        match self.best_organism_id {
            Some(id) => self
                .organisms
                .iter()
                .find(|o| o.read().unwrap().id() == id)
                .map(|o| (id, o.read().unwrap().phenotype().clone())),
            None => None,
        }
    }
}
