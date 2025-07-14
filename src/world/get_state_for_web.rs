use serde::Serialize;
use serde_json;

// Helper structs purely for serialisation of the World state to match web_pdd.md
#[derive(Serialize, Debug)]
struct WorldBoundsState {
    x: (f64, f64),
    y: (f64, f64),
}

#[derive(Serialize, Debug)]
struct ScoreRangeState {
    min: f64,
    max: f64,
}

#[derive(Serialize, Debug)]
struct RegionBoundsState {
    x: (f64, f64),
    y: (f64, f64),
}

#[derive(Serialize, Debug)]
pub struct RegionState {
    pub carrying_capacity: usize,
    bounds: RegionBoundsState,
    min_score: Option<f64>,
}

#[derive(Serialize, Debug)]
struct OrganismParamsState {
    x: f64,
    y: f64,
}

#[derive(Serialize, Debug)]
struct OrganismState {
    params: OrganismParamsState,
    age: usize,
}

#[derive(Serialize, Debug)]
struct WorldState {
    world_bounds: WorldBoundsState,
    score_range: ScoreRangeState,
    regions: Vec<RegionState>,
    organisms: Vec<OrganismState>,
}

impl super::World {
    /// Returns a `String` containing a JSON representation of the current World state,
    /// structured according to the `web_pdd.md` for web visualization.
    pub fn get_state_for_web(&self) -> String {
        let dims = self.dimensions.get_dimensions();
        // This function is specifically for a 2D visualization.
        // It will panic if the world is not 2D.
        assert_eq!(dims.len(), 2, "get_state_for_web is only for 2D worlds");

        let world_bounds = WorldBoundsState {
            x: (*dims[0].range().start(), *dims[0].range().end()),
            y: (*dims[1].range().start(), *dims[1].range().end()),
        };

        let organisms: Vec<OrganismState> = self
            .organisms
            .iter()
            .filter_map(|o| {
                if o.is_dead() {
                    return None; // Filter out dead organisms
                }
                let expressed_values = o.phenotype().expressed_values();
                // The first two non-system parameters are x and y
                let params = OrganismParamsState {
                    x: expressed_values[crate::NUM_SYSTEM_PARAMETERS],
                    y: expressed_values[crate::NUM_SYSTEM_PARAMETERS + 1],
                };
                Some(OrganismState {
                    params,
                    age: o.age(),
                })
            })
            .collect();

        let mut min_score_global = f64::MAX;
        let mut max_score_global = f64::MIN;

        let regions: Vec<RegionState> = self
            .regions
            .regions()
            .iter()
            .map(|(key, region)| {
                if let Some(score) = region.min_score() {
                    min_score_global = min_score_global.min(score);
                    max_score_global = max_score_global.max(score);
                }

                // Calculate region bounds from key
                let mut bounds_x = (0.0, 0.0);
                let mut bounds_y = (0.0, 0.0);

                for (i, &dim_idx) in key.iter().enumerate() {
                    let dim = &dims[i];
                    let intervals = (dim.number_of_divisions() + 1) as f64;
                    let div_size = (*dim.range().end() - *dim.range().start()) / intervals;
                    let start = *dim.range().start() + dim_idx as f64 * div_size;
                    let mut end = start + div_size;
                    // Ensure exact upper bound on last interval to avoid floating precision gaps
                    if dim_idx + 1 == intervals as usize {
                        end = *dim.range().end();
                    }
                    if i == 0 {
                        bounds_x = (start, end);
                    } else {
                        bounds_y = (start, end);
                    }
                }

                RegionState {
                    bounds: RegionBoundsState {
                        x: bounds_x,
                        y: bounds_y,
                    },
                    min_score: region.min_score(),
                    carrying_capacity: region.carrying_capacity().unwrap_or(0),
                }
            })
            .collect();

        // If no scores found, use a default range
        if max_score_global < min_score_global {
            min_score_global = 0.0;
            max_score_global = 0.0;
        }

        // Debug assertion: every organism must belong to at least one region
        for org in &organisms {
            let in_region = regions.iter().any(|r| {
                let (x0, x1) = r.bounds.x;
                let (y0, y1) = r.bounds.y;
                org.params.x >= x0 && org.params.x <= x1 && org.params.y >= y0 && org.params.y <= y1
            });
            if !in_region {
                eprintln!("Organism outside any region: {org:?}");
                eprintln!("Regions: {regions:?}");
                eprintln!("Dimension 0 range: {:?}", dims[0].range());
                eprintln!("Dimension 1 range: {:?}", dims[1].range());

                if !in_region {
                    panic!(
                        "Organism {:?} outside any region. dims0 {:?} dims1 {:?}. Regions: {:?}",
                        org,
                        dims[0].range(),
                        dims[1].range(),
                        regions
                    );
                }
            }
        }

        let state = WorldState {
            world_bounds,
            score_range: ScoreRangeState {
                min: min_score_global,
                max: max_score_global,
            },
            regions,
            organisms,
        };

        serde_json::to_string(&state).expect("Serialization of World state failed")
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::world::world_function::WorldFunction;
    use std::ops::RangeInclusive;

    // Minimal WorldFunction for tests
    #[derive(Debug)]
    struct DummyFn;
    impl WorldFunction for DummyFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![0.0]
        }
    }

    #[test]
    fn given_2d_world_when_get_state_for_web_then_returns_valid_json_in_pdd_format() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0, 0.0..=20.0];
        let gc = GlobalConstants::new(4, 10);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let mut world = super::super::World::new(&bounds, gc, world_fn);
        // Manually run a round to populate regions and organisms with some data
        world.training_run(&[], &[]);

        let json = world.get_state_for_web();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check top-level keys
        assert!(parsed.get("world_bounds").is_some());
        assert!(parsed.get("score_range").is_some());
        assert!(parsed.get("regions").is_some());
        assert!(parsed.get("organisms").is_some());

        // Check that the world bounds from the state encompass the original input bounds.
        let wb = parsed.get("world_bounds").unwrap();
        let x_bounds = wb.get("x").unwrap().as_array().unwrap();
        let y_bounds = wb.get("y").unwrap().as_array().unwrap();
        assert!(x_bounds[0].as_f64().unwrap() >= *bounds[0].start());
        assert!(x_bounds[1].as_f64().unwrap() <= *bounds[0].end());
        assert!(y_bounds[0].as_f64().unwrap() >= *bounds[1].start());
        assert!(y_bounds[1].as_f64().unwrap() <= *bounds[1].end());
    }

    #[test]
    #[should_panic]
    fn given_non_2d_world_when_get_state_for_web_then_panics() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0]; // Only 1 dimension
        let gc = GlobalConstants::new(4, 10);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);
        world.get_state_for_web(); // Should panic
    }
}
