use crate::world::{World, organisms::Organism};

impl World {
    // apply several rounds of training data to find the lowest scoring organism
    // this organism then represents the "best" solution to the problem in accordance with
    // section 5.3.2. of the pdd
    pub fn get_best_organism(
        &self,
        _training_data: &[&[f64]],
        _known_outputs: &[&[f64]],
    ) -> Organism {
        todo!();
    }
}
