use hill_descent_lib::{
    parameters::global_constants::GlobalConstants,
    world::{world_function::WorldFunction, World},
};
use serde_json::Value;
use std::ops::RangeInclusive;

#[derive(Debug)]
struct VariableFn;

impl WorldFunction for VariableFn {
    fn run(&self, params: &[f64], _vars: &[f64]) -> Vec<f64> {
        // Return varied outputs to trigger reproduction
        vec![params[0] * params[1] + 0.1] 
    }
}

fn get_organism_ids_from_world(world: &World) -> Vec<usize> {
    let json_str = world.get_state_for_web();
    let parsed: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    
    parsed["organisms"]
        .as_array()
        .expect("organisms should be an array")
        .iter()
        .map(|organism| organism["id"].as_u64().expect("id should be a number") as usize)
        .collect()
}

#[test]
fn test_organism_ids_persist_across_epochs() {
    // Arrange: Create a small world to easily track organisms
    let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, 0.0..=1.0];
    let gc = GlobalConstants::new(10, 4); // Small population
    let mut world = World::new(&bounds, gc, Box::new(VariableFn));

    let inputs = vec![0.0, 0.0];
    let known_outputs = vec![1.0];

    // Capture initial organism IDs
    let initial_ids = get_organism_ids_from_world(&world);
    println!("Initial organism IDs: {:?}", initial_ids);

    // Act: Run one training epoch
    world.training_run(&inputs, &known_outputs);

    // Capture IDs after first epoch
    let after_epoch_ids = get_organism_ids_from_world(&world);
    println!("After epoch 1 IDs: {:?}", after_epoch_ids);

    // Run second epoch
    world.training_run(&inputs, &known_outputs);

    // Capture IDs after second epoch
    let after_epoch2_ids = get_organism_ids_from_world(&world);
    println!("After epoch 2 IDs: {:?}", after_epoch2_ids);

    // Assert: Check if any original organisms persisted
    let initial_survived_in_epoch1 = initial_ids.iter().any(|&id| after_epoch_ids.contains(&id));
    let epoch1_survived_in_epoch2 = after_epoch_ids.iter().any(|&id| after_epoch2_ids.contains(&id));

    println!("Initial organisms survived epoch 1: {}", initial_survived_in_epoch1);
    println!("Epoch 1 organisms survived epoch 2: {}", epoch1_survived_in_epoch2);

    // At least some organisms should survive between epochs (since we're using constant fitness)
    assert!(initial_survived_in_epoch1 || epoch1_survived_in_epoch2, 
            "Some organisms should persist across epochs, but all IDs changed");
}
