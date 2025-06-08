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
    _global_constants: GlobalConstants,
    _rng: SmallRng,
}

impl World {
    pub fn new(
        dimensions: Vec<Dimension>,
        last_division_index: usize,
        // organisms: Organisms, // Organisms will be created internally
        regions: Regions,
        user_defined_parameter_bounds: &[RangeInclusive<f64>],
        global_constants: GlobalConstants,
    ) -> Self {
        let mut rng = SmallRng::seed_from_u64(DEFAULT_WORLD_SEED);
        let organisms = Organisms::new(&mut rng, user_defined_parameter_bounds, &global_constants);

        World {
            _dimensions: dimensions,
            _last_division_index: last_division_index,
            _organisms: organisms,
            _regions: regions,
            _global_constants: global_constants,
            _rng: rng,
        }
    }
}
