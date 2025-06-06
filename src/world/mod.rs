use dimension::Dimension;
use organisms::Organisms;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use regions::Regions; // Required for SmallRng::from_seed

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
        organisms: Organisms,
        regions: Regions,
        seed: u64,
    ) -> Self {
        World {
            _dimensions: dimensions,
            _last_division_index: last_division_index,
            _organisms: organisms,
            _regions: regions,
            rng: SmallRng::seed_from_u64(seed),
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
