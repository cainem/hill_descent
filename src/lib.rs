pub mod gamete;
pub mod locus;
pub mod parameters;
pub mod phenotype;
pub mod world;

pub const NUM_SYSTEM_PARAMETERS: usize = 7;
pub const E0: f64 = f64::MIN_POSITIVE;

use std::ops::RangeInclusive;

pub use gamete::Gamete;
pub use locus::Locus;
use parameters::GlobalConstants;
pub use phenotype::Phenotype;
use world::World;

// this will take a list of parameters and return a world
pub fn setup_world(_params: &[RangeInclusive<f64>], _global_constants: GlobalConstants) -> World {
    todo!()
}

// this will need sample data
pub fn run_world(_world: &mut World) {
    todo!()
}
