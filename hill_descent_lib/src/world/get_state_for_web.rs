use crate::locus::locus_adjustment::DirectionOfTravel;
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

// Full detail structures for complete organism information
#[derive(Serialize, Debug)]
struct LocusAdjustmentState {
    adjustment_value: f64,
    direction_of_travel: String, // "Add" or "Subtract"
    doubling_or_halving_flag: bool,
    checksum: u64,
}

#[derive(Serialize, Debug)]
struct LocusState {
    value: f64,
    adjustment: LocusAdjustmentState,
    apply_adjustment_flag: bool,
}

#[derive(Serialize, Debug)]
struct GameteState {
    loci: Vec<LocusState>,
}

#[derive(Serialize, Debug)]
struct SystemParametersState {
    m1: f64,
    m2: f64,
    m3: f64,
    m4: f64,
    m5: f64,
    max_age: f64,
    crossover_points: f64,
}

#[derive(Serialize, Debug)]
struct PhenotypeState {
    gamete1: GameteState,
    gamete2: GameteState,
    expressed_values: Vec<f64>,
    system_parameters: SystemParametersState,
    expressed_hash: u64,
}

#[derive(Serialize, Debug)]
struct OrganismState {
    id: usize,
    params: OrganismParamsState,
    age: usize,
    max_age: usize,
    score: Option<f64>,
    region_key: Option<Vec<usize>>,
    is_dead: bool,
    parent_id_1: Option<usize>,
    parent_id_2: Option<usize>,
    phenotype: PhenotypeState,
}

#[derive(Serialize, Debug)]
struct WorldState {
    world_bounds: WorldBoundsState,
    score_range: ScoreRangeState,
    regions: Vec<RegionState>,
    organisms: Vec<OrganismState>,
}

// Helper functions for converting internal structures to state structures
impl LocusAdjustmentState {
    fn from_locus_adjustment(adj: &crate::locus::locus_adjustment::LocusAdjustment) -> Self {
        Self {
            adjustment_value: adj.adjustment_value().get(),
            direction_of_travel: match adj.direction_of_travel() {
                DirectionOfTravel::Add => "Add".to_string(),
                DirectionOfTravel::Subtract => "Subtract".to_string(),
            },
            doubling_or_halving_flag: adj.doubling_or_halving_flag(),
            checksum: adj.checksum(),
        }
    }
}

impl LocusState {
    fn from_locus(locus: &crate::locus::Locus) -> Self {
        Self {
            value: locus.value().get(),
            adjustment: LocusAdjustmentState::from_locus_adjustment(locus.adjustment()),
            apply_adjustment_flag: locus.apply_adjustment_flag(),
        }
    }
}

impl GameteState {
    fn from_gamete(gamete: &crate::gamete::Gamete) -> Self {
        Self {
            loci: gamete.loci().iter().map(LocusState::from_locus).collect(),
        }
    }
}

impl SystemParametersState {
    fn from_system_parameters(
        sys_params: &crate::parameters::system_parameters::SystemParameters,
    ) -> Self {
        Self {
            m1: sys_params.m1(),
            m2: sys_params.m2(),
            m3: sys_params.m3(),
            m4: sys_params.m4(),
            m5: sys_params.m5(),
            max_age: sys_params.max_age(),
            crossover_points: sys_params.crossover_points(),
        }
    }
}

impl PhenotypeState {
    fn from_phenotype(phenotype: &crate::phenotype::Phenotype) -> Self {
        Self {
            gamete1: GameteState::from_gamete(phenotype.gamete1()),
            gamete2: GameteState::from_gamete(phenotype.gamete2()),
            expressed_values: phenotype.expressed_values().to_vec(),
            system_parameters: SystemParametersState::from_system_parameters(
                phenotype.system_parameters(),
            ),
            expressed_hash: phenotype.expressed_hash(),
        }
    }
}

impl super::World {
    /// Returns a `String` containing a JSON representation of the current World state,
    /// structured for 2D web visualization.
    ///
    /// **Note:** This method is specifically designed for 2D worlds and produces a format
    /// optimized for web-based visualization. For general-purpose state serialization that
    /// works with any number of dimensions, use [`get_state()`](Self::get_state) instead.
    ///
    /// # Panics
    ///
    /// Panics if the world is not exactly 2-dimensional.
    ///
    /// Returns a JSON representation optimized for 2D web visualization.
    ///
    /// This is a specialized version of [`get_state`](super::World::get_state) designed for
    /// the `hill_descent_server` web interface. It formats data specifically for
    /// 2D visualizations and includes additional metadata for rendering.
    ///
    /// **Important**: This method only works with 2-dimensional optimization problems
    /// and will panic if called on worlds with more or fewer than 2 dimensions.
    ///
    /// # Returns
    ///
    /// A JSON string containing:
    ///
    /// - **world_bounds**: X and Y parameter ranges
    /// - **organisms**: Living organisms with 2D coordinates, scores, ages, parent IDs
    /// - **regions**: Spatial partitions with 2D bounding boxes
    /// - **score_range**: Min/max scores across all organisms
    ///
    /// Dead organisms are filtered out for cleaner visualization.
    ///
    /// # JSON Structure
    ///
    /// ```json
    /// {
    ///   "world_bounds": {
    ///     "x": [-10.0, 10.0],
    ///     "y": [-10.0, 10.0]
    ///   },
    ///   "organisms": [
    ///     {
    ///       "id": 42,
    ///       "params": {"x": 1.23, "y": 4.56},
    ///       "age": 5,
    ///       "max_age": 10,
    ///       "score": 0.0123,
    ///       "region_key": [0, 1],
    ///       "is_dead": false,
    ///       "parent_id_1": 20,
    ///       "parent_id_2": 35,
    ///       "phenotype": { ... }
    ///     }
    ///   ],
    ///   "regions": [
    ///     {
    ///       "carrying_capacity": 10,
    ///       "bounds": {
    ///         "x": [0.0, 5.0],
    ///         "y": [0.0, 5.0]
    ///       },
    ///       "min_score": 0.01
    ///     }
    ///   ],
    ///   "score_range": {
    ///     "min": 0.001,
    ///     "max": 10.5
    ///   }
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ## Web Server Integration
    ///
    /// ```
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
    ///
    /// #[derive(Debug)]
    /// struct Himmelblau;
    ///
    /// impl SingleValuedFunction for Himmelblau {
    ///     fn single_run(&self, params: &[f64]) -> f64 {
    ///         let x = params[0];
    ///         let y = params[1];
    ///         (x.powi(2) + y - 11.0).powi(2) + (x + y.powi(2) - 7.0).powi(2)
    ///     }
    /// }
    ///
    /// // Must be 2D
    /// let param_range = vec![-5.0..=5.0, -5.0..=5.0];
    /// let constants = GlobalConstants::new(200, 20);
    /// let mut world = setup_world(&param_range, constants, Box::new(Himmelblau));
    ///
    /// world.training_run(&[], &[0.0]);
    ///
    /// // Get web-optimized JSON
    /// let json = world.get_state_for_web();
    ///
    /// // Send to web client
    /// // http_response.body(json)
    /// ```
    ///
    /// ## Visualization API Endpoint
    ///
    /// ```ignore
    /// // In hill_descent_server
    /// #[get("/api/state")]
    /// async fn get_state(world: web::Data<Mutex<World>>) -> impl Responder {
    ///     let world = world.lock().unwrap();
    ///     HttpResponse::Ok()
    ///         .content_type("application/json")
    ///         .body(world.get_state_for_web())
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the world is not 2-dimensional:
    ///
    /// ```should_panic
    /// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction};
    ///
    /// #[derive(Debug)]
    /// struct F;
    /// impl SingleValuedFunction for F {
    ///     fn single_run(&self, p: &[f64]) -> f64 { p.iter().sum() }
    /// }
    ///
    /// // 3D world - will panic!
    /// let param_range = vec![-1.0..=1.0; 3];
    /// let constants = GlobalConstants::new(10, 2);
    /// let mut world = setup_world(&param_range, constants, Box::new(F));
    ///
    /// world.training_run(&[], &[0.0]);
    /// world.get_state_for_web(); // PANICS: not 2D
    /// ```
    ///
    /// # Performance
    ///
    /// Similar to [`get_state`](super::World::get_state), this is O(n) for n organisms.
    /// The additional filtering and 2D-specific formatting has minimal overhead.
    ///
    /// # See Also
    ///
    /// - [`get_state`](super::World::get_state) - Full state for n-dimensional problems
    /// - `hill_descent_server` - Web visualization server using this method
    /// - `web_pdd.md` - Documentation of the web visualization contract
    ///
    /// # Example
    ///
    /// ```ignore
    /// let world = setup_world(&two_d_params, constants, function);
    /// let json = world.get_state_for_web(); // Works for 2D only
    /// ```
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
                let raw_max_age = o.phenotype().system_parameters().max_age();
                let rounded_max_age = raw_max_age.round() as usize;

                let (parent_id_1, parent_id_2) = o.parent_ids();
                Some(OrganismState {
                    id: o.id(),
                    params,
                    age: o.age(),
                    max_age: rounded_max_age,
                    score: o.score(),
                    region_key: o.region_key(),
                    is_dead: o.is_dead(),
                    parent_id_1,
                    parent_id_2,
                    phenotype: PhenotypeState::from_phenotype(o.phenotype()),
                })
            })
            .collect();

        let mut min_score_global = f64::MAX;
        let mut max_score_global = f64::MIN;

        // Also capture the region keys so we can validate organism membership precisely.
        let mut region_keys: Vec<Vec<usize>> = Vec::new();

        let regions: Vec<RegionState> = self
            .regions
            .iter_regions()
            .map(|(key, region)| {
                if let Some(score) = region.min_score() {
                    min_score_global = min_score_global.min(score);
                    max_score_global = max_score_global.max(score);
                }

                // Calculate region bounds from key
                let mut bounds_x = (0.0, 0.0);
                let mut bounds_y = (0.0, 0.0);

                for (i, &dim_idx) in key.iter().enumerate() {
                    let (start, end) = dims[i]
                        .interval_bounds(dim_idx)
                        .expect("Region key contained an out-of-range interval index");
                    if i == 0 {
                        bounds_x = (start, end);
                    } else {
                        bounds_y = (start, end);
                    }
                }

                // Keep the key for later membership validation
                region_keys.push(key.clone());

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

        // Debug assertion: every organism must belong to exactly one region key derived by get_interval
        for org in &organisms {
            let xi = dims[0]
                .get_interval(org.params.x)
                .expect("Organism x not in any interval despite dimensions");
            let yi = dims[1]
                .get_interval(org.params.y)
                .expect("Organism y not in any interval despite dimensions");

            let in_region_key = region_keys
                .iter()
                .any(|k| k.len() == 2 && k[0] == xi && k[1] == yi);
            if !in_region_key {
                eprintln!("Organism outside any region: {org:?}");
                eprintln!("Regions: {regions:?}");
                eprintln!("Dimension 0 range: {:?}", dims[0].range());
                eprintln!("Dimension 1 range: {:?}", dims[1].range());

                panic!(
                    "Organism {:?} not mapped to any region key (xi={}, yi={}). dims0 {:?} dims1 {:?}. Regions: {:?}",
                    org,
                    xi,
                    yi,
                    dims[0].range(),
                    dims[1].range(),
                    regions
                );
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
        let gc = GlobalConstants::new(10, 4);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let mut world = super::super::World::new(&bounds, gc, world_fn);
        // Manually run a round to populate regions and organisms with some data
        world.training_run(&[], &[0.0]);

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

        // Check organism fields
        let organisms = parsed.get("organisms").unwrap().as_array().unwrap();
        for org in organisms {
            assert!(org.get("params").is_some());
            assert!(org.get("age").is_some());
            assert!(org.get("max_age").is_some());
            // max_age should be within its defined bounds [2.0, 10.0].
            let max_age = org.get("max_age").unwrap().as_f64().unwrap();
            assert!(
                (2.0..=10.0).contains(&max_age),
                "max_age {max_age} is out of bounds"
            );
        }
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
