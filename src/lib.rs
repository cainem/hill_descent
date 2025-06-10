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
pub fn setup_world<F>(
    _params: &[RangeInclusive<f64>],
    _global_constants: GlobalConstants,
    // The new function parameter
    function: F,
) -> World<F>
where
    // The trait bound specifying the function's signature
    F: Fn(&[f64]) -> Vec<f64>,
{
    let _world = World::new(_params, _global_constants, function);

    todo!()
}

// this will need sample data
pub fn training_run<F>(_world: &mut World<F>)
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    todo!()
}

// pass in a sample of training runs and return the best parameters
pub fn get_best_parameters<F>(_world: &World<F>) -> Vec<f64>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    todo!()
}
