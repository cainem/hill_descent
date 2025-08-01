use crate::GlobalConstants;
use crate::world::World;
use crate::world::single_valued_function::SingleValuedFunction;
use std::ops::RangeInclusive;
use wasm_bindgen::prelude::*;

// 1. Define the objective function for WASM
#[derive(Debug)]
struct Himmelblau;

impl SingleValuedFunction for Himmelblau {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64 {
        assert_eq!(2, phenotype_expressed_values.len());
        let x = phenotype_expressed_values[0];
        let y = phenotype_expressed_values[1];
        let term1 = (x.powi(2) + y - 11.0).powi(2);
        let term2 = (x + y.powi(2) - 7.0).powi(2);
        term1 + term2
    }
}

// 2. Create a WASM-accessible wrapper for the World
/// A WASM-accessible wrapper for the main `World` struct.
///
/// This struct is the primary interface between JavaScript and the Rust simulation.
/// It allows for the creation and stepping of the simulation environment.
#[wasm_bindgen]
pub struct WasmWorld {
    world: World,
}

// 3. Implement methods on the wrapper that can be called from JavaScript
#[wasm_bindgen]
impl WasmWorld {
    /// Creates a new world instance, configured for the Himmelblau test.
    pub fn new() -> Self {
        let param_range = vec![
            RangeInclusive::new(-25000000.0, -5000000.0),
            RangeInclusive::new(-25000000.0, -5000000.0),
        ];
        let global_constants = GlobalConstants::new(100, 10);

        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let world = World::new(&param_range, global_constants, Box::new(Himmelblau));

        WasmWorld { world }
    }

    /// Runs one epoch of the simulation and returns the best score from that epoch.
    pub fn training_run(&mut self) -> f64 {
        self.world.training_run(&[], &[]);
        self.world.get_best_score()
    }

    /// Returns a JSON string representing the current state of the world for web visualization.
    /// This is used by the frontend to render the visualization.
    pub fn get_state_for_web(&self) -> String {
        self.world.get_state_for_web()
    }
}
