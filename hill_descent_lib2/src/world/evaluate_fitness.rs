//! Evaluate fitness for all organisms.

use super::World;

impl World {
    /// Evaluates fitness for all organisms.
    ///
    /// Sends EvaluateFitnessRequest to all organisms and collects responses.
    /// Updates best_score and best_organism_id if a better score is found.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Input values for fitness function
    /// * `known_outputs` - Known output values for error calculation
    pub fn evaluate_fitness(&mut self, inputs: &[f64], known_outputs: &[f64]) {
        todo!("Implement evaluate_fitness")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_when_evaluate_fitness_then_all_organisms_scored() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_when_evaluate_fitness_then_best_score_updated() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_world_when_evaluate_fitness_then_best_organism_id_updated() {
        todo!()
    }
}
