//! Get best parameters from the world.

use super::World;

impl World {
    /// Returns the expressed parameter values of the best organism.
    pub fn get_best_params(&self) -> Vec<f64> {
        self.best_params.clone()
    }
}
