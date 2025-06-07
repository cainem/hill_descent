use crate::parameters::global_constants::GlobalConstants;
use dimension::Dimension;
use organisms::Organisms;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use regions::Regions; // Required for SmallRng::from_seed
use std::ops::RangeInclusive;

const DEFAULT_WORLD_SEED: u64 = 2_147_483_647; // A Mersenne prime (2^31 - 1)

pub mod dimension;
pub mod organisms;
pub mod regions;

#[derive(Debug, Clone)]
pub struct World {
    _dimensions: Vec<Dimension>,
    _last_division_index: usize,
    _organisms: Organisms,
    _regions: Regions,
    rng: SmallRng,
}

impl World {
    pub fn new(
        dimensions: Vec<Dimension>,
        last_division_index: usize,
        // organisms: Organisms, // Organisms will be created internally
        regions: Regions,
        user_defined_parameter_bounds: &[RangeInclusive<f64>],
        population_size: usize,
        max_regions: usize,
    ) -> Self {
        let mut rng = SmallRng::seed_from_u64(DEFAULT_WORLD_SEED);
        let global_constants = GlobalConstants::new(population_size, max_regions);
        let organisms_instance =
            Organisms::new(&mut rng, user_defined_parameter_bounds, &global_constants);

        World {
            _dimensions: dimensions,
            _last_division_index: last_division_index,
            _organisms: organisms_instance,
            _regions: regions,
            rng, // rng is already initialized
        }
    }

    pub fn reset_rng(&mut self, new_seed: u64) {
        self.rng = SmallRng::seed_from_u64(new_seed);
    }

    // Example of how you might provide access to the rng
    // pub fn rng_mut(&mut self) -> &mut SmallRng {
    //     &mut self.rng
    // }
}
