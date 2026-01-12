//! Web visualization state serialization.
//!
//! Provides JSON state formatted for 2D web visualization.

use rayon::prelude::*;
use serde::Serialize;

use super::World;

/// World bounds for 2D visualization.
#[derive(Serialize, Debug)]
struct WorldBoundsState {
    x: (f64, f64),
    y: (f64, f64),
}

/// Score range across all organisms.
#[derive(Serialize, Debug)]
struct ScoreRangeState {
    min: f64,
    max: f64,
}

/// Region bounds for 2D visualization.
#[derive(Serialize, Debug)]
struct RegionBoundsState {
    x: (f64, f64),
    y: (f64, f64),
}

/// Region state for visualization.
#[derive(Serialize, Debug)]
struct RegionState {
    carrying_capacity: usize,
    bounds: RegionBoundsState,
    min_score: Option<f64>,
}

/// Organism position params.
#[derive(Serialize, Debug)]
struct OrganismParamsState {
    x: f64,
    y: f64,
}

/// Organism state for visualization.
#[derive(Serialize, Debug)]
struct OrganismState {
    id: u64,
    params: OrganismParamsState,
    age: usize,
    max_age: usize,
    score: Option<f64>,
    region_key: Option<Vec<usize>>,
    is_dead: bool,
}

/// Complete world state for web visualization.
#[derive(Serialize, Debug)]
struct WorldState {
    world_bounds: WorldBoundsState,
    score_range: ScoreRangeState,
    regions: Vec<RegionState>,
    organisms: Vec<OrganismState>,
}

impl World {
    /// Returns a JSON representation optimized for 2D web visualization.
    ///
    /// This method queries all organisms for their current state and formats
    /// the data for the `hill_descent_server` web interface.
    ///
    /// **Important**: This method only works with 2-dimensional optimization problems
    /// and will panic if called on worlds with more or fewer than 2 dimensions.
    ///
    /// # Returns
    ///
    /// A JSON string containing:
    /// - **world_bounds**: X and Y parameter ranges
    /// - **organisms**: Living organisms with 2D coordinates, scores, ages
    /// - **regions**: Spatial partitions with 2D bounding boxes
    /// - **score_range**: Min/max scores across all organisms
    ///
    /// Dead organisms are filtered out for cleaner visualization.
    ///
    /// # Panics
    ///
    /// Panics if the world is not 2-dimensional.
    pub fn get_state_for_web(&self) -> String {
        let dims = self.dimensions.get_dimensions();
        assert_eq!(dims.len(), 2, "get_state_for_web is only for 2D worlds");

        // Get world bounds from dimensions
        let world_bounds = WorldBoundsState {
            x: (*dims[0].range().start(), *dims[0].range().end()),
            y: (*dims[1].range().start(), *dims[1].range().end()),
        };

        // Query all organisms for their web state in parallel
        let web_states: Vec<_> = self
            .organisms
            .par_iter()
            .map(|(_, o)| {
                let org = o.read().unwrap();
                (org.id(), org.get_web_state())
            })
            .collect();

        // Build organism states, filtering out dead organisms
        let mut min_score = f64::MAX;
        let mut max_score = f64::MIN;

        let organisms: Vec<OrganismState> = web_states
            .into_iter()
            .filter_map(|(id, result)| {
                // Filter out dead organisms
                if result.is_dead {
                    return None;
                }

                // Update score range
                if let Some(score) = result.score {
                    if score < min_score {
                        min_score = score;
                    }
                    if score > max_score {
                        max_score = score;
                    }
                }

                // Extract x, y from params
                let params = OrganismParamsState {
                    x: result.params.first().copied().unwrap_or(0.0),
                    y: result.params.get(1).copied().unwrap_or(0.0),
                };

                Some(OrganismState {
                    id,
                    params,
                    age: result.age,
                    max_age: result.max_age,
                    score: result.score,
                    region_key: result.region_key.map(|k| k.values().to_vec()),
                    is_dead: result.is_dead,
                })
            })
            .collect();

        // Handle case where no scores found
        if min_score == f64::MAX {
            min_score = 0.0;
            max_score = 0.0;
        }

        let score_range = ScoreRangeState {
            min: min_score,
            max: max_score,
        };

        // Build region states
        let regions: Vec<RegionState> = self
            .regions
            .iter()
            .map(|(key, region)| {
                // Get region bounds from dimensions and key
                let bounds = self.calculate_region_bounds(key.values(), dims);

                RegionState {
                    carrying_capacity: region.carrying_capacity().unwrap_or(0),
                    bounds,
                    min_score: region.min_score(),
                }
            })
            .collect();

        let state = WorldState {
            world_bounds,
            score_range,
            regions,
            organisms,
        };

        serde_json::to_string(&state).expect("Serialization should not fail")
    }

    /// Calculate region bounds from a region key and dimensions.
    fn calculate_region_bounds(
        &self,
        key_values: &[usize],
        dims: &[crate::world::dimensions::Dimension],
    ) -> RegionBoundsState {
        let x_bounds = dims[0]
            .interval_bounds(key_values[0])
            .unwrap_or((*dims[0].range().start(), *dims[0].range().end()));
        let y_bounds = dims[1]
            .interval_bounds(key_values[1])
            .unwrap_or((*dims[1].range().start(), *dims[1].range().end()));

        RegionBoundsState {
            x: x_bounds,
            y: y_bounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::single_valued_function::SingleValuedFunction;
    use crate::{GlobalConstants, TrainingData};
    use std::ops::RangeInclusive;

    #[derive(Debug)]
    struct SumOfSquares;

    impl SingleValuedFunction for SumOfSquares {
        fn single_run(&self, params: &[f64]) -> f64 {
            params.iter().map(|x| x * x).sum()
        }
    }

    #[test]
    fn given_2d_world_when_get_state_for_web_then_returns_valid_json() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));

        // Run one training cycle to populate data
        world.training_run(TrainingData::None { floor_value: 0.0 });

        let json = world.get_state_for_web();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");

        // Check required fields exist
        assert!(parsed.get("world_bounds").is_some());
        assert!(parsed.get("organisms").is_some());
        assert!(parsed.get("regions").is_some());
        assert!(parsed.get("score_range").is_some());
    }

    #[test]
    #[should_panic(expected = "get_state_for_web is only for 2D worlds")]
    fn given_3d_world_when_get_state_for_web_then_panics() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0, -5.0..=5.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(10, 2, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));
        world.training_run(TrainingData::None { floor_value: 0.0 });

        // This should panic
        let _ = world.get_state_for_web();
    }

    #[test]
    fn given_world_when_get_state_for_web_then_world_bounds_correct() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -5.0..=5.0];
        let constants = GlobalConstants::new_with_seed(10, 2, 42);

        let mut world = World::new(&bounds, constants, Box::new(SumOfSquares));
        world.training_run(TrainingData::None { floor_value: 0.0 });

        let json = world.get_state_for_web();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let world_bounds = parsed.get("world_bounds").unwrap();
        let x = world_bounds.get("x").unwrap().as_array().unwrap();
        let y = world_bounds.get("y").unwrap().as_array().unwrap();

        assert_eq!(x[0].as_f64().unwrap(), -10.0);
        assert_eq!(x[1].as_f64().unwrap(), 10.0);
        assert_eq!(y[0].as_f64().unwrap(), -5.0);
        assert_eq!(y[1].as_f64().unwrap(), 5.0);
    }
}
