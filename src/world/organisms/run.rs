use crate::world::{organisms::Organisms, world_function::WorldFunction};

impl Organisms {
    pub fn run(&mut self, function: &dyn WorldFunction, inputs: &[f64], known_outputs: &[f64]) {
        // run the world function with the input for each phenotype
        for organism in self.iter_mut() {
            organism.run(function, inputs, known_outputs);
        }
    }
}
