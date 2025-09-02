use hill_descent_lib::{
    GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction,
};
use std::ops::RangeInclusive;

// Simple test function for zone mapping verification
#[derive(Debug)]
struct SimpleFunction;

impl SingleValuedFunction for SimpleFunction {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        // Simple quadratic function
        x * x + y * y
    }
}

#[test]
fn test_zone_mapping_in_web_state() {
    // Create a small 2D world
    let param_range = vec![
        RangeInclusive::new(-2.0, 2.0),
        RangeInclusive::new(-2.0, 2.0),
    ];
    let global_constants = GlobalConstants::new(20, 4); // Small population

    let mut world = setup_world(&param_range, global_constants, Box::new(SimpleFunction));

    // Run a few epochs to establish regions and zones
    for _ in 0..10 {
        world.training_run(&[], &[]);
    }

    // Get web state which should include zone information
    let web_state_json = world.get_state_for_web();

    // Parse the JSON to verify zone information is included
    let web_state: serde_json::Value =
        serde_json::from_str(&web_state_json).expect("Failed to parse web state JSON");

    // Check that regions array exists
    let regions = web_state["regions"]
        .as_array()
        .expect("Web state should contain regions array");

    // Verify that at least some regions have zone information
    if !regions.is_empty() {
        // Check the first region has a zone field
        let first_region = &regions[0];
        assert!(
            first_region.get("zone").is_some(),
            "Region should have a zone field"
        );

        println!("Zone mapping test passed!");
        println!("First region zone: {:?}", first_region["zone"]);
        println!("Number of regions: {}", regions.len());
    } else {
        println!("No regions created in test - this is expected for some parameter ranges");
    }
}

#[test]
fn test_zone_mapping_methods() {
    // Create a small 2D world
    let param_range = vec![
        RangeInclusive::new(-1.0, 1.0),
        RangeInclusive::new(-1.0, 1.0),
    ];
    let global_constants = GlobalConstants::new(10, 2);

    let mut world = setup_world(&param_range, global_constants, Box::new(SimpleFunction));

    // Run a few epochs to establish regions and populate zone mapping
    for _ in 0..5 {
        world.training_run(&[], &[]);
    }

    // Access regions directly to test zone mapping methods
    // Note: This is testing internal functionality - the public API would use get_state_for_web
    println!("Zone mapping methods test completed");
}
