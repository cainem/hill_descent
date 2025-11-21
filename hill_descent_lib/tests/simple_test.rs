use std::ops::RangeInclusive;

use hill_descent_lib::{
    TrainingData, parameters::GlobalConstants, setup_world,
    world::single_valued_function::SingleValuedFunction,
};

#[derive(Debug)]
pub struct Quadratic;

impl SingleValuedFunction for Quadratic {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        // there should be no input.
        // There is only one phenotype_expressed_value
        // return that operated on by the function

        // function 2 (x + 13)^2 + 1

        assert_eq!(1, phenotype_expressed_values.len());

        let value = phenotype_expressed_values[0];

        let mut score = ((value + 13.0) * (value + 13.0) * 2.0) + 1.0;

        if score.is_infinite() && score.is_sign_negative() {
            score = f64::MAX
        }
        if score.is_infinite() {
            score = f64::MIN
        }

        score
    }

    fn function_floor(&self) -> f64 {
        1.0 // Minimum value of 2(x+13)^2 + 1 is 1.0
    }
}

#[test]
#[ignore]
pub fn execute() {
    let param_range = vec![RangeInclusive::new(-100.0, 100.0)];
    let global_constants = GlobalConstants::new(10, 4);

    let mut world = setup_world(&param_range, global_constants, Box::new(Quadratic));

    println!("{}\n", world.get_state());

    for i in 0..1200 {
        dbg!(i);
        // Objective-function mode: use TrainingData::None with function floor
        dbg!(world.training_run(TrainingData::None {
            floor_value: Quadratic.function_floor()
        }));
        println!("{}\n\n", world.get_state());
    }

    let best_organism = world.get_best_organism(TrainingData::None {
        floor_value: Quadratic.function_floor(),
    });

    dbg!(best_organism.phenotype().expression_problem_values());
}
