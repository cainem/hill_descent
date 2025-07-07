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

        //dbg!(value);

        let mut score = (value * value * 2.0) + 1.0;

        if score.is_infinite() && score.is_sign_negative() {
            score = f64::MAX
        }
        if score.is_infinite() {
            score = f64::MIN
        }

        //dbg!(score);

        vec![score]
    }
}

#[test]
pub fn execute() {
    let param_range = vec![RangeInclusive::new(-100.0, 100.0)];
    let global_constants = GlobalConstants::new(10, 4);

    let mut world = setup_world(&param_range, global_constants, Box::new(Quadratic));

    println!("{}\n", world.get_state());

    for i in 0..1120 {
        dbg!(i);
        dbg!(world.training_run(&[], &[1.0]));
        println!("{}\n\n", world.get_state());
    }

    let best_organism = world.get_best_organism(&[&[0.0]], &[&[1.0]]);

    dbg!(best_organism.phenotype().expression_problem_values());
}
