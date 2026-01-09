//! Get best score from the world.

use super::World;

impl World {
    /// Returns the best fitness score seen so far.
    pub fn get_best_score(&self) -> f64 {
        self.best_score
    }
}
