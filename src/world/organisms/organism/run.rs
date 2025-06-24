use crate::world::{organisms::Organism, world_function::WorldFunction};

impl Organism {
    /// Runs the organism's phenotype with the provided function and inputs.
    ///
    /// This method executes the organism's phenotype using the specified world function
    /// and inputs, and updates the organism's score based on the outputs compared to known outputs.
    pub fn run(&mut self, function: &dyn WorldFunction, inputs: &[f64], known_outputs: &[f64]) {
        // Run the world function with the input for each phenotype
        let phenotype = self.get_phenotype_rc();
        let phenotype_expressed_values = phenotype.expression_problem_values();
        let outputs = function.run(&phenotype_expressed_values, inputs);

        // Evaluate outputs against known outputs
        // to determine the fitness of the phenotype in accordance with the pdd.md
        // Update the organism's score accordingly
    }
}
