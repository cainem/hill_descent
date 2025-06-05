pub mod gamete;
pub mod locus;
pub mod parameters;
pub mod phenotype;
pub mod world;

pub use gamete::Gamete;
pub use locus::Locus;
pub use phenotype::Phenotype;
use world::World;

use crate::parameters::parameter::Parameter;

// this will take a list of parameters and return a world
pub fn setup_world(_params: Vec<Parameter>) -> World {
    todo!()
}

// this will need sample data
pub fn run_world(_world: &mut World) {
    todo!()
}
