//! Get best parameters from the world.

use super::World;

impl World {
    /// Returns the expressed parameter values of the best organism.
    ///
    /// # Returns
    ///
    /// The parameter values (excluding system parameters) of the organism
    /// with the best fitness score.
    pub fn get_best_params(&self) -> Vec<f64> {
        todo!("Implement get_best_params")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_with_evaluations_when_get_best_params_then_returns_best_organism_params() {
        todo!()
    }
}
