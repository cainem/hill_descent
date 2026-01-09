//! Get best organism from the world.

use super::World;
use crate::phenotype::Phenotype;
use std::sync::Arc;

impl World {
    /// Returns the best organism seen so far (ID and Phenotype).
    pub fn get_best_organism(&self) -> Option<(u64, Arc<Phenotype>)> {
        match self.best_organism_id {
            Some(id) => self
                .organisms
                .iter()
                .find(|o| o.read().unwrap().id() == id)
                .map(|o| (id, o.read().unwrap().phenotype().clone())),
            None => None,
        }
    }

    /// Returns the ID of the best organism seen so far.
    pub fn get_best_organism_id(&self) -> Option<u64> {
        self.best_organism_id
    }
}
