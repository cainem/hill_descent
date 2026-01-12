//! World constructor - creates a new World with initial population.

use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use rand::{SeedableRng, rngs::StdRng};

use super::{Dimensions, Regions, World, WorldFunction};
use crate::{organism::Organism, parameters::GlobalConstants, phenotype::Phenotype};

impl World {
    /// Creates a new World with an initial random population.
    pub fn new(
        param_range: &[std::ops::RangeInclusive<f64>],
        global_constants: GlobalConstants,
        world_function: Box<dyn WorldFunction + Send + Sync>,
    ) -> Self {
        let world_seed = global_constants.world_seed();
        let mut rng = StdRng::seed_from_u64(world_seed);

        // Create extended parameter bounds (problem params + system params)
        let extended_bounds = create_extended_bounds(param_range);

        // Create dimensions from the user-provided bounds (problem space only)
        let dimensions = Arc::new(Dimensions::new(param_range));

        // Convert Box to Arc
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::from(world_function);

        let population_size = global_constants.population_size();

        // Generate initial organisms into IndexMap (maintains insertion order for determinism)
        let mut organisms: IndexMap<u64, Arc<RwLock<Organism>>> =
            IndexMap::with_capacity(population_size);

        for organism_id in 0..(population_size as u64) {
            let phenotype = Arc::new(Phenotype::new_random_phenotype(&mut rng, &extended_bounds));

            let org = Organism::new(
                organism_id,
                (None, None),
                phenotype,
                Arc::clone(&dimensions),
                Arc::clone(&world_function),
            );
            organisms.insert(organism_id, Arc::new(RwLock::new(org)));
        }

        // Create regions
        let regions = Regions::new(&global_constants);

        World {
            organisms,
            dimensions,
            dimension_version: 0,
            regions,
            world_function,
            global_constants,
            best_score: f64::MAX,
            best_organism_id: None,
            best_params: Vec::new(),
            next_organism_id: population_size as u64,
            world_seed,
        }
    }
}

/// Creates extended parameter bounds by prepending system parameter bounds.
fn create_extended_bounds(
    param_range: &[std::ops::RangeInclusive<f64>],
) -> Vec<std::ops::RangeInclusive<f64>> {
    // System parameter bounds (prepended to user bounds)
    let system_bounds: Vec<std::ops::RangeInclusive<f64>> = vec![
        0.0..=1.0,  // m1_prob_false_to_true
        0.0..=1.0,  // m2_prob_true_to_false
        0.0..=1.0,  // m3_prob_adj_double_halve_flag
        0.0..=1.0,  // m4_prob_adj_direction_flag
        0.0..=1.0,  // m5_prob_locus_value_mutation
        2.0..=10.0, // max_age
        1.0..=10.0, // crossover_points
    ];

    let mut extended = system_bounds;
    extended.extend_from_slice(param_range);
    extended
}
