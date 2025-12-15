//! World constructor - creates a new World with initial population.

use std::sync::Arc;

use messaging_thread_pool::ThreadPool;
use rand::{SeedableRng, rngs::StdRng};

use super::{Dimensions, Regions, World, WorldFunction};
use crate::{
    organism::{CreateOrganism, Organism},
    parameters::GlobalConstants,
    phenotype::Phenotype,
};

impl World {
    /// Creates a new World with an initial random population.
    ///
    /// This method:
    /// 1. Creates a thread pool for organisms
    /// 2. Generates initial random phenotypes
    /// 3. Creates organisms in the pool
    /// 4. Sets up dimensions and regions
    ///
    /// # Arguments
    ///
    /// * `param_range` - Bounds for each parameter dimension (extended for system parameters)
    /// * `global_constants` - Configuration (population size, target regions, seed)
    /// * `world_function` - The fitness function to optimize
    ///
    /// # Returns
    ///
    /// A new World ready for training.
    ///
    /// # Panics
    ///
    /// Panics if population_size or target_regions is 0.
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

        // Get population size first (needed for thread count calculation)
        let population_size = global_constants.population_size();

        // Create thread pool with optimal thread count.
        // Using threads = population_size gives optimal work distribution (1 organism per thread).
        // Cap at 5x logical CPUs to prevent excessive context switching on large populations.
        let available_cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        let max_threads = available_cpus * 5;
        let thread_count = population_size.min(max_threads) as u64;
        let organism_pool = ThreadPool::<Organism>::new(thread_count);

        // Generate initial organisms
        let organism_ids: Vec<u64> = (0..population_size as u64).collect();

        // Build batch of create requests for all organisms
        let create_requests: Vec<CreateOrganism> = organism_ids
            .iter()
            .map(|&organism_id| {
                // Generate random phenotype
                let phenotype =
                    Arc::new(Phenotype::new_random_phenotype(&mut rng, &extended_bounds));

                CreateOrganism {
                    id: organism_id,
                    parent_ids: (None, None), // Initial organisms have no parents
                    phenotype,
                    dimensions: Arc::clone(&dimensions),
                    world_function: Arc::clone(&world_function),
                }
            })
            .collect();

        // Add all organisms to pool in a batch
        organism_pool
            .send_and_receive(create_requests.into_iter())
            .expect("Thread pool should be available during initialization")
            .for_each(drop);

        // Create regions
        let regions = Regions::new(&global_constants);

        World {
            organism_pool,
            dimensions,
            dimension_version: 0,
            regions,
            world_function,
            global_constants,
            best_score: f64::MAX,
            best_organism_id: None,
            best_params: Vec::new(),
            organism_ids,
            next_organism_id: population_size as u64,
            world_seed,
        }
    }

    /// Creates a new World with a specific thread count.
    ///
    /// This is primarily for benchmarking and testing different thread configurations.
    /// For normal use, prefer [`World::new`] which uses the optimal thread count.
    ///
    /// # Arguments
    ///
    /// * `param_range` - Bounds for each parameter dimension
    /// * `global_constants` - Configuration (population size, target regions, seed)
    /// * `world_function` - The fitness function to optimize
    /// * `thread_count` - Number of threads for the organism pool
    ///
    /// # Panics
    ///
    /// Panics if thread_count is 0.
    #[doc(hidden)]
    pub fn new_with_thread_count(
        param_range: &[std::ops::RangeInclusive<f64>],
        global_constants: GlobalConstants,
        world_function: Box<dyn WorldFunction + Send + Sync>,
        thread_count: u64,
    ) -> Self {
        assert!(thread_count > 0, "thread_count must be greater than 0");

        let world_seed = global_constants.world_seed();
        let mut rng = StdRng::seed_from_u64(world_seed);

        let extended_bounds = create_extended_bounds(param_range);
        let dimensions = Arc::new(Dimensions::new(param_range));
        let world_function: Arc<dyn WorldFunction + Send + Sync> = Arc::from(world_function);

        let organism_pool = ThreadPool::<Organism>::new(thread_count);

        let population_size = global_constants.population_size();
        let organism_ids: Vec<u64> = (0..population_size as u64).collect();

        let create_requests: Vec<CreateOrganism> = organism_ids
            .iter()
            .map(|&organism_id| {
                let phenotype =
                    Arc::new(Phenotype::new_random_phenotype(&mut rng, &extended_bounds));
                CreateOrganism {
                    id: organism_id,
                    parent_ids: (None, None),
                    phenotype,
                    dimensions: Arc::clone(&dimensions),
                    world_function: Arc::clone(&world_function),
                }
            })
            .collect();

        organism_pool
            .send_and_receive(create_requests.into_iter())
            .expect("Thread pool should be available during initialization")
            .for_each(drop);

        let regions = Regions::new(&global_constants);

        World {
            organism_pool,
            dimensions,
            dimension_version: 0,
            regions,
            world_function,
            global_constants,
            best_score: f64::MAX,
            best_organism_id: None,
            best_params: Vec::new(),
            organism_ids,
            next_organism_id: population_size as u64,
            world_seed,
        }
    }
}

/// Creates extended parameter bounds by prepending system parameter bounds.
///
/// The system parameters are at the BEGINNING of expressed values:
/// [m1, m2, m3, m4, m5, max_age, crossover_points, ...problem_params...]
///
/// System parameter bounds:
/// - m1-m5: mutation rates in 0.0..=1.0
/// - max_age: organism lifespan in 2.0..=10.0
/// - crossover_points: for reproduction in 1.0..=10.0
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::single_valued_function::SingleValuedFunction;
    use std::ops::RangeInclusive;

    // Mock WorldFunction for testing
    #[derive(Debug)]
    struct TestFunction;

    impl SingleValuedFunction for TestFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            0.0
        }
    }

    #[test]
    fn given_valid_params_when_world_new_then_population_created() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new(50, 5);

        let world = World::new(&bounds, constants, Box::new(TestFunction));

        assert_eq!(world.organism_count(), 50);
        assert_eq!(world.organism_ids.len(), 50);
        assert_eq!(world.next_organism_id, 50);
    }

    #[test]
    fn given_world_new_when_created_then_organism_ids_sequential() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-5.0..=5.0];
        let constants = GlobalConstants::new(10, 2);

        let world = World::new(&bounds, constants, Box::new(TestFunction));

        // IDs should be 0..9
        for (idx, &id) in world.organism_ids.iter().enumerate() {
            assert_eq!(id, idx as u64);
        }
    }

    #[test]
    fn given_world_new_when_created_then_best_score_is_max() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=100.0, 0.0..=100.0];
        let constants = GlobalConstants::new(20, 4);

        let world = World::new(&bounds, constants, Box::new(TestFunction));

        assert_eq!(world.best_score, f64::MAX);
        assert!(world.best_organism_id.is_none());
    }

    #[test]
    fn given_world_new_when_created_then_dimensions_version_zero() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-1.0..=1.0, -1.0..=1.0, -1.0..=1.0];
        let constants = GlobalConstants::new(30, 3);

        let world = World::new(&bounds, constants, Box::new(TestFunction));

        assert_eq!(world.dimension_version(), 0);
        assert_eq!(world.dimensions().num_dimensions(), 3);
    }

    #[test]
    fn given_world_new_when_created_then_regions_initialized() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0, 0.0..=10.0];
        let constants = GlobalConstants::new(100, 10);

        let world = World::new(&bounds, constants, Box::new(TestFunction));

        // Regions are empty initially until calculate_region_keys is called
        assert!(world.regions.is_empty());
    }

    #[test]
    fn given_world_new_when_same_seed_then_deterministic() {
        let bounds: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0, -10.0..=10.0];
        let constants = GlobalConstants::new_with_seed(20, 4, 42);

        let world1 = World::new(&bounds, constants, Box::new(TestFunction));
        let world2 = World::new(&bounds, constants, Box::new(TestFunction));

        // Both worlds should have the same organism IDs
        assert_eq!(world1.organism_ids, world2.organism_ids);
        assert_eq!(world1.world_seed(), world2.world_seed());
    }

    #[test]
    fn given_extended_bounds_when_created_then_includes_system_params() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=10.0, 0.0..=20.0];

        let extended = create_extended_bounds(&bounds);

        // Should have NUM_SYSTEM_PARAMETERS + original 2
        assert_eq!(extended.len(), crate::NUM_SYSTEM_PARAMETERS + 2);

        // First 5 should be system param bounds (m1-m5: 0.0..=1.0)
        for (i, bound) in extended.iter().enumerate().take(5) {
            assert_eq!(*bound, 0.0..=1.0, "m{} bounds should be 0.0..=1.0", i + 1);
        }

        // max_age: 2.0..=10.0
        assert_eq!(extended[5], 2.0..=10.0);

        // crossover_points: 1.0..=10.0
        assert_eq!(extended[6], 1.0..=10.0);

        // User bounds should be after system params
        assert_eq!(extended[7], 0.0..=10.0);
        assert_eq!(extended[8], 0.0..=20.0);
    }
}
