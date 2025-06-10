use super::World;

impl World {
    pub fn training_run(&mut self, inputs: &[f64], known_outputs: &[f64]) -> f64 {
        // update the regions based on the new population

        // run function with the input for each phenotype and update the phenotypes last score

        // update the min known scores for the regions based on the scores of the organisms

        // update the carrying capacities of the known region

        // reproduce the organisms based populations and carrying capacities of the regions

        todo!();
    }
}
