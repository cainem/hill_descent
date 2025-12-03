//! Get best score from the world.

use super::World;

impl World {
    /// Returns the best fitness score seen so far.
    ///
    /// Lower scores are better (minimization problem).
    pub fn get_best_score(&self) -> f64 {
        self.best_score
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_with_evaluations_when_get_best_score_then_returns_minimum() {
        todo!()
    }
}
