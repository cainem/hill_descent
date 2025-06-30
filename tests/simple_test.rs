use std::ops::RangeInclusive;

use hill_descent::{
    parameters::GlobalConstants, setup_world, world::world_function::WorldFunction,
};

#[derive(Debug)]
pub struct Quadratic;

impl WorldFunction for Quadratic {
    fn run(&self, phenotype_expressed_values: &[f64], _inputs: &[f64]) -> Vec<f64> {
        // there should be no input.
        // There is only one phenotype_expressed_value
        // return that operated on by the function

        // function 2 x^2 + 1

        assert_eq!(1, phenotype_expressed_values.len());

        let value = phenotype_expressed_values[0];

        vec![(1.0 / ((value * value) * 2.0) + 1.0)]
    }
}

#[test]
pub fn execute() {
    let param_range = vec![RangeInclusive::new(f64::MIN/2.0, f64::MAX/2.0)];
    let global_constants = GlobalConstants::new(10, 4);

    let mut world = setup_world(&param_range, global_constants, Box::new(Quadratic));

    for _i in 0..999 {
        dbg!(world.training_run(&[], &[0.0]));
    }
}
