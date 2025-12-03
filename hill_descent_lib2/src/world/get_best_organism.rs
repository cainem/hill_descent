//! Get best organism from the world.

use super::World;

impl World {
    /// Returns the ID of the best organism.
    ///
    /// # Returns
    ///
    /// The ID of the organism with the best fitness score, or None if
    /// no evaluations have occurred.
    pub fn get_best_organism_id(&self) -> Option<u64> {
        self.best_organism_id
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_with_evaluations_when_get_best_organism_id_then_returns_id() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_without_evaluations_when_get_best_organism_id_then_returns_none() {
        todo!()
    }
}
