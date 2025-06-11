use crate::parameters::global_constants::GlobalConstants;
use crate::world::dimensions::Dimensions;
use organisms::Organisms;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use regions::Regions; // Required for SmallRng::from_seed
use std::ops::RangeInclusive;

const DEFAULT_WORLD_SEED: u64 = 2_147_483_647; // A Mersenne prime (2^31 - 1)

pub mod dimensions;
pub mod organisms;
pub mod regions;

#[derive(Debug, Clone)]
pub struct World<F>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    _dimensions: Dimensions,
    _organisms: Organisms,
    _regions: Regions,
    _global_constants: GlobalConstants,
    _rng: SmallRng,
    _world_function: F,
}

impl<F> World<F>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    pub fn new(
        user_defined_parameter_bounds: &[RangeInclusive<f64>],
        global_constants: GlobalConstants,
        world_function: F,
    ) -> Self {
        let mut rng = SmallRng::seed_from_u64(DEFAULT_WORLD_SEED);
        let mut organisms =
            Organisms::new(user_defined_parameter_bounds, &global_constants, &mut rng);

        let spacial_limits = organisms.find_spacial_limits();
        // Pass global_constants to Dimensions::new, and spacial_limits by reference
        let mut dimensions = Dimensions::new(&spacial_limits, &global_constants);
        let mut regions = Regions::new(&global_constants);

        regions.update(&mut organisms, &mut dimensions);

        World {
            _dimensions: dimensions,
            _organisms: organisms,
            _regions: regions,
            _global_constants: global_constants,
            _rng: rng,
            _world_function: world_function,
        }
    }
}
